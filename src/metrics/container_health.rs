use crate::config::get_config;
use crate::helpers::ContainerId;
use crate::metrics::Metric;
use bollard::Docker;
use bollard::models::{
    ContainerState, ContainerStateStatusEnum, ContainerSummary, HealthStatusEnum,
};
use bollard::query_parameters::{InspectContainerOptionsBuilder, ListContainersOptionsBuilder};
use futures::stream::{self, StreamExt};
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::collections::HashMap;
use std::future;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug_span, error, instrument};

type ContainerName = String;

#[derive(Clone, Debug, Eq, Hash, PartialEq, EncodeLabelSet)]
struct Labels {
    id: ContainerId,
    name: ContainerName,
}

pub(crate) struct ContainerHealthMetric {
    metric: Arc<Family<Labels, Gauge>>,
    docker: Arc<Docker>,
}

impl Metric for ContainerHealthMetric {
    const NAME: &'static str = "container_health";
    const DESCRIPTION: &'static str = "Reports the health state of a Docker container";
    const INTERVAL: Duration = Duration::from_secs(15);

    fn new(registry: &mut Registry, docker: Arc<Docker>) -> Self {
        let metric = Family::<Labels, Gauge>::default();

        registry.register(Self::NAME, Self::DESCRIPTION, metric.clone());

        Self {
            metric: Arc::new(metric),
            docker,
        }
    }

    #[instrument(skip(self),fields(metric=Self::NAME))]
    async fn update(&mut self) {
        let mut filters: HashMap<&str, Vec<&str>> = HashMap::with_capacity(1);

        if get_config().container_health_label_filter {
            filters.insert(
                "label",
                vec!["docker-prometheus-exporter.metric.container_health.enabled=true"],
            );
        }

        let options = ListContainersOptionsBuilder::new()
            .all(true)
            .limit(i32::MAX)
            .size(false)
            .filters(&filters)
            .build();

        let summaries = {
            let summaries = self.docker.list_containers(Some(options)).await;

            self.metric.clear();

            match summaries {
                Ok(list) => list,
                Err(error) => {
                    error!("{error}");
                    return;
                }
            }
        };

        stream::iter(summaries.into_iter())
            .filter(|summary| {
                let is_blacklisted = summary.labels.as_ref().is_none_or(|labels| {
                    labels
                        .get("docker-prometheus-exporter.metric.container_health.enabled")
                        .is_some_and(|value| value.eq_ignore_ascii_case("false"))
                });

                future::ready(!is_blacklisted)
            })
            .map(|container| (container, self.docker.clone(), self.metric.clone()))
            .for_each_concurrent(Some(10), |(container, docker, metric)| async move {
                let id = match &container.id {
                    None => {
                        error!("A container did not have an id!");
                        return;
                    }
                    Some(id) => ContainerId::from(id.clone()),
                };

                let span = debug_span!("inspect", ?id);
                let _ = span.enter();

                let name = match get_container_name(&container) {
                    None => {
                        error!(?id, "Unable to fetch name from container!");
                        return;
                    }
                    Some(name) => name,
                };

                let options = InspectContainerOptionsBuilder::new().size(false).build();

                let inspect = match docker.inspect_container(id.as_str(), Some(options)).await {
                    Ok(inspect) => inspect,
                    Err(error) => {
                        error!(?id, "{error}");
                        return;
                    }
                };

                let Some(state) = inspect.state else {
                    error!(?id, "Container state was none!");
                    return;
                };

                let gauge = metric.get_or_create(&Labels { id, name });

                gauge.set(HealthStatus::from(state).into());
            })
            .await;
    }
}

enum HealthStatus {
    Unknown,
    Stopped,
    NoHealthCheck,
    Unhealthy,
    Healthy,
}

impl From<HealthStatus> for i64 {
    fn from(value: HealthStatus) -> Self {
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
        Some(names) => names.first().cloned(),
    }
}
