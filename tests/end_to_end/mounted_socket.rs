use crate::common;
use crate::common::GetMetricsMode;
use crate::common::healthcheck::{HealthCheck, assert_healthcheck_metric};
use std::fs::{File, remove_file};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Dpe {
    compose_file_path: PathBuf,
}

impl Dpe {
    fn new(dir_path: &Path) -> Self {
        let compose_file_path = dir_path.join("compose.yml");

        Self { compose_file_path }
    }

    fn start(&self, port: u16) {
        let compose_template = include_str!("mounted_socket/compose.template.yml");
        let compose_contents =
            compose_template.replace("${DPE_MOUNTED_SOCKET_PORT}", port.to_string().as_str());

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
    let dir_path = PathBuf::from("./tests/end_to_end/mounted_socket/");
    let project_name = dir_path.file_name().unwrap();

    let port = common::available_port();

    let health_check = HealthCheck::new(&dir_path);
    health_check.start();

    let dpe = Dpe::new(&dir_path);
    dpe.start(port);

    let metrics = common::get_metrics(port, project_name, GetMetricsMode::new_docker()).await;
    assert_healthcheck_metric(metrics.as_str(), project_name, true);
}
