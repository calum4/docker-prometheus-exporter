use crate::helpers::ContainerId;
use crate::metrics::Metric;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use bollard::container::ListContainersOptions;
use bollard::Docker;
use bollard::models::{ContainerState, ContainerStateStatusEnum, ContainerSummary, HealthStatusEnum};
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use tracing::{debug_span, error, instrument};
use crate::config::get_config;

type ContainerName = String;

#[derive(Clone, Debug, Eq, Hash, PartialEq, EncodeLabelSet)]
struct Labels {
    id: ContainerId,
    name: ContainerName,
}

pub(crate) struct ContainerHealthMetric {
    metric: Family<Labels, Gauge>,
    cache: HashMap<ContainerId, Labels>,
    docker: Arc<Docker>,
}

impl ContainerHealthMetric {
    // TODO - Remove clones when instantiating Label
    fn finish_update(&mut self, values: Vec<(ContainerId, ContainerName, HealthStatus)>) {
        self.metric.clear();

        for (id, name, value) in &values {
            let gauge = self
                .metric
                .get_or_create(&Labels { id: id.clone(), name: name.clone() });

            let id = id.clone();

            gauge.set(value.into());
            self.cache.insert(
                id.clone(),
                Labels {
                    id,
                    name: name.clone(),
                },
            );
        }
    }
}

impl Metric for ContainerHealthMetric {
    const NAME: &'static str = "container_health";
    const DESCRIPTION: &'static str = "Reports the health state of a Docker container";
    const INTERVAL: Duration = Duration::from_secs(15);

    fn new(registry: &mut Registry, docker: Arc<Docker>) -> Self {
        let metric = Family::<Labels, Gauge>::default();

        registry.register(Self::NAME, Self::DESCRIPTION, metric.clone());

        Self {
            metric,
            cache: HashMap::new(),
            docker,
        }
    }

    #[instrument(skip(self),fields(metric=Self::NAME))]
    async fn update(&mut self) {
        let mut filters: HashMap<&str, Vec<&str>> = HashMap::with_capacity(1);

        if get_config().container_health_label_filter {
            filters.insert("label", vec!["docker-prometheus-exporter.metric.container_health.enabled=true"]);
        }

        let options = ListContainersOptions {
            all: true,
            limit: None,
            size: false,
            filters,
        };

        let summaries = match self.docker.list_containers(Some(options)).await {
            Ok(list) => list,
            Err(error) => {
                error!("{error}");
                self.finish_update(vec![]);
                return;
            }
        };

        let mut values: Vec<(ContainerId, ContainerName, HealthStatus)> =
            Vec::with_capacity(summaries.len());

        for container in summaries {
            let id = match &container.id {
                None => {
                    error!("A container did not have an id!");
                    continue;
                }
                Some(id) => ContainerId::from(id.clone()),
            };

            let span = debug_span!("inspect", ?id);
            let _ = span.enter();

            let name = match get_container_name(&container) {
                None => {
                    error!(?id, "Unable to fetch name from container!");
                    continue;
                }
                Some(name) => name,
            };

            let inspect = match self.docker.inspect_container(id.get(), None).await {
                Ok(inspect) => inspect,
                Err(error) => {
                    error!(?id, "{error}");
                    continue;
                }
            };

            let Some(state) = inspect.state else {
                error!(?id, "Container state was none!");
                continue;
            };

            values.push((id, name, state.into()));
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
            return HealthStatus::Unknown;
        };

        if !matches!(status, ContainerStateStatusEnum::RUNNING) {
            return HealthStatus::Stopped;
        }

        let Some(health) = state.health.and_then(|h| h.status) else {
            return HealthStatus::NoHealthCheck;
        };

        match health {
            HealthStatusEnum::HEALTHY => HealthStatus::Healthy,
            _ => HealthStatus::Unhealthy,
        }
    }
}

fn get_container_name(container: &ContainerSummary) -> Option<ContainerName> {
    match &container.names {
        None => None,
        Some(names) => names.first().map(|n| n[1..].to_string()),
    }
}
