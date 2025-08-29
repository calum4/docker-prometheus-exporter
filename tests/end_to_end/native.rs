use crate::common;
use crate::common::GetMetricsMode;
use crate::common::healthcheck::{HealthCheck, assert_healthcheck_metric};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

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

//#[ignore] TODO - Re-enable
#[tokio::test]
async fn native() {
    let dir_path = PathBuf::from("./tests/end_to_end/native/");
    let project_name = dir_path.file_name().unwrap();

    let port = common::available_port();

    let health_check = HealthCheck::new(&dir_path);
    health_check.start();

    let docker_version = Command::new("docker").arg("-v").output().unwrap();

    if !docker_version.status.success() {
        panic!("docker is not available");
    }

    let _dpe = Dpe::start(port);

    let metrics = common::get_metrics(port, project_name, GetMetricsMode::Native).await;

    assert_healthcheck_metric(metrics.as_str(), project_name, false);

    for line in metrics.lines() {
        if line.starts_with("docker_up{} 1") {
            return;
        }
    }
}
