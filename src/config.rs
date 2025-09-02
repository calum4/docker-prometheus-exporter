use crate::metrics::container_health::ContainerHealthConfig;
use crate::metrics::up::UpConfig;
use axum_client_ip::ClientIpSource;
use clap::{Args, CommandFactory, FromArgMatches, Parser};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::OnceLock;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Config {
    /// Address to bind to
    #[arg(long, default_value_t = IpAddr::V4(Ipv4Addr::LOCALHOST))]
    pub(crate) listen_addr: IpAddr,
    /// Port to bind to
    #[arg(long, default_value_t = 9000)]
    pub(crate) listen_port: u16,
    /// Source for which the client's IP address is retrieved for logging.
    /// To see available options, visit the axum-client-ip [docs](https://github.com/imbolc/axum-client-ip/blob/2e3f353bbb04796aa6f7bde3e31a96129240afd5/src/lib.rs#L114)
    #[arg(long, default_value_t = ClientIpSource::ConnectInfo)]
    pub(crate) client_ip_source: ClientIpSource,
    /// Filter log output, format documentation from [tracing_subscriber](https://docs.rs/tracing-subscriber/0.3/tracing_subscriber/filter/struct.EnvFilter.html)
    #[arg(long, default_value_t)]
    pub(crate) rust_log: EnvFilter,
    #[command(flatten)]
    pub(crate) metrics: Metrics,
}

#[derive(Args, Debug)]
pub(crate) struct Metrics {
    #[command(flatten)]
    pub(crate) up: UpConfig,
    #[command(flatten)]
    pub(crate) container_health: ContainerHealthConfig,
}

pub(crate) fn config() -> Result<&'static Config, &'static clap::Error> {
    static CONFIG: OnceLock<Result<Config, clap::Error>> = OnceLock::new();

    fn construct() -> Result<Config, clap::Error> {
        let mut command = Config::command();
        command = command.mut_args(|arg| {
            let Some(id) = arg.get_long() else {
                return arg;
            };

            let name = id.replace(".", "_").replace("-", "_").to_uppercase();
            arg.env(name)
        });

        let matches = command.try_get_matches()?;
        Config::from_arg_matches(&matches)
    }

    CONFIG.get_or_init(construct).as_ref()
}
