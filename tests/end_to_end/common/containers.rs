use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::common::run_mode::RunMode;

#[derive(Debug, Eq, PartialEq)]
pub enum Containers {
    Dpe,
    Healthy,
    Unhealthy,
    NoHealthCheck,
    Stopped,
    DockerSocketProxy,
}

impl Containers {
    pub const TOTAL: u8 = 5;

    pub fn health(&self) -> &str {
        match self {
            Containers::Dpe => "4",
            Containers::Healthy => "4",
            Containers::Unhealthy => "3",
            Containers::NoHealthCheck => "2",
            Containers::Stopped => "1",
            Containers::DockerSocketProxy => "2",
        }
    }

    pub const fn total(run_mode: RunMode) -> u8 {
        match run_mode {
            RunMode::Binary => 4,
            RunMode::DockerSocketMounted => 5,
            RunMode::DockerSocketProxy => 6,
        }
    }
}

impl Display for Containers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Containers::Dpe => "docker-prometheus-exporter",
            Containers::Healthy => "healthy",
            Containers::Unhealthy => "unhealthy",
            Containers::NoHealthCheck => "no_health_check",
            Containers::Stopped => "stopped",
            Containers::DockerSocketProxy => "docker-socket-proxy",
        })
    }
}

impl FromStr for Containers {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "docker-prometheus-exporter" => Ok(Self::Dpe),
            "healthy" => Ok(Self::Healthy),
            "unhealthy" => Ok(Self::Unhealthy),
            "no_health_check" => Ok(Self::NoHealthCheck),
            "stopped" => Ok(Self::Stopped),
            "docker-socket-proxy" => Ok(Self::DockerSocketProxy),
            _ => Err(()),
        }
    }
}

#[test]
fn display_equals_from_str() {
    use Containers::*;

    assert_eq!(Dpe, Containers::from_str(Dpe.to_string().as_str()).unwrap());
    assert_eq!(
        Healthy,
        Containers::from_str(Healthy.to_string().as_str()).unwrap()
    );
    assert_eq!(
        Unhealthy,
        Containers::from_str(Unhealthy.to_string().as_str()).unwrap()
    );
    assert_eq!(
        NoHealthCheck,
        Containers::from_str(NoHealthCheck.to_string().as_str()).unwrap()
    );
    assert_eq!(
        Stopped,
        Containers::from_str(Stopped.to_string().as_str()).unwrap()
    );
    assert_eq!(
        DockerSocketProxy,
        Containers::from_str(DockerSocketProxy.to_string().as_str()).unwrap()
    );
}
