use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct Config {
    pub listen_addr: IpAddr,
    pub listen_port: u16,
    pub container_health_label_filter: bool,
}

pub(crate) fn get_config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();

    fn construct() -> Config {
        Config {
            listen_addr: env::var("LISTEN_ADDR")
                .map(|addr| IpAddr::from_str(addr.as_str()).expect("Invalid LISTEN_ADDR provided"))
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            listen_port: env::var("LISTEN_PORT")
                .map(|port| u16::from_str(port.as_str()).expect("Invalid LISTEN_PORT provided"))
                .unwrap_or(9000),
            container_health_label_filter: env::var("CONTAINER_HEALTH_FILTER_LABEL").map(|val| val.eq_ignore_ascii_case("true")).unwrap_or(true),
        }
    }

    CONFIG.get_or_init(construct)
}
