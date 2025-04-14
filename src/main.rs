use crate::config::get_config;
use crate::metrics::load;
use axum::extract::OriginalUri;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Router, serve};
use axum_client_ip::{SecureClientIp, SecureClientIpSource};
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use std::sync::Arc;
use bollard::Docker;
use tokio::net::TcpListener;
use tokio::signal;
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

#[tokio::main]
async fn main() {
    start_tracing();

    start_http_server().await;

    let docker = Docker::connect_with_local_defaults().expect("unable to connect to docker");

    load(Arc::new(docker));

    info!("Ready!");

    signal::ctrl_c().await.expect("Failed to listen for CTRL+C");
}

async fn start_http_server() {
    let addr = SocketAddr::from((get_config().listen_addr, get_config().listen_port));

    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|_| panic!("unable to bind to {addr}"));

    let router = Router::new()
        .route("/", get(serve_metrics))
        .route("/metrics", get(serve_metrics))
        .layer(SecureClientIpSource::ConnectInfo.into_extension());

    tokio::spawn(async move {
        serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("unable to serve metrics")
    });

    info!("Listening on http://{addr}");
}

#[instrument(fields(path=path.path()))]
async fn serve_metrics(
    SecureClientIp(ip): SecureClientIp,
    OriginalUri(path): OriginalUri,
) -> Result<String, StatusCode> {
    let encoder = TextEncoder::new();

    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("unable to encode metrics");

    match String::from_utf8(buffer) {
        Ok(string) => Ok(string),
        Err(error) => {
            error!("{error}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
