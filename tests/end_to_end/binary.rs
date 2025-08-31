use crate::common;
use crate::common::healthcheck::{HealthCheck, assert_healthcheck_metric};
use crate::common::test_environment::TestEnvironment;
use std::process::{Child, Command, Stdio};
use crate::common::run_mode::RunMode;

struct Dpe {
    process: Child,
}

impl Dpe {
    fn start(port: u16) -> Self {
        let process = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("--listen-addr")
            .arg("127.0.0.1")
            .arg("--listen-port")
            .arg(port.to_string())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        Self { process }
    }
}

impl Drop for Dpe {
    fn drop(&mut self) {
        if let Err(error) = self.process.kill() {
            eprintln!("failed to kill docker-prometheus-exporter process: {error}");
        }
    }
}

#[ignore]
#[tokio::test]
async fn binary() {
    const RUN_MODE: RunMode = RunMode::Binary;
    let port = common::available_port();

    let test_env = TestEnvironment::default();
    test_env.setup();

    let health_check = HealthCheck::new(test_env.temp_dir.as_path());
    health_check.start();

    let docker_version = Command::new("docker").arg("-v").output().unwrap();

    if !docker_version.status.success() {
        panic!("docker is not available");
    }

    let _dpe = Dpe::start(port);

    let metrics = common::get_metrics(port, test_env.id.as_str(), RUN_MODE).await;

    assert_healthcheck_metric(metrics.as_str(), test_env.id.as_str(), RUN_MODE);

    for line in metrics.lines() {
        if line.starts_with("docker_up{} 1") {
            return;
        }
    }
}
