use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use docker_api::Docker;
use docker_api::models::{ContainerState, ContainerSummary};
use docker_api::opts::{ContainerListOpts};
use prometheus::{IntGauge, IntGaugeVec, Opts, register_int_gauge_vec};
use tracing::{debug_span, error, instrument};
use crate::metrics::Metric;

type ContainerId = String;
type ContainerName = String;

struct CachedMetric {
    id: ContainerId,
    name: ContainerName,
    metric: IntGauge,
}

pub(crate) struct ContainerHealthMetric {
    metric: IntGaugeVec,
    cache: HashMap<ContainerId, CachedMetric>,
    docker: Arc<Docker>,
}

impl ContainerHealthMetric {
    const LABEL_NAMES: [&'static str; 2] = ["id", "name"];

    pub(crate) fn new(docker: Arc<Docker>) -> Self {
        let opts = Opts::new(Self::NAME, Self::DESCRIPTION);

        Self {
            metric: register_int_gauge_vec!(opts, &Self::LABEL_NAMES).unwrap(),
            cache: HashMap::new(),
            docker,
        }
    }

    fn finish_update(&mut self, values: Vec<(ContainerId, ContainerName, HealthStatus)>) {
        for (id, name, value) in &values {
            let gauge = match self.metric.get_metric_with_label_values(&[id.as_str(), name.as_str()]) {
                Ok(gauge) => gauge,
                Err(error) => {
                    error!(id, "{error}"); // TODO - Truncate id to 12 chars
                    continue
                }
            };

            let id = id.clone();

            gauge.set(value.into());
            self.cache.insert(id.clone(), CachedMetric {
                id,
                name: name.clone(),
                metric: gauge,
            });
        }

        let remove_ids = self.cache.keys()
            .filter(|id| !values.iter().any(|(v_id, _, _)| &v_id == id))
            .cloned()
            .collect::<Vec<_>>();

        for id in remove_ids {
            let Some(cached_metric) = self.cache.remove(&id) else {
                continue
            };

            let values = [cached_metric.id.as_str(), cached_metric.name.as_str()];

            if let Err(error) = self.metric.remove_label_values(&values) {
                error!(id, "{error}"); // TODO - Truncate id
            };
        }
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
                error!("{error}");
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
                    error!(id=truncated_id, "{error}");
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

impl From<&HealthStatus> for i64 {
    fn from(value: &HealthStatus) -> Self {
        match value {
            HealthStatus::Unknown => 0,
            HealthStatus::Stopped => 1,
            HealthStatus::NoHealthCheck => 2,
            HealthStatus::Unhealthy => 3,
            HealthStatus::Healthy => 4,
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
