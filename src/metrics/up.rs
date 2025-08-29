use crate::config::Config;
use crate::metrics::Metric;
use bollard::Docker;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, instrument};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Default, EncodeLabelSet)]
struct Labels {}

pub(crate) struct UpMetric {
    metric: Family<Labels, Gauge>,
    docker: Arc<Docker>,
}

impl Metric for UpMetric {
    const NAME: &'static str = "docker_up";
    const DESCRIPTION: &'static str = "Reports the state of Docker";
    const INTERVAL: Duration = Duration::from_secs(5);

    fn new(registry: &mut Registry, docker: Arc<Docker>, _config: &'static Config) -> Self {
        let metric = Family::<Labels, Gauge>::default();

        registry.register(Self::NAME, Self::DESCRIPTION, metric.clone());

        Self { metric, docker }
    }

    #[instrument(skip(self),fields(metric=Self::NAME))]
    async fn update(&mut self) {
        let up = match self.docker.ping().await {
            Ok(_) => 1,
            Err(error) => {
                error!("{error}");
                0
            }
        };

        self.metric.get_or_create(&Labels::default()).set(up);
    }
}
