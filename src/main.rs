use docker_api::Docker;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::signal;
use tracing::info;
use crate::config::CONFIG_ENV;

mod config;

pub(crate) trait Metric {

}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        //.compact() // TODO - Decide if to use compact or standard
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .init();

    let builder = PrometheusBuilder::new();
    builder.install().expect("Failed to install Prometheus Exporter");

    let docker = {
        let host = match &CONFIG_ENV.docker_host {
            None => {
                #[cfg(unix)]
                {
                    "unix://var/run/docker.sock"
                }

                #[cfg(not(unix))]
                {
                    "tcp://127.0.0.1:2376"
                }
            }
            Some(host) => host,
        };

        dbg!(host);

        Docker::new(host).expect("Failed to connect to Docker!")
    };

    let test = docker.ping().await.expect("Failed to connect to Docker!");

    dbg!(test, docker.info().await);

    info!("Ready!");

    signal::ctrl_c().await.expect("Failed to listen for CTRL+C");
}
