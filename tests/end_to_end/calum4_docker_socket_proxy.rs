use crate::common;
use crate::common::dpe::Dpe;
use crate::common::GetMetricsMode;
use crate::common::healthcheck::{assert_healthcheck_metric, HealthCheck};
use crate::common::test_environment::TestEnvironment;

#[ignore]
#[tokio::test]
async fn calum4_docker_socket_proxy() {
    const COMPOSE_CONTENTS: &str = include_str!("../../examples/compose.calum4.docker-socket-proxy.yml");

    let port = common::available_port();

    let test_env = TestEnvironment::default();
    test_env.setup();

    let health_check = HealthCheck::new(test_env.temp_dir.as_path());
    health_check.start();

    let dpe = Dpe::new(test_env.temp_dir.as_path());
    dpe.start(port, COMPOSE_CONTENTS);

    let metrics =
        common::get_metrics(port, test_env.id.as_str(), GetMetricsMode::new_docker()).await;
    assert_healthcheck_metric(metrics.as_str(), test_env.id.as_str(), true);
}
