use crate::common::Containers;
use regex::Regex;
use std::fs::File;
use std::fs::remove_file;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

pub struct HealthCheck {
    compose_file_path: PathBuf,
}

fn copy_healthcheck_compose_file<P: AsRef<Path>>(path: P) {
    let compose_contents = include_str!("compose.healthchecks.yml");

    let mut compose_file = File::create(path).unwrap();
    compose_file.write_all(compose_contents.as_bytes()).unwrap();

    compose_file.flush().unwrap();
    compose_file.sync_all().unwrap();
}

impl HealthCheck {
    pub fn new(dir_path: &Path) -> Self {
        let compose_file_path = dir_path.join("compose.healthchecks.yml");

        Self { compose_file_path }
    }

    pub fn start(&self) {
        copy_healthcheck_compose_file(&self.compose_file_path);

        Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(&self.compose_file_path)
            .arg("up")
            .arg("-d")
            .output()
            .unwrap();
    }
}

impl Drop for HealthCheck {
    fn drop(&mut self) {
        let down = Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(&self.compose_file_path)
            .arg("down")
            .output();

        match down {
            Ok(mut out) if !out.status.success() => {
                out.stdout.push(b'\n');
                out.stdout.extend(out.stderr);

                match String::from_utf8(out.stdout) {
                    Ok(s) => eprintln!("failed to teardown healthcheck containers: {s}"),
                    Err(_) => eprintln!("failed to teardown healthcheck containers"),
                }
            }
            Err(error) => {
                eprintln!("failed to teardown healthcheck containers: {error}");
            }
            _ => {}
        }

        if let Err(error) = remove_file(&self.compose_file_path) {
            eprintln!("unable to remove healthcheck compose file: {error}");
        }
    }
}

pub fn assert_healthcheck_metric(metrics: &str, project_name: &str, dpe_running_on_docker: bool) {
    let regex = Regex::new(CONTAINER_HEALTH_REGEX).expect("regex is tested");

    let mut total_captured_containers: u8 = 0;

    for capture in regex.captures_iter(metrics) {
        let captured_project_name = capture.name("project_name").expect("regex is tested");
        if captured_project_name.as_str() != project_name {
            continue;
        }

        let name = capture.name("name").expect("regex is tested");
        let container =
            Containers::from_str(name.as_str()).expect("FromStr matches all container names");

        let health = capture.name("health").expect("regex is tested");
        assert_eq!(health.as_str(), container.health());

        total_captured_containers += 1;
    }

    let total_containers = if dpe_running_on_docker {
        Containers::TOTAL
    } else {
        Containers::TOTAL - 1
    };

    assert_eq!(total_captured_containers, total_containers);
}

pub const CONTAINER_HEALTH_REGEX: &str = r##"container_health\{id="(?<id>\w+)",name="/?(?<project_name>\w+_dpe_test)-(?<name>[\w_-]+)-\d"}\s(?<health>[1-4])"##;

#[test]
fn container_health_regex() {
    let regex = Regex::new(CONTAINER_HEALTH_REGEX).unwrap();
    let metrics = include_str!("./sample_metrics.txt");

    let mut captures_iter = regex.captures_iter(metrics);

    let stopped = captures_iter.next().unwrap();
    assert_eq!(
        stopped.name("id").map(|s| s.as_str()),
        Some("fd85dd9613b7b9ad537b5e5c7697761201fd4bac6f3e0595a3b630fda9aec0d0")
    );
    assert_eq!(
        stopped.name("project_name").map(|s| s.as_str()),
        Some("hJ9ev7F5QP_dpe_test")
    );
    assert_eq!(
        stopped.name("name").map(|s| s.as_str()),
        Some("docker-prometheus-exporter")
    );
    assert_eq!(stopped.name("health").map(|s| s.as_str()), Some("4"));

    let stopped = captures_iter.next().unwrap();
    assert_eq!(
        stopped.name("id").map(|s| s.as_str()),
        Some("b45e624803e848314164e3c4612d68769f91d2d5357a82a4882e27d6ac0e7381")
    );
    assert_eq!(
        stopped.name("project_name").map(|s| s.as_str()),
        Some("hJ9ev7F5QP_dpe_test")
    );
    assert_eq!(stopped.name("name").map(|s| s.as_str()), Some("unhealthy"));
    assert_eq!(stopped.name("health").map(|s| s.as_str()), Some("3"));

    let stopped = captures_iter.next().unwrap();
    assert_eq!(
        stopped.name("id").map(|s| s.as_str()),
        Some("012e2fd2391cd9adfde36a7f91ac77ef0753fd282b09f8ef2a8832533d4d587a")
    );
    assert_eq!(
        stopped.name("project_name").map(|s| s.as_str()),
        Some("hJ9ev7F5QP_dpe_test")
    );
    assert_eq!(
        stopped.name("name").map(|s| s.as_str()),
        Some("no_health_check")
    );
    assert_eq!(stopped.name("health").map(|s| s.as_str()), Some("2"));

    let stopped = captures_iter.next().unwrap();
    assert_eq!(
        stopped.name("id").map(|s| s.as_str()),
        Some("e7aa8d873ddf4bb520c962c3ae8feadc54568e2f9915a7d5cba40b30314a90c4")
    );
    assert_eq!(
        stopped.name("project_name").map(|s| s.as_str()),
        Some("hJ9ev7F5QP_dpe_test")
    );
    assert_eq!(stopped.name("name").map(|s| s.as_str()), Some("healthy"));
    assert_eq!(stopped.name("health").map(|s| s.as_str()), Some("4"));

    let stopped = captures_iter.next().unwrap();
    assert_eq!(
        stopped.name("id").map(|s| s.as_str()),
        Some("37ebddf22f6f064bbe982368f8f0ac090da590c2f9ef75b4cef0151471f3fcce")
    );
    assert_eq!(
        stopped.name("project_name").map(|s| s.as_str()),
        Some("hJ9ev7F5QP_dpe_test")
    );
    assert_eq!(stopped.name("name").map(|s| s.as_str()), Some("stopped"));
    assert_eq!(stopped.name("health").map(|s| s.as_str()), Some("1"));
}
