use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::signal;

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
    builder.install().expect("failed to install recorder/exporter");

    signal::ctrl_c().await.expect("Failed to listen for CTRL+C");
}
