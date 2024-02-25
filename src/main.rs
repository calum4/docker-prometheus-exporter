use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use docker_api::Docker;
use http_body_util::Full;
use hyper::body::{Body, Bytes, Incoming};
use hyper::{http, Request, Response};
use hyper::header::CONTENT_TYPE;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use prometheus::{Encoder, TextEncoder};
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{error, info, instrument};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use crate::config::CONFIG_ENV;
use crate::metrics::load;

mod config;
mod metrics;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .init();

    if let Err(error) = start_http_server().await {
        panic!("{error}");
    }

    let docker = {
        let host = match &CONFIG_ENV.docker_host {
            None => {
                #[cfg(unix)]
                {
                    "unix:///var/run/docker.sock"
                }

                #[cfg(not(unix))]
                {
                    "tcp://127.0.0.1:2376"
                }
            }
            Some(host) => host,
        };

        Docker::new(host).unwrap()
    };

    load(Arc::new(docker));

    info!("Ready!");

    signal::ctrl_c().await.expect("Failed to listen for CTRL+C");
}

async fn start_http_server() -> tokio::io::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    let listener = TcpListener::bind(addr).await?;

    info!("Listening on http://{addr}");

    tokio::spawn(async move {
        loop {
            let (stream, addr) = match listener.accept().await {
                Ok((stream, addr)) => (stream, addr),
                Err(error) => {
                    error!("Errored when receiving connection! {error}");
                    continue
                }
            };

            let io = TokioIo::new(stream);

            tokio::spawn(async move {
                if let Err(error) = http1::Builder::new().serve_connection(io, service_fn(serve_req)).await {
                    error!(addr=addr.to_string(), "Errored when serving connection! {error}");
                }
            });
        }
    });

    Ok(())
}

// async fn serve_req(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, http::Error> {
//     let encoder = TextEncoder::new();
//
//     let metric_families = prometheus::gather();
//     let mut buffer = vec![];
//     encoder.encode(&metric_families, &mut buffer).unwrap();
//
//     Response::builder()
//         .status(200)
//         .header(CONTENT_TYPE, encoder.format_type())
//         .body(Full::from(Body::from(buffer)))
// }

async fn serve_req(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}
