use crate::helpers::ContainerId;
use crate::metrics::Metric;
use prometheus::{IntGaugeVec, Opts, register_int_gauge_vec};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use bollard::container::ListContainersOptions;
use bollard::Docker;
use bollard::models::{ContainerState, ContainerStateStatusEnum, ContainerSummary, HealthStatusEnum};
use tracing::{debug_span, error, instrument};

type ContainerName = String;

struct MetricLabels {
    id: ContainerId,
    name: ContainerName,
}

pub(crate) struct ContainerHealthMetric {
    metric: IntGaugeVec,
    cache: HashMap<ContainerId, MetricLabels>,
    docker: Arc<Docker>,
}

impl ContainerHealthMetric {
    const LABEL_NAMES: [&'static str; 2] = ["id", "name"];

    pub(crate) fn new(docker: Arc<Docker>) -> Self {
        let opts = Opts::new(Self::NAME, Self::DESCRIPTION);

        Self {
            metric: register_int_gauge_vec!(opts, &Self::LABEL_NAMES)
                .expect("unable to register container_health metric"),
            cache: HashMap::new(),
            docker,
        }
    }

    fn finish_update(&mut self, values: Vec<(ContainerId, ContainerName, HealthStatus)>) {
        for (id, name, value) in &values {
            let gauge = match self
                .metric
                .get_metric_with_label_values(&[id.get(), name.as_str()])
            {
                Ok(gauge) => gauge,
                Err(error) => {
                    error!(?id, "{error}");
                    continue;
                }
            };

            let id = id.clone();

            gauge.set(value.into());
            self.cache.insert(
                id.clone(),
                MetricLabels {
                    id,
                    name: name.clone(),
                },
            );
        }

        let remove_ids = self
            .cache
            .keys()
            .filter(|id| !values.iter().any(|(v_id, _, _)| &v_id == id))
            .cloned()
            .collect::<Vec<_>>();

        for id in remove_ids {
            let Some(cached_metric) = self.cache.remove(&id) else {
                continue;
            };

            let values = [cached_metric.id.get(), cached_metric.name.as_str()];

            if let Err(error) = self.metric.remove_label_values(&values) {
                error!(?id, "{error}");
            };
        }
    }
}

impl Metric for ContainerHealthMetric {
    const NAME: &'static str = "container_health";
    const DESCRIPTION: &'static str = "Reports the health state of a Docker container";
    const INTERVAL: Duration = Duration::from_secs(15);

    #[instrument(skip(self),fields(metric=Self::NAME))]
    async fn update(&mut self) {
        let filters: HashMap<&str, Vec<&str>> = HashMap::with_capacity(0);
        // TODO - Filter by label option

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
