use axum_client_ip::{ClientIpSource, ParseClientIpSourceError};
use std::env;
use std::net::{AddrParseError, IpAddr, Ipv4Addr};
use std::num::ParseIntError;
use std::str::FromStr;
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug)]
pub struct Config {
    pub listen_addr: IpAddr,
    pub listen_port: u16,
    pub client_ip_source: ClientIpSource,
    pub container_health_label_filter: bool,
}

#[derive(Error, Debug)]
pub(crate) enum ConfigError {
    #[error("invalid LISTEN_ADDR environment variable provided: {0}")]
    ListenAddr(#[from] AddrParseError),
    #[error("invalid LISTEN_PORT environment variable provided: {0}")]
    ListenPort(#[from] ParseIntError),
    #[error("invalid CLIENT_IP_SOURCE environment variable provided: {0}")]
    ClientSourceIp(#[from] ParseClientIpSourceError),
    #[error(
        "invalid CONTAINER_HEALTH_FILTER_LABEL environment variable provided, must be true/false, was '{0}'"
    )]
    ContainerHealthIp(String),
}

pub(crate) fn config() -> Result<&'static Config, &'static ConfigError> {
    static CONFIG: OnceLock<Result<Config, ConfigError>> = OnceLock::new();

    fn construct() -> Result<Config, ConfigError> {
        Ok(Config {
            listen_addr: env::var("LISTEN_ADDR")
                .map(|addr| IpAddr::from_str(addr.as_str()))
                .unwrap_or(Ok(IpAddr::V4(Ipv4Addr::LOCALHOST)))?,
            listen_port: env::var("LISTEN_PORT")
                .map(|port| u16::from_str(port.as_str()))
                .unwrap_or(Ok(9000))?,
            client_ip_source: env::var("CLIENT_IP_SOURCE")
                .map(|source| ClientIpSource::from_str(source.as_str()))
                .unwrap_or(Ok(ClientIpSource::ConnectInfo))?,
            container_health_label_filter: env::var("CONTAINER_HEALTH_FILTER_LABEL")
                .map(|val| {
                    if val.eq_ignore_ascii_case("true") {
                        Ok(true)
                    } else if val.eq_ignore_ascii_case("false") {
                        Ok(false)
                    } else {
                        Err(ConfigError::ContainerHealthIp(val))
                    }
                })
                .unwrap_or(Ok(true))?,
        })
    }

    CONFIG.get_or_init(construct).as_ref()
}
