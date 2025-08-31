pub mod containers;
pub mod dpe;
pub mod healthcheck;
pub mod run_mode;
pub mod test_environment;

use crate::common::containers::Containers;
use crate::common::dpe::{Dpe, DpeBinary, DpeDocker};
use crate::common::healthcheck::{HealthCheck, assert_healthcheck_metric};
use crate::common::run_mode::RunMode;
use crate::common::test_environment::TestEnvironment;
use rand::{Rng, rng};
use regex::Regex;
use reqwest::{Client, Request, Url};
use std::io::Read;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::process::{Child, Command, Output};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::interval;

fn random_port() -> u16 {
    rng().random_range(49152..=65535)
}

pub fn available_port() -> u16 {
    for _ in 0..30 {
        let port = random_port();
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

        if TcpListener::bind(addr).is_err() {
            continue;
        };

        return port;
    }

    panic!("unable to find available port");
}

enum GetMetricsMode {
    Binary,
    Docker {
        is_healthy: bool,
        last_output: Option<Output>,
    },
}

impl From<RunMode> for GetMetricsMode {
    fn from(run_mode: RunMode) -> Self {
        match run_mode {
            RunMode::Binary => Self::Binary,
            RunMode::DockerSocketMounted { .. } | RunMode::DockerSocketProxy { .. } => {
                Self::Docker {
                    is_healthy: false,
                    last_output: None,
                }
            }
        }
    }
}

enum LastMetricsResult {
    None,
    Error(reqwest::Error),
    Metrics(String),
}

pub async fn get_metrics(port: u16, project_name: &str, run_mode: RunMode) -> String {
    let regex = Regex::new(healthcheck::CONTAINER_HEALTH_REGEX).expect("tested");
    let mut mode = run_mode.into();

    let (client, req) = setup_metrics_req(port);

    let mut last_metrics_result = LastMetricsResult::None;

    let mut wakeup_interval = interval(Duration::from_secs(2));

    for _ in 0..35 {
        wakeup_interval.tick().await;

        if let GetMetricsMode::Docker {
            ref mut is_healthy,
            ref mut last_output,
        } = mode
        {
            let container_name = format!("/{project_name}-docker-prometheus-exporter-1");

            let output = Command::new("docker")
                .arg("inspect")
                .arg("--format='{{.State.Health.Status}}'")
                .arg(container_name)
                .output();

            let Ok(output) = output else {
                continue;
            };

            *last_output = Some(output.clone());

            if !output.status.success() {
                continue;
            }

            let Ok(health_status) = String::from_utf8(output.stdout) else {
                continue;
            };

            if !health_status.contains("healthy") {
                continue;
            }

            *is_healthy = true;
        }

        let res = client
            .execute(req.try_clone().expect("get request has no body"))
            .await;

        let res = match res {
            Ok(res) => res,
            Err(error) => {
                last_metrics_result = LastMetricsResult::Error(error);
                continue;
            }
        };

        let metrics = match res.text().await {
            Ok(metrics) => {
                last_metrics_result = LastMetricsResult::Metrics(metrics.clone());
                metrics
            }
            Err(error) => {
                last_metrics_result = LastMetricsResult::Error(error);
                continue;
            }
        };

        let healthcheck_container = match mode {
            GetMetricsMode::Binary => Containers::Healthy,
            GetMetricsMode::Docker { .. } => Containers::Dpe,
        };

        for capture in regex.captures_iter(metrics.as_str()) {
            let captured_project_name = capture.name("project_name").expect("regex is tested");
            if captured_project_name.as_str() != project_name {
                continue;
            }

            let name = capture.name("name").expect("regex tested");

            let Ok(container) = Containers::from_str(name.as_str()) else {
                continue;
            };

            if container != healthcheck_container {
                continue;
            }

            let health = capture.name("health").expect("regex tested");
            if health.as_str() != container.health() {
                break;
            }

            return metrics;
        }
    }

    println!("last metrics result:");
    match last_metrics_result {
        LastMetricsResult::None => println!("None"),
        LastMetricsResult::Error(error) => eprintln!("{error:?}"),
        LastMetricsResult::Metrics(metrics) => println!("{metrics}"),
    }

    match mode {
        GetMetricsMode::Binary => panic!("timed out before DPE was ready"),
        GetMetricsMode::Docker {
            is_healthy,
            last_output,
        } => {
            if !is_healthy {
                println!("docker-prometheus-exporter health status:");

                match last_output {
                    None => eprintln!("no output"),
                    Some(output) => print_process_output(&output),
                }
            }

            panic!("timed out before DPE was ready: is_healthy={is_healthy}");
        }
    }
}

fn setup_metrics_req(port: u16) -> (Client, Request) {
    let base_url = {
        let mut base_url = Url::parse("http://127.0.0.1/").expect("hardcoded");
        base_url
            .set_port(Some(port))
            .map_err(|_| "unable to set port".to_string())
            .expect("port to be in u16 range");
        base_url
    };

    let client = Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .unwrap();

    let req = client
        .get(base_url.join("/").expect("hardcoded"))
        .build()
        .expect("hardcoded");

    (client, req)
}

pub async fn test_metrics(run_mode: RunMode) {
    let port = available_port();

    let test_env = TestEnvironment::default();
    test_env.setup();

    let health_check = HealthCheck::new(test_env.temp_dir.as_path(), test_env.id.as_str());
    health_check.start();

    let docker_version = Command::new("docker").arg("-v").output().unwrap();
    if !docker_version.status.success() {
        panic!("docker is not available");
    }

    let compose_version = Command::new("docker")
        .arg("compose")
        .arg("version")
        .output()
        .unwrap();
    if !compose_version.status.success() {
        panic!("docker compose is not available");
    }

    let _dpe: Box<dyn Dpe> = match run_mode {
        RunMode::Binary => Box::new(DpeBinary::start(port)),
        RunMode::DockerSocketMounted { compose_contents }
        | RunMode::DockerSocketProxy { compose_contents } => Box::new(DpeDocker::start(
            port,
            test_env.temp_dir.as_path(),
            compose_contents,
        )),
    };

    let metrics = get_metrics(port, test_env.id.as_str(), run_mode).await;

    assert_healthcheck_metric(metrics.as_str(), test_env.id.as_str(), run_mode);

    let mut has_docker_up = false;
    for line in metrics.lines() {
        if line.starts_with("docker_up{} 1") {
            has_docker_up = true;
            break;
        }
    }

    assert!(has_docker_up);
}

/// Does not panic, safe to use in destructors
pub fn print_child_process_output(process: &mut Child) {
    if let Some(stdout) = process.stdout.as_mut() {
        let mut buf = String::new();

        if stdout.read_to_string(&mut buf).is_ok() {
            println!("{buf}");
        }
    }

    if let Some(stderr) = process.stderr.as_mut() {
        let mut buf = String::new();

        if stderr.read_to_string(&mut buf).is_ok() {
            eprintln!("{buf}");
        }
    }
}

/// Does not panic, safe to use in destructors
pub fn print_process_output(output: &Output) {
    if let Ok(stdout) = String::from_utf8(output.stdout.clone()) {
        println!("{stdout}");
    }

    if let Ok(stderr) = String::from_utf8(output.stderr.clone()) {
        eprintln!("{stderr}");
    }
}
