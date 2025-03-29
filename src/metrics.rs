use std::sync::Arc;
use std::time::Duration;
use bollard::Docker;
use prometheus_client::registry::Registry;
use tokio::time::interval;

mod container_health;
mod up;

macro_rules! metrics {
    ($($metric:ty),+ $(,)?) => {
        pub(crate) fn initialise(registry: &mut Registry, docker: Arc<Docker>) {
            $(
                start(<$metric>::new(registry, docker.clone()));
            )*
        }
    };
}

metrics! {
    up::UpMetric,
    container_health::ContainerHealthMetric,
}

trait Metric
where
    Self: Send + 'static,
{
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
    const INTERVAL: Duration;

    fn new(registry: &mut Registry, docker: Arc<Docker>) -> Self;
    fn update(&mut self) -> impl Future<Output = ()> + Send;
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
