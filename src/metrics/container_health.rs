use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use docker_api::Docker;
use docker_api::models::{ContainerState, ContainerSummary};
use docker_api::opts::{ContainerListOpts};
use metrics::{describe_gauge, gauge, Gauge};
use tracing::{debug_span, error, instrument};
use crate::metrics::Metric;

type ContainerId = String;
type ContainerName = String;

pub(crate) struct ContainerHealthMetric {
    metrics: HashMap<ContainerId, Gauge>,
    docker: Arc<Docker>,
}

impl ContainerHealthMetric {
    pub(crate) fn new(docker: Arc<Docker>) -> Self {
        ContainerHealthMetric {
            metrics: HashMap::new(),
            docker,
        }
    }

    fn finish_update(&mut self, values: Vec<(ContainerId, ContainerName, HealthStatus)>) {
        let new_metrics: HashMap<ContainerId, Gauge> = HashMap::with_capacity(values.len());

        for (id, name, value) in values {
            let gauge = self.metrics.remove(&id).unwrap_or_else(|| {
                let gauge = gauge!(ContainerHealthMetric::NAME, "id"=>id, "name"=>name);
                describe_gauge!(ContainerHealthMetric::NAME, ContainerHealthMetric::DESCRIPTION);

                gauge
            });

            gauge.set::<f64>(value.into());
        }

        self.metrics = new_metrics; // TODO - Dropping the old vec doesn't unregister previous metrics
    }
}

#[async_trait]
impl Metric for ContainerHealthMetric {
    const NAME: &'static str = "container_health";
    const DESCRIPTION: &'static str = "Reports the health state of a Docker container";
    const INTERVAL: Duration = Duration::from_secs(15);

    #[instrument(skip(self),fields(metric=Self::NAME))]
    async fn update(&mut self) {
        let containers = self.docker.containers();

        let summaries = match containers.list(&ContainerListOpts::builder().all(true).build()).await {
            Ok(list) => list,
            Err(error) => {
                error!("Encountered error when fetching container list! {error}");
                self.finish_update(vec![]);
                return;
            }
        };

        let mut values: Vec<(ContainerId, ContainerName, HealthStatus)> = Vec::with_capacity(summaries.len());

        for container in summaries {
            let Some(id) = &container.id else {
                error!("A container did not have an id!");
                continue
            };

            let truncated_id = id[..12].to_string();

            let span = debug_span!("inspect", "id"=truncated_id);
            let _ = span.enter();

            let name = match get_container_name(&container) {
                None => {
                    error!(id=truncated_id, "Unable to fetch name from container!");
                    continue
                }
                Some(name) => name,
            };

            let inspect = match containers.get(id).inspect().await {
                Ok(inspect) => inspect,
                Err(error) => {
                    error!(id=truncated_id, "Encountered error when inspecting container! {error}");
                    continue
                }
            };

            let Some(state) = inspect.state else {
                error!(id=truncated_id, "Container state was none!");
                continue
            };

            values.push((id.clone(), name, state.into()));
        }

        self.finish_update(values);
    }
}

enum HealthStatus {
    Unknown,
    Stopped,
    NoHealthCheck,
    Unhealthy,
    Healthy,
}

impl From<HealthStatus> for f64 {
    fn from(value: HealthStatus) -> Self {
        match value {
            HealthStatus::Unknown => 0_f64,
            HealthStatus::Stopped => 1_f64,
            HealthStatus::NoHealthCheck => 2_f64,
            HealthStatus::Unhealthy => 3_f64,
            HealthStatus::Healthy => 4_f64,
        }
    }
}

impl From<ContainerState> for HealthStatus {
    fn from(state: ContainerState) -> Self {
        let Some(status) = state.status else {
            return HealthStatus::Unknown
        };

        if status != "running" {
            return HealthStatus::Stopped
        }

        let Some(health) = state.health.and_then(|h| h.status) else {
            return HealthStatus::NoHealthCheck
        };

        match health.as_str() {
            "healthy" => HealthStatus::Healthy,
            _ =>  HealthStatus::Unhealthy
        }
    }
}

fn get_container_name(container: &ContainerSummary) -> Option<ContainerName> {
    match &container.names {
        None => None,
        Some(names) => {
            names.first().map(|n| n[1..].to_string())
        }
    }
}
