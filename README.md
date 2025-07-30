
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
  <a href="#usage">Usage</a> •
  <a href="#security">Security</a> •
  <a href="#metrics">Metrics</a> •
  <a href="#environment-variables">Configuration</a> •
  <a href="#license">License</a> •
  <a href="#contributing">Contributing</a>
</p>

## Changelog

The full changelog can be found at [CHANGELOG.md](CHANGELOG.md)

## [1.1.2] - 2025-07-30

### Changed
- Updated Alpine to v3.22
- Updated axum-client-ip to v1.1.3
- Updated bollard to v0.19.1
- Updated axum to v0.8.4
- Updated tower-http to v0.6.6
- Updated axum-client-ip to v1.1.3
- Updated tokio to v1.47.0
- Bumped misc dependencies (cargo update)

## Usage

Follow one of the installation methods detailed below

### Proxy Docker Socket (Recommended)

This method is **HIGHLY** recommended over directly mounting the Docker socket to the container, see the
[security section](#security).

```yaml
services:
  docker-socket-proxy:
    image: ghcr.io/calum4/docker-socket-proxy:latest
    container_name: docker-socket-proxy
    environment:
      - PING=1
      - VERSION=1
      - EVENTS=0 # enabled by default
      - CONTAINER_LIST=1
      - CONTAINER_INSPECT=1
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
    expose:
      - "2357:2357/tcp"
    restart: unless-stopped
    read_only: true
    security_opt:
      - no-new-privileges=true
    cap_drop:
      - ALL
    tmpfs:
      - /run
    networks:
      - docker-socket-proxy
    labels:
      "docker-prometheus-exporter.metric.container_health.enabled": true

  docker-prometheus-exporter:
    image: calum4/docker-prometheus-exporter:latest
    container_name: docker-prometheus-exporter
    environment:
      - RUST_LOG=info,docker_prometheus_exporter=info
      - LISTEN_ADDR=0.0.0.0
      - DOCKER_HOST=tcp://docker-socket-proxy:2375
    ports:
      - "127.0.0.1:9000:9000"
    labels:
      "docker-prometheus-exporter.metric.container_health.enabled": true
    depends_on:
      - docker-socket-proxy
    restart: unless-stopped
    read_only: true
    security_opt:
      - no-new-privileges=true
    cap_drop:
      - ALL
    networks:
      - docker-socket-proxy
      - docker-prometheus-exporter
    user: "65534:65534"

networks:
  docker-socket-proxy:
    driver: bridge
    internal: true
  docker-prometheus-exporter:
```

### Mount Docker Socket
```yaml
services:
  docker-prometheus-exporter:
    container_name: docker-prometheus-exporter
    image: calum4/docker-prometheus-exporter:1
    user: "0:0" # can instead be run as an unprivileged user with the docker group
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

## Security

Docker Prometheus Exporter requires access to the Docker Engine API, more specifically the following endpoints:

| Endpoint                                  | Usage                                              | Why is it needed?                        | Risks                                                                          |
|-------------------------------------------|----------------------------------------------------|------------------------------------------|--------------------------------------------------------------------------------|
| [/version][SystemVersion]                 | [main][main]                                       | API version negotiation                  | None known                                                                     |
| [/_ping][SystemPing]                      | [metric/up][metric/up]                             | Check whether the docker daemon is alive | None known                                                                     |
| [/containers/json][ContainerList]         | [metric/container_health][metric/container_health] | Fetch the names and ids of containers    | Provides basic information about a container                                   |
| [/containers/{id}/json][ContainerInspect] | [metric/container_health][metric/container_health] | Fetch the health status of the container | Provides extensive information on a container, including environment variables |

[SystemVersion]: https://docs.docker.com/reference/api/engine/version/v1.48/#tag/System/operation/SystemVersion
[SystemPing]: https://docs.docker.com/reference/api/engine/version/v1.48/#tag/System/operation/SystemPing
[ContainerList]: https://docs.docker.com/reference/api/engine/version/v1.48/#tag/Container/operation/ContainerList
[ContainerInspect]: https://docs.docker.com/reference/api/engine/version/v1.48/#tag/Container/operation/ContainerInspect

[main]: https://github.com/calum4/docker-prometheus-exporter/blob/d3a55a1a7a88b99fad7d4ccf3096551725de67e3/src/main.rs
[metric/up]: https://github.com/calum4/docker-prometheus-exporter/blob/dcc1453b4d2c36322310b4454bcae8eced9ea305/src/metrics/up.rs
[metric/container_health]: https://github.com/calum4/docker-prometheus-exporter/blob/dcc1453b4d2c36322310b4454bcae8eced9ea305/src/metrics/container_health.rs

Providing unrestricted access to the Docker socket is highly discouraged. 
> Docker socket /var/run/docker.sock is the UNIX socket that Docker is listening to. This is the primary entry point for
> the Docker API. The owner of this socket is root. Giving someone access to it is equivalent to giving unrestricted
> root access to your host.
>
> \- [OWASP - Docker Security Cheat Sheet](https://web.archive.org/web/20250330142850/https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html#rule-1-do-not-expose-the-docker-daemon-socket-even-to-the-containers)
> via [The Internet Archive](https://archive.org), accessed 2025-04-17

Therefore, it is recommended that access to the Docker socket is proxied, and endpoints whitelisted. 

### [calum4/docker-socket-proxy](https://github.com/calum4/docker-socket-proxy)

Fork of [linuxserver/docker-socket-proxy](https://github.com/linuxserver/docker-socket-proxy) utilising HAProxy, 
modified to enable fine-grained endpoint restriction for docker-prometheus-exporter. View the changes 
[here](https://github.com/linuxserver/docker-socket-proxy/compare/main...calum4:docker-socket-proxy:main).

<details>
  <summary>View docker-compose.yml</summary>

  ```yaml
  services:
    docker-socket-proxy:
      image: ghcr.io/calum4/docker-socket-proxy:latest
      container_name: docker-socket-proxy
      environment:
        - PING=1
        - VERSION=1
        - EVENTS=0 # enabled by default
        - CONTAINER_LIST=1
        - CONTAINER_INSPECT=1
      volumes:
        - /var/run/docker.sock:/var/run/docker.sock:ro
      expose:
        - "2357:2357/tcp"
      restart: unless-stopped
      read_only: true
      security_opt:
        - no-new-privileges=true
      cap_drop:
        - ALL
      tmpfs:
        - /run
      networks:
        - docker-socket-proxy
      labels:
        "docker-prometheus-exporter.metric.container_health.enabled": true
  
    docker-prometheus-exporter:
      image: calum4/docker-prometheus-exporter:latest
      container_name: docker-prometheus-exporter
      environment:
        - RUST_LOG=info,docker_prometheus_exporter=info
        - LISTEN_ADDR=0.0.0.0
        - DOCKER_HOST=tcp://docker-socket-proxy:2375
      ports:
        - "127.0.0.1:9000:9000"
      labels:
        "docker-prometheus-exporter.metric.container_health.enabled": true
      depends_on:
        - docker-socket-proxy
      restart: unless-stopped
      read_only: true
      security_opt:
        - no-new-privileges=true
      cap_drop:
        - ALL
      networks:
        - docker-socket-proxy
        - docker-prometheus-exporter
      user: "65534:65534"
  
  networks:
    docker-socket-proxy:
      driver: bridge
      internal: true
    docker-prometheus-exporter:
  ```

</details>

### [wollomatic/socket-proxy](https://github.com/wollomatic/socket-proxy)

Highly configurable general purpose unix socket proxy written in Go with zero external dependencies.

<details>
  <summary>View docker-compose.yml</summary>

  ```yaml
  services:
    docker-socket-proxy:
      image: wollomatic/socket-proxy:1
      container_name: docker-socket-proxy
      restart: unless-stopped
      user: "0:0" # can instead be run as an unprivileged user with the docker group
      mem_limit: 64M
      read_only: true
      cap_drop:
        - ALL
      security_opt:
        - no-new-privileges
      command:
        - '-loglevel=info'
        - '-listenip=0.0.0.0'
        - '-allowfrom=docker-prometheus-exporter'
        - '-allowGET=^(/v[\d\.]+)?/((version)|(_ping)|(containers/json)|(containers/[a-zA-Z0-9_.-]+/json))$'
        - '-watchdoginterval=3600' # check once per hour for socket availability
        - '-stoponwatchdog' # halt program on error and let compose restart it
        - '-shutdowngracetime=5' # wait 5 seconds before shutting down
      volumes:
        - /var/run/docker.sock:/var/run/docker.sock:ro
      networks:
        - docker-socket-proxy
      labels:
        "docker-prometheus-exporter.metric.container_health.enabled": true
  
    docker-prometheus-exporter:
      image: calum4/docker-prometheus-exporter:latest
      container_name: docker-prometheus-exporter
      environment:
        - RUST_LOG=info,docker_prometheus_exporter=info
        - LISTEN_ADDR=0.0.0.0
        - DOCKER_HOST=tcp://docker-socket-proxy:2375
      ports:
        - "127.0.0.1:9000:9000"
      labels:
        "docker-prometheus-exporter.metric.container_health.enabled": true
      depends_on:
        - docker-socket-proxy
      restart: unless-stopped
      read_only: true
      security_opt:
        - no-new-privileges=true
      cap_drop:
        - ALL
      networks:
        - docker-socket-proxy
        - docker-prometheus-exporter
      user: "65534:65534"
  
  networks:
    docker-socket-proxy:
      driver: bridge
      internal: true
    docker-prometheus-exporter:
  ```

</details>

### [linuxserver/docker-socket-proxy](https://github.com/linuxserver/docker-socket-proxy)

Unlike the previous 2 options, this does not provide fine-grained restriction to only the endpoints that
`docker-prometheus-exporter` requires. Due to this, the `/containers` endpoint must be enabled, consequently opening
other GET endpoints such as:
- [ContainerExport](https://docs.docker.com/reference/api/engine/version/v1.48/#tag/Container/operation/ContainerExport)
- [ContainerLogs](https://docs.docker.com/reference/api/engine/version/v1.48/#tag/Container/operation/ContainerLogs)
- [ContainerAttachWebsocket](https://docs.docker.com/reference/api/engine/version/v1.48/#tag/Container/operation/ContainerAttachWebsocket)

<details>
  <summary>View docker-compose.yml</summary>

  ```yaml
  services:
    docker-socket-proxy:
      image: lscr.io/linuxserver/socket-proxy:latest
      container_name: docker-socket-proxy
      environment:
        - PING=1
        - VERSION=1
        - EVENTS=0 # enabled by default
        - CONTAINERS=1
      volumes:
        - /var/run/docker.sock:/var/run/docker.sock:ro
      expose:
        - "2357:2357/tcp"
      restart: unless-stopped
      read_only: true
      security_opt:
        - no-new-privileges=true
      cap_drop:
        - ALL
      tmpfs:
        - /run
      networks:
        - docker-socket-proxy
      labels:
        "docker-prometheus-exporter.metric.container_health.enabled": true
  
    docker-prometheus-exporter:
      image: calum4/docker-prometheus-exporter:latest
      container_name: docker-prometheus-exporter
      environment:
        - RUST_LOG=info,docker_prometheus_exporter=info
        - LISTEN_ADDR=0.0.0.0
        - DOCKER_HOST=tcp://docker-socket-proxy:2375
      ports:
        - "127.0.0.1:9000:9000"
      labels:
        "docker-prometheus-exporter.metric.container_health.enabled": true
      depends_on:
        - docker-socket-proxy
      restart: unless-stopped
      read_only: true
      security_opt:
        - no-new-privileges=true
      cap_drop:
        - ALL
      networks:
        - docker-socket-proxy
        - docker-prometheus-exporter
      user: "65534:65534"

  networks:
    docker-socket-proxy:
      driver: bridge
      internal: true
    docker-prometheus-exporter:
  ```

</details>

## Metrics
| Metric Name        | Description                                    | Units/Values                                                                                | Labels                                          |
|--------------------|------------------------------------------------|---------------------------------------------------------------------------------------------|-------------------------------------------------|
| `docker_up`        | Reports the state of Docker                    | 0 - Offline<br/>1 - Online                                                                  | N/A                                             |
| `container_health` | Reports the health state of a Docker container | 0 - Unknown<br/>1 - Stopped<br/>2 - Alive, no healthcheck<br/>3 - Unhealthy<br/>4 - Healthy | `id` - Container ID<br/>`name` - Container Name |

## Configuration

### Environment Variables

| Name                            | Description                                                                                                                                                                         | Default                                                                             |
|---------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------|
| `RUST_LOG`                      | Sets logging verbosity, see [documentation](https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/filter/struct.EnvFilter.html#directives)                                   | `error`                                                                             |
| `DOCKER_HOST`                   | URI for the Docker Daemon                                                                                                                                                           | Unix - `unix:///var/run/docker.sock`<br/>Windows - `npipe:////./pipe/docker_engine` |
| `LISTEN_ADDR`                   | Metrics endpoint listen address                                                                                                                                                     | `127.0.0.1`                                                                         |
| `LISTEN_PORT`                   | Metrics endpoint listen port                                                                                                                                                        | `9000`                                                                              |
| `CLIENT_IP_SOURCE`              | Sets the Client IP source for logging, see [documentation](https://github.com/imbolc/axum-client-ip/blob/2e3f353bbb04796aa6f7bde3e31a96129240afd5/src/lib.rs#L114) for valid values | `ConnectInfo`                                                                       |
| `CONTAINER_HEALTH_FILTER_LABEL` | Whether the `container_health` metric should only report containers which have the `docker-prometheus-exporter.metric.container_health.enabled=true` label                          | `true`                                                                              |

### Container Labels

| Label                                                              | Description                                                                                                                                                        |
|--------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `docker-prometheus-exporter.metric.container_health.enabled=true`  | When used in conjunction with the `CONTAINER_HEALTH_FILTER_LABEL=true` environment variable, enables the `container_health` metric for the corresponding container |
| `docker-prometheus-exporter.metric.container_health.enabled=false` | Disables the `container_health` metric for the corresponding container, regardless of the `CONTAINER_HEALTH_FILTER_LABEL` environment variable                     |


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
