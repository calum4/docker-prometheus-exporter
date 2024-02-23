use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use docker_api::Docker;
use tokio::time::interval;
use crate::metrics::docker_up::DockerUp;
use crate::metrics::test::Test;

pub(crate) mod docker_up;
mod test;

#[async_trait]
trait Metric where Self: Send + 'static {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
    const INTERVAL: Duration;

    async fn update(&self);

    fn get_interval(&self) -> Duration {
        Self::INTERVAL
    }
}

pub(crate) fn load(docker: Arc<Docker>) {
    start(DockerUp::new(docker));
    start(Test::new());
}

fn start(metric: impl Metric) {
    tokio::spawn(async move {
        let mut interval = interval(metric.get_interval());

        loop {
            interval.tick().await;
            metric.update().await;
        }
    });
}
