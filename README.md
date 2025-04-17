
<h1 align="center">
  <br>
  Docker Prometheus Exporter
  <br>
</h1>

<h4 align="center">Exports basic metrics from Docker for scraping by Prometheus </h4>

<p align="center">
  <a href="https://crates.io/crates/docker-prometheus-exporter">
    <img alt="Crates.io" src="https://img.shields.io/crates/v/docker-prometheus-exporter">
  </a>
  <a href="https://hub.docker.com/r/calum4/docker-prometheus-exporter">
    <img alt="Docker Hub" src="https://img.shields.io/docker/v/calum4/docker-prometheus-exporter?label=Docker%20Hub">
  </a>
  <a href="https://github.com/calum4/docker-prometheus-exporter/actions/workflows/audit.yml">
    <img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/calum4/docker-prometheus-exporter/audit.yml?label=cargo-audit">
  </a>
  <img alt="Crates.io License" src="https://img.shields.io/crates/l/docker-prometheus-exporter">
</p>

<p align="center">
  <a href="#changelog">Changelog</a> •
  <a href="#security">Security</a> •
  <a href="#usage">Usage</a> •
  <a href="#metrics">Metrics</a> •
  <a href="#environment-variables">Configuration</a> •
  <a href="#license">License</a> •
  <a href="#contributing">Contributing</a>
</p>

## Changelog

The full changelog can be found at [CHANGELOG.md](CHANGELOG.md)

### [1.0.0] - 2025-04-15

