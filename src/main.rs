use crate::config::{Config, ConfigError, config};
use axum::extract::OriginalUri;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Extension, Router, serve};
use axum_client_ip::ClientIp;
use bollard::Docker;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::mpsc;
use tower_http::add_extension::AddExtensionLayer;
use tracing::{error, info, instrument};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

mod config;
mod helpers;
mod metrics;

#[cfg(debug_assertions)]
fn start_tracing() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .init();
}

#[cfg(not(debug_assertions))]
fn start_tracing() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_file(false)
        .with_line_number(false)
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .init();
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error(transparent)]
    Config(#[from] &'static ConfigError),
    #[error("unable to connect to docker: {0}")]
    BollardConnect(bollard::errors::Error),
    #[error(transparent)]
    BollardNegotiate(bollard::errors::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = config()?;

    start_tracing();

    let docker = Docker::connect_with_defaults().map_err(Error::BollardConnect)?;

    let docker = docker
        .negotiate_version()
        .await
        .map_err(Error::BollardNegotiate)?;

    let mut metrics_registry = Registry::default();
    metrics::initialise(&mut metrics_registry, Arc::new(docker), config);

    let (exit_tx, mut exit_rx) = mpsc::channel::<Result<(), Error>>(1);

    start_http_server(Arc::new(metrics_registry), config, exit_tx.clone()).await?;

    tokio::spawn(async move {
        let ctrl_c = signal::ctrl_c().await;

        if let Err(error) = exit_tx.send(ctrl_c.map_err(Into::into)).await {
            panic!("unable to send exit from ctrl_c: {error}");
        };
    });

    info!("Ready!");

    if let Some(value) = exit_rx.recv().await {
        value?
    };

    exit_rx.close();

    Ok(())
}

async fn start_http_server(
    metrics_registry: Arc<Registry>,
    config: &Config,
    exit_tx: mpsc::Sender<Result<(), Error>>,
) -> Result<(), std::io::Error> {
    let addr = SocketAddr::from((config.listen_addr, config.listen_port));

    let listener = TcpListener::bind(addr).await?;

    let router = Router::new()
        .route("/", get(serve_metrics))
        .route("/metrics", get(serve_metrics))
        .route("/ping", get(ping))
        .layer(config.client_ip_source.clone().into_extension())
        .layer(AddExtensionLayer::new(metrics_registry));

    tokio::spawn(async move {
        let serve = serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;

        if let Err(error) = exit_tx.send(serve.map_err(Into::into)).await {
            panic!("unable to send exit from http serve: {error}");
        };
    });

    info!("Listening on http://{addr}");

    Ok(())
}

#[instrument(fields(path=path.path()), skip(metrics_registry))]
#[axum::debug_handler]
async fn serve_metrics(
    ClientIp(ip): ClientIp,
    OriginalUri(path): OriginalUri,
    metrics_registry: Extension<Arc<Registry>>,
) -> Result<String, StatusCode> {
    let mut buffer = String::new();

    match encode(&mut buffer, &metrics_registry) {
        Ok(_) => Ok(buffer),
        Err(error) => {
            error!("{error}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[axum::debug_handler]
async fn ping() -> (StatusCode, &'static str) {
    (StatusCode::OK, "pong")
}
