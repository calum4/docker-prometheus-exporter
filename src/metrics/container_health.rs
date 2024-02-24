use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use docker_api::Docker;
use docker_api::opts::{ContainerListOpts};
use metrics::{describe_gauge, gauge, Gauge};
use tracing::{debug, error, instrument};
use crate::metrics::Metric;

pub(crate) struct ContainerHealthMetric {
    metric: Gauge,
    docker: Arc<Docker>,
}

impl ContainerHealthMetric {
    pub(crate) fn new(docker: Arc<Docker>) -> Self {
        let gauge = gauge!(ContainerHealthMetric::NAME);
        describe_gauge!(ContainerHealthMetric::NAME, ContainerHealthMetric::DESCRIPTION);

        ContainerHealthMetric {
            metric: gauge,
            docker,
        }
    }
}

impl Debug for ContainerHealthMetric {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metric")
            .field("name", &Self::NAME)
            .finish()
    }
}

#[async_trait]
impl Metric for ContainerHealthMetric {
    const NAME: &'static str = "container_health";
    const DESCRIPTION: &'static str = "Reports the health state of Docker containers";
    const INTERVAL: Duration = Duration::from_secs(5);

    #[instrument]
    async fn update(&self) {
        let containers = self.docker.containers();

        let list = match containers.list(&ContainerListOpts::builder().all(true).build()).await {
            Ok(list) => list,
            Err(error) => {
                error!("Encountered error when fetching container list! {error}");
                // TODO - Report as unhealthy?
                return;
            }
        };

        for container in list {
            let truncated_id = container.id.clone().map(|id| id[..12].to_string());

            debug!(container.id = truncated_id, "Fetching metrics");

            let Some(id) = container.id else {
                continue
            };

            let inspect = match containers.get(id).inspect().await {
                Ok(inspect) => inspect,
                Err(error) => {
                    error!(container.id = truncated_id, "Encountered error when inspecting container! {error}");
                    // TODO - Report as unhealthy?
                    return;
                }
            };

            let Some(state) = inspect.state else {
                error!(container.id = truncated_id, "Container state was none!");
                // TODO - Report as unhealthy?
                return;
            };

            dbg!(state);
        }
    }
}
