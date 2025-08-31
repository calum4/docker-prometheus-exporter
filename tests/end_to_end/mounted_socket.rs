use crate::common;
use crate::common::dpe::Dpe;
use crate::common::healthcheck::{HealthCheck, assert_healthcheck_metric};
use crate::common::run_mode::RunMode;
use crate::common::test_environment::TestEnvironment;

#[ignore]
#[tokio::test]
async fn mounted_socket() {
    const COMPOSE_CONTENTS: &str = include_str!("../../examples/compose.mounted.yml");
    const RUN_MODE: RunMode = RunMode::DockerSocketMounted;
    
    let port = common::available_port();

    let test_env = TestEnvironment::default();
    test_env.setup();

    let health_check = HealthCheck::new(test_env.temp_dir.as_path());
    health_check.start();

    let dpe = Dpe::new(test_env.temp_dir.as_path());
    dpe.start(port, COMPOSE_CONTENTS);

    let metrics =
        common::get_metrics(port, test_env.id.as_str(), RUN_MODE).await;
    assert_healthcheck_metric(metrics.as_str(), test_env.id.as_str(), RUN_MODE);
}
