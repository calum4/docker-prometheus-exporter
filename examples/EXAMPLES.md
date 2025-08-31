# Run as Docker Container

All the Docker examples utilise Docker Compose, you can find an installation guide [here.](https://docs.docker.com/compose/install/)

## Usage

1. Copy one of the Docker Compose files from one of the methods listed below in the Methods section.
2. Paste the contents of the Docker Compose file to a new `compose.yml` file wherever you please.
3. Run `docker compose up -d`. Note depending on your Docker Compose installation, the command may instead be `docker-compose up -d`.
4. Docker Prometheus Exporter will now be available at [http://127.0.0.1:9000/metrics](http://127.0.0.1:9000/metrics).
5. Now you can set up Prometheus and enjoy your Docker metrics! 

Configuration options are documented in the Configuration section of the README [here.](../README.md#Configuration)

## Methods

### Docker Socket Proxied

Proxying the Docker Socket is the preferred method of deployment, namely because it reduces the blast radius of any 
security vulnerabilities in docker-prometheus-exporter. However, this of course only shifts the security burden to the
socket proxy rather than eliminating it which is unfortunately not possible. 

Even so, this is not a silver bullet as docker-prometheus-exporter still requires access to endpoints which could be
exploited to collect sensitive information. You can read more about this in the Security Considerations section of the README
[here.](../README.md#security-considerations)

#### ([calum4/docker-socket-proxy](https://github.com/calum4/docker-socket-proxy))

Fork of [linuxserver/docker-socket-proxy](https://github.com/linuxserver/docker-socket-proxy) utilising HAProxy,
modified to enable fine-grained endpoint restriction for docker-prometheus-exporter. View the changes
[here](https://github.com/linuxserver/docker-socket-proxy/compare/main...calum4:docker-socket-proxy:main).

Compose file: [`compose.calum4.docker-socket-proxy.yml](compose.calum4.docker-socket-proxy.yml)

#### [wollomatic/socket-proxy](https://github.com/wollomatic/socket-proxy)

Highly configurable general purpose unix socket proxy written in Go with zero external dependencies.

Compose file: [`compose.wollomatic.socket-proxy.yml](compose.wollomatic.socket-proxy.yml)

#### [linuxserver/docker-socket-proxy](https://github.com/linuxserver/docker-socket-proxy)

Unlike [`calum4/docker-socket-proxy`](#calum4docker-socket-proxy) and [`wollomatic/socket-proxy`](#wollomaticsocket-proxy),
this does not provide fine-grained restriction to only the endpoints that `docker-prometheus-exporter` requires. 

Due to this, the `/containers` endpoint must be enabled, consequently opening other GET endpoints such as:
- [ContainerExport](https://docs.docker.com/reference/api/engine/version/v1.48/#tag/Container/operation/ContainerExport)
- [ContainerLogs](https://docs.docker.com/reference/api/engine/version/v1.48/#tag/Container/operation/ContainerLogs)
- [ContainerAttachWebsocket](https://docs.docker.com/reference/api/engine/version/v1.48/#tag/Container/operation/ContainerAttachWebsocket)

Compose file: [`compose.linuxserver.docker-socket-proxy.yml](compose.linuxserver.docker-socket-proxy.yml)

### Docker Socket Mounted (Not Recommended)

Directly mounting the Docker Socket is NOT recommended for reasons described in the Security Considerations section of the README [here.](../README.md#security-considerations)

Compose file: [`compose.mounted.yml`](compose.mounted.yml)

# Run Binary Directly

Running the binary directly is NOT recommended for reasons described in the Security Considerations section of the README [here.](../README.md#security-considerations)

## Usage

1. Install the binary by either downloading it from the GitHub Releases or by running `cargo install docker-prometheus-exporter`.
2. Run the binary with `docker-compose-exporter`, the default configuration will likely satisfy your need.

Configuration options are documented in the Configuration section of the README [here.](../README.md#Configuration)
