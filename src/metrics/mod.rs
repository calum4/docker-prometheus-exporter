use crate::metrics::container_health::ContainerHealthMetric;
use crate::metrics::up::UpMetric;
use std::sync::Arc;
use std::time::Duration;
use bollard::Docker;
use tokio::time::interval;

mod container_health;
pub(crate) mod up;

trait Metric
where
    Self: Send + 'static,
{
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
    const INTERVAL: Duration;

    fn update(&mut self) -> impl Future<Output = ()> + Send;
}

pub(crate) fn load(docker: Arc<Docker>) {
    start(UpMetric::new(docker.clone()));
    start(ContainerHealthMetric::new(docker.clone()));
}

fn start<M>(mut metric: M)
where
    M: Metric,
{
    tokio::spawn(async move {
        let mut interval = interval(M::INTERVAL);

        loop {
            interval.tick().await;
            metric.update().await;
        }
    });
}
