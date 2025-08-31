use crate::common::run_mode::RunMode;
use crate::common::{Containers, print_process_output};
use regex::Regex;
use std::fs::remove_file;
use std::fs::{File, create_dir};
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
    pub fn new(dir_path: &Path, project_name: &str) -> Self {
        let compose_dir_path = dir_path.join(format!("{project_name}_healthcheck/"));
        create_dir(compose_dir_path.as_path()).unwrap();

        let compose_file_path = compose_dir_path.join("compose.healthchecks.yml");

        Self { compose_file_path }
    }

    pub fn start(&self) {
        copy_healthcheck_compose_file(&self.compose_file_path);

        let mut output = Command::new("docker");
        output
            .arg("compose")
            .arg("-f")
            .arg(&self.compose_file_path)
            .arg("up")
            .arg("-d");

        println!("Running: {:?}", &output);
        let output = output.output().unwrap();

        if !output.status.success() {
            print_process_output(&output);
            panic!("failed to bring up the healthcheck containers");
        }
    }
}

impl Drop for HealthCheck {
    fn drop(&mut self) {
        let mut down = Command::new("docker");
        down.arg("compose")
            .arg("-f")
            .arg(&self.compose_file_path)
            .arg("down");

        println!("Running: {:?}", &down);
        let down = down.output();

        match down {
            Ok(out) if !out.status.success() => {
                print_process_output(&out);
            }
            Err(error) => {
                eprintln!("failed to teardown healthcheck containers: {error:?}");
            }
            _ => {}
        }

        if let Err(error) = remove_file(&self.compose_file_path) {
            eprintln!("unable to remove healthcheck compose file: {error:?}");
        }
    }
}

pub fn assert_healthcheck_metric(metrics: &str, project_name: &str, run_mode: RunMode) {
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

    assert_eq!(total_captured_containers, Containers::total(run_mode));
}

pub const CONTAINER_HEALTH_REGEX: &str = r##"container_health\{id="(?<id>\w+)",name="/?(?<project_name>\w+_dpe_test)(_healthcheck)?-(?<name>[\w_-]+)-\d"}\s(?<health>[1-4])"##;

#[test]
fn container_health_regex() {
    let regex = Regex::new(CONTAINER_HEALTH_REGEX).unwrap();
    let metrics = include_str!("./sample_metrics.txt");

    let mut captures_iter = regex.captures_iter(metrics);

    let stopped = captures_iter.next().unwrap();
    assert_eq!(
        stopped.name("id").map(|s| s.as_str()),
        Some("0a98b9ee471bcdac5cd83b9ec728ae680aa42c71bf550d3a96541ae3b2c25d8c")
    );
    assert_eq!(
        stopped.name("project_name").map(|s| s.as_str()),
        Some("hvtss5ns87_dpe_test")
    );
    assert_eq!(
        stopped.name("name").map(|s| s.as_str()),
        Some("docker-prometheus-exporter")
    );
    assert_eq!(stopped.name("health").map(|s| s.as_str()), Some("4"));

    let stopped = captures_iter.next().unwrap();
    assert_eq!(
        stopped.name("id").map(|s| s.as_str()),
        Some("a270a323cd54407835904c89848ca708b42fa6069625c2834a73a84270479496")
    );
    assert_eq!(
        stopped.name("project_name").map(|s| s.as_str()),
        Some("hvtss5ns87_dpe_test")
    );
    assert_eq!(stopped.name("name").map(|s| s.as_str()), Some("unhealthy"));
    assert_eq!(stopped.name("health").map(|s| s.as_str()), Some("3"));

    let stopped = captures_iter.next().unwrap();
    assert_eq!(
        stopped.name("id").map(|s| s.as_str()),
        Some("0ed45f8703a36fb4c805a63f3b25a4fc535f24c1a6c885e5c7446a47e212b1b1")
    );
    assert_eq!(
        stopped.name("project_name").map(|s| s.as_str()),
        Some("hvtss5ns87_dpe_test")
    );
    assert_eq!(
        stopped.name("name").map(|s| s.as_str()),
        Some("no_health_check")
    );
    assert_eq!(stopped.name("health").map(|s| s.as_str()), Some("2"));

    let stopped = captures_iter.next().unwrap();
    assert_eq!(
        stopped.name("id").map(|s| s.as_str()),
        Some("0a1a965f18a4bbc72387c3c212f84f1470aee6419d5e2fcbb199ddd219e7daf7")
    );
    assert_eq!(
        stopped.name("project_name").map(|s| s.as_str()),
        Some("hvtss5ns87_dpe_test")
    );
    assert_eq!(stopped.name("name").map(|s| s.as_str()), Some("stopped"));
    assert_eq!(stopped.name("health").map(|s| s.as_str()), Some("1"));

    let stopped = captures_iter.next().unwrap();
    assert_eq!(
        stopped.name("id").map(|s| s.as_str()),
        Some("5506174db67fd79094e6344209c698c958909ae0573cf726fcb6be96c8f13f43")
    );
    assert_eq!(
        stopped.name("project_name").map(|s| s.as_str()),
        Some("hvtss5ns87_dpe_test")
    );
    assert_eq!(stopped.name("name").map(|s| s.as_str()), Some("healthy"));
    assert_eq!(stopped.name("health").map(|s| s.as_str()), Some("4"));
}
