use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use docker_api::Docker;
use tokio::time::interval;
use crate::metrics::up::UpMetric;
// use crate::metrics::container_health::ContainerHealthMetric;
// use crate::metrics::up::UpMetric;

pub(crate) mod up;
mod container_health;

#[async_trait]
trait Metric where Self: Send + 'static {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
    const INTERVAL: Duration;

    async fn update(&mut self);

    fn get_interval(&self) -> Duration {
        Self::INTERVAL
    }
}

pub(crate) fn load(docker: Arc<Docker>) {
    start(UpMetric::new(docker.clone()));
    // start(ContainerHealthMetric::new(docker.clone()));
}

fn start(mut metric: impl Metric) {
    tokio::spawn(async move {
        let mut interval = interval(metric.get_interval());

        loop {
            interval.tick().await;
            metric.update().await;
        }
    });
}
