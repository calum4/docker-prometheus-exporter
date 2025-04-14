use std::sync::Arc;
use std::time::Duration;
use docker_api::Docker;
use prometheus::{IntGauge, register_int_gauge};
use tracing::instrument;
use crate::metrics::Metric;

pub(crate) struct UpMetric {
    metric: IntGauge,
    docker: Arc<Docker>,
}

impl UpMetric {
    pub(crate) fn new(docker: Arc<Docker>) -> Self {
        UpMetric {
            metric: register_int_gauge!(Self::NAME, Self::DESCRIPTION).unwrap(),
            docker,
        }
    }
}

impl Metric for UpMetric {
    const NAME: &'static str = "docker_up";
    const DESCRIPTION: &'static str = "Reports the state of Docker";
    const INTERVAL: Duration = Duration::from_secs(5);

    #[instrument(skip(self),fields(metric=Self::NAME))]
    async fn update(&mut self) {
        let up = match self.docker.ping().await {
            Ok(_) => 1,
            Err(_) => 0,
        };

        self.metric.set(up);
    }
}
