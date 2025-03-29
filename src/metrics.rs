use std::sync::Arc;
use std::time::Duration;
use bollard::Docker;
use tokio::time::interval;

mod container_health;
mod up;

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
    start(up::UpMetric::new(docker.clone()));
    start(container_health::ContainerHealthMetric::new(docker.clone()));
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
