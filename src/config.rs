use lazy_static::lazy_static;
use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

#[derive(Debug)]
pub struct ConfigEnv {
    pub docker_host: Option<String>,
    pub listen_addr: IpAddr,
    pub listen_port: u16,
}

lazy_static! {
    pub static ref CONFIG_ENV: ConfigEnv = get_config_env();
}

fn get_config_env() -> ConfigEnv {
    ConfigEnv {
        docker_host: env::var("DOCKER_HOST").ok(),
        listen_addr: env::var("LISTEN_ADDR").map(|addr| IpAddr::from_str(addr.as_str()).expect("Invalid LISTEN_ADDR provided")).unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        listen_port: env::var("LISTEN_PORT").map(|port| u16::from_str(port.as_str()).expect("Invalid LISTEN_PORT provided")).unwrap_or(9000),
    }
}
