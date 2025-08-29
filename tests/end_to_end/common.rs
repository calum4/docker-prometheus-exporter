pub mod containers;
pub mod healthcheck;

use crate::common::containers::Containers;
use rand::{Rng, rng};
use regex::Regex;
use reqwest::{Client, Request, Url};
use std::ffi::OsStr;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::process::Command;
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

pub enum GetMetricsMode {
    Native,
    Docker { is_healthy: bool },
}

impl GetMetricsMode {
    pub fn new_docker() -> Self {
        Self::Docker { is_healthy: false }
    }
}

pub async fn get_metrics(port: u16, project_name: &OsStr, mut mode: GetMetricsMode) -> String {
    let regex = Regex::new(healthcheck::CONTAINER_HEALTH_REGEX).expect("tested");
    let (client, req) = setup_metrics_req(port);

    let mut wakeup_interval = interval(Duration::from_secs(2));

    for _ in 0..35 {
        wakeup_interval.tick().await;

        if let GetMetricsMode::Docker { ref mut is_healthy } = mode {
            let container_name = {
                let mut container_name = project_name.to_os_string();
                container_name.push("-docker-prometheus-exporter-1");
                container_name
            };

            let output = Command::new("docker")
                .arg("inspect")
                .arg("--format='{{.State.Health.Status}}'")
                .arg(container_name)
                .output();

            let Ok(output) = output else {
                continue;
            };

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

        let Ok(res) = client
            .execute(req.try_clone().expect("get request has no body"))
            .await
        else {
            continue;
        };

        let Ok(metrics) = res.text().await else {
            continue;
        };

        let healthcheck_container = match mode {
            GetMetricsMode::Native => Containers::Healthy,
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

    match mode {
        GetMetricsMode::Native => panic!("timed out before DPE was ready"),
        GetMetricsMode::Docker { is_healthy, .. } => {
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
