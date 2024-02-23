use std::sync::Arc;
use docker_api::Docker;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::signal;
use tracing::{info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use crate::config::CONFIG_ENV;
use crate::metrics::load;

mod config;
mod metrics;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let builder = PrometheusBuilder::new();
    builder.install().expect("Failed to install Prometheus Exporter");

    info!("test?");
    println!("HI");

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
