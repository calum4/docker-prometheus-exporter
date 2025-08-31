use crate::common;
use crate::common::GetMetricsMode;
use crate::common::healthcheck::{HealthCheck, assert_healthcheck_metric};
use crate::common::test_environment::TestEnvironment;
use std::env::current_dir;
use std::fs::{File, remove_file};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Dpe {
    compose_file_path: PathBuf,
}

impl Dpe {
    fn new(dir_path: &Path) -> Self {
        let compose_file_path = dir_path.join("compose.dpe.yml");

        Self { compose_file_path }
    }

    fn start(&self, port: u16) {
        let compose_template = include_str!("../../examples/compose.mounted.yml");
        let compose_contents = compose_template
            .replace(r##"127.0.0.1:9000:9000"##, format!("127.0.0.1:{}:9000", port).as_str())
            .replace(
                "image: calum4/docker-prometheus-exporter",
                format!("build: {}", current_dir().unwrap().to_str().unwrap()).as_str()
            );

        let mut compose_file = File::create(&self.compose_file_path).unwrap();
        compose_file.write_all(compose_contents.as_bytes()).unwrap();

        compose_file.flush().unwrap();
        compose_file.sync_all().unwrap();

        Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(&self.compose_file_path)
            .arg("up")
            .arg("-d")
            .arg("--build")
            .output()
            .unwrap();
    }
}

impl Drop for Dpe {
    fn drop(&mut self) {
        let down = Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(&self.compose_file_path)
            .arg("down")
            .arg("--rmi")
            .arg("local") // delete image
            .output();

        match down {
            Ok(mut out) if !out.status.success() => {
                out.stdout.push(b'\n');
                out.stdout.extend(out.stderr);

                match String::from_utf8(out.stdout) {
                    Ok(s) => {
                        eprintln!("failed to teardown docker-prometheus-exporter containers: {s}")
                    }
                    Err(_) => eprintln!("failed to teardown docker-prometheus-exporter containers"),
                }
            }
            Err(error) => {
                eprintln!("failed to teardown docker-prometheus-exporter containers: {error}");
            }
            _ => {}
        }

        if let Err(error) = remove_file(&self.compose_file_path) {
            eprintln!("unable to remove docker-prometheus-exporter compose file: {error}");
        }
    }
}

#[ignore]
#[tokio::test]
async fn mounted_socket() {
    let port = common::available_port();

    let test_env = TestEnvironment::default();
    test_env.setup();

    let health_check = HealthCheck::new(test_env.temp_dir.as_path());
    health_check.start();

    let dpe = Dpe::new(test_env.temp_dir.as_path());
    dpe.start(port);

    let metrics =
        common::get_metrics(port, test_env.id.as_str(), GetMetricsMode::new_docker()).await;
    assert_healthcheck_metric(metrics.as_str(), test_env.id.as_str(), true);
}