#### Added
- `/ping` endpoint, intended for a lightweight healthcheck
- Configurable client ip source for cases when running behind a reverse proxy. Configurable via the `CLIENT_IP_SOURCE`
  environment variable, see [README.md](README.md#environment-variables)

#### Changed
- BREAKING: Default behaviour for container_health metric now only reports the health status on containers with the
  following label `docker-prometheus-exporter.metric.container_health.enabled`. Configurable with the
  `CONTAINER_HEALTH_FILTER_LABEL` environment variable, see [README.md](README.md#environment-variables)
- BREAKING: Default listen address changed to `127.0.0.1`
- Migrated from [docker-api](https://github.com/vv9k/docker-api-rs) to `bollard`(https://github.com/fussybeaver/bollard)
- `container_health` metric collection performance enhanced on high container count hosts

#### Removed
- BREAKING: `DOCKER_HOST` environment variable. Now connects via Unix Socket or Windows Pipe

## Security

Docker Prometheus Exporter requires access to the Docker Engine API, this is inherently risky, however necessary
requirement.
> Docker socket /var/run/docker.sock is the UNIX socket that Docker is listening to. This is the primary entry point for
> the Docker API. The owner of this socket is root. Giving someone access to it is equivalent to giving unrestricted
> root access to your host.
>
> \- [OWASP - Docker Security Cheat Sheet](https://web.archive.org/web/20250330142850/https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html#rule-1-do-not-expose-the-docker-daemon-socket-even-to-the-containers)
> via [The Internet Archive](https://archive.org), accessed 2025-04-17

In an attempt to mitigate this risk, the recommended usage utilises [docker-socket-proxy](https://github.com/linuxserver/docker-socket-proxy)
in order to restrict the usage of most endpoints. This however is still not a perfect solution due to the reliance on
the `/containers` endpoint, which opens up other GET endpoints such as:
- [ContainerExport](https://docs.docker.com/reference/api/engine/version/v1.48/#tag/Container/operation/ContainerExport)
- [ContainerLogs](https://docs.docker.com/reference/api/engine/version/v1.48/#tag/Container/operation/ContainerLogs)
- [ContainerAttachWebsocket](https://docs.docker.com/reference/api/engine/version/v1.48/#tag/Container/operation/ContainerAttachWebsocket)

## Usage

Follow one of the installation methods detailed below

### Proxy Docker Socket (Recommended)

This method is **HIGHLY** recommended over directly mounting the Docker socket to the container, see the
[security section](#security).

```yaml
services:
  socket-proxy:
    image: lscr.io/linuxserver/socket-proxy:3.0.9
    container_name: socket-proxy
    environment:
      - CONTAINERS=1
      - PING=1
      - VERSION=1
      - EVENTS=0 # enabled by default
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
    expose:
      - "2357:2357/tcp"
    restart: unless-stopped
    read_only: true
    tmpfs:
      - /run
    labels:
      "docker-prometheus-exporter.metric.container_health.enabled": true

  docker-prometheus-exporter:
    image: calum4/docker-prometheus-exporter:1
    container_name: docker-prometheus-exporter
    environment:
      - RUST_LOG=info,docker_prometheus_exporter=info
      - LISTEN_ADDR=0.0.0.0
      - DOCKER_HOST=tcp://socket-proxy:2375
    ports:
      - "127.0.0.1:9000:9000"
    labels:
      "docker-prometheus-exporter.metric.container_health.enabled": true
    depends_on:
      - socket-proxy
    restart: unless-stopped
    read_only: true
    security_opt:
      - no-new-privileges=true
    user: "1000:1000"
```

### Mount Docker Socket
```yaml
services:
  docker-prometheus-exporter:
    container_name: docker-prometheus-exporter
    image: calum4/docker-prometheus-exporter:1
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
    environment:
      - RUST_LOG=info,docker_prometheus_exporter=info
      - LISTEN_ADDR=0.0.0.0
    ports:
      - "127.0.0.1:9000:9000"
    labels:
      "docker-prometheus-exporter.metric.container_health.enabled": true
    restart: unless-stopped
    read_only: true
```

### Other Methods
- [Crates.io](https://crates.io/crates/docker-prometheus-exporter)
- [Github Releases](https://github.com/calum4/docker-prometheus-exporter/releases)

## Metrics
| Metric Name        | Description                                    | Units/Values                                                                                | Labels                                          |
|--------------------|------------------------------------------------|---------------------------------------------------------------------------------------------|-------------------------------------------------|
| `docker_up`        | Reports the state of Docker                    | 0 - Offline<br/>1 - Online                                                                  | N/A                                             |
| `container_health` | Reports the health state of a Docker container | 0 - Unknown<br/>1 - Stopped<br/>2 - Alive, no healthcheck<br/>3 - Unhealthy<br/>4 - Healthy | `id` - Container ID<br/>`name` - Container Name |

## Configuration

### Environment Variables

| Name                            | Description                                                                                                                                                | Default                                                                             |
|---------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------|
| `RUST_LOG`                      | Sets logging verbosity, see [documentation](https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/filter/struct.EnvFilter.html#directives)          | `error`                                                                             |
| `DOCKER_HOST`                   | URI for the Docker Daemon                                                                                                                                  | Unix - `unix:///var/run/docker.sock`<br/>Windows - `npipe:////./pipe/docker_engine` |
| `LISTEN_ADDR`                   | Metrics endpoint listen address                                                                                                                            | `127.0.0.1`                                                                         |
| `LISTEN_PORT`                   | Metrics endpoint listen port                                                                                                                               | `9000`                                                                              |
| `CLIENT_IP_SOURCE`              | Sets the Client IP source for logging, see [documentation](https://github.com/imbolc/axum-client-ip/blob/v1.0.0/src/lib.rs) for valid values               | `ConnectInfo`                                                                       |
| `CONTAINER_HEALTH_FILTER_LABEL` | Whether the `container_health` metric should only report containers which have the `docker-prometheus-exporter.metric.container_health.enabled=true` label | `true`                                                                              |

### Container Labels

| Label                                                             | Description                                                                                                                                                        |
|-------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `docker-prometheus-exporter.metric.container_health.enabled=true` | When used in conjunction with the `CONTAINER_HEALTH_FILTER_LABEL=true` environment variable, enables the `container_health` metric for the corresponding container |


## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
