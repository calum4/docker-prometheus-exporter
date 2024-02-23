use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use docker_api::Docker;
use metrics::{describe_gauge, gauge, Gauge};
use tracing::instrument;
use crate::metrics::Metric;

pub(crate) struct DockerUp {
    metric: Gauge,
    docker: Arc<Docker>,
}

impl DockerUp {
    pub(crate) fn new(docker: Arc<Docker>) -> Self {
        let gauge = gauge!(DockerUp::NAME);
        describe_gauge!(DockerUp::NAME, DockerUp::DESCRIPTION);

        DockerUp {
            metric: gauge,
            docker,
        }
    }
}

#[async_trait]
impl Metric for DockerUp {
    const NAME: &'static str = "docker_up";
    const DESCRIPTION: &'static str = "Reports the state of Docker";
    const INTERVAL: Duration = Duration::from_secs(5);

    #[instrument]
    async fn update(&self) {
        let up = match self.docker.ping().await {
            Ok(_) => 1_f64,
            Err(_) => 0_f64,
        };

        self.metric.set(up);
    }
}

impl Debug for DockerUp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metric")
            .field("name", &Self::NAME)
            .finish()
    }
}
