// use std::sync::Arc;
// use std::time::Duration;
// use async_trait::async_trait;
// use docker_api::Docker;
// use metrics::{describe_gauge, gauge, Gauge};
// use tracing::instrument;
// use crate::metrics::Metric;
//
// pub(crate) struct UpMetric {
//     metric: Gauge,
//     docker: Arc<Docker>,
// }
//
// impl UpMetric {
//     pub(crate) fn new(docker: Arc<Docker>) -> Self {
//         let gauge = gauge!(UpMetric::NAME);
//         describe_gauge!(UpMetric::NAME, UpMetric::DESCRIPTION);
//
//         UpMetric {
//             metric: gauge,
//             docker,
//         }
//     }
// }
//
// #[async_trait]
// impl Metric for UpMetric {
//     const NAME: &'static str = "docker_up";
//     const DESCRIPTION: &'static str = "Reports the state of Docker";
//     const INTERVAL: Duration = Duration::from_secs(5);
//
//     #[instrument(skip(self),fields(metric=Self::NAME))]
//     async fn update(&mut self) {
//         let up = match self.docker.ping().await {
//             Ok(_) => 1_f64,
//             Err(_) => 0_f64,
//         };
//
//         self.metric.set(up);
//     }
// }
