# Docker Prometheus Exporter

Exports basic metrics from Docker to the defined endpoint with path `/` or `/metrics` for scraping by Prometheus.

## Available Metrics
| Metric Name        | Description                                    | Units/Values                                                                                | Labels                                          |
|--------------------|------------------------------------------------|---------------------------------------------------------------------------------------------|-------------------------------------------------|
| `docker_up`        | Reports the state of Docker                    | 0 - Offline<br/>1 - Online                                                                  | N/A                                             |
| `container_health` | Reports the health state of a Docker container | 0 - Unknown<br/>1 - Stopped<br/>2 - Alive, no healthcheck<br/>3 - Unhealthy<br/>4 - Healthy | `id` - Container ID<br/>`name` - Container Name |

## Environment Variables

| Name                            | Description                                                                                                                                                | Default       |
|---------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------|
| `RUST_LOG`                      | Sets logging verbosity, see [documentation](https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/filter/struct.EnvFilter.html#directives)          | `info`        |
| `LISTEN_ADDR`                   | Metrics endpoint listen address                                                                                                                            | `0.0.0.0`     |
| `LISTEN_PORT`                   | Metrics endpoint listen port                                                                                                                               | `9000`        |
| `CLIENT_IP_SOURCE`              | Sets the Client IP source for logging, see [documentation](https://github.com/imbolc/axum-client-ip/blob/v1.0.0/src/lib.rs) for valid values               | `ConnectInfo` |
| `CONTAINER_HEALTH_FILTER_LABEL` | Whether the `container_health` metric should only report containers which have the `docker-prometheus-exporter.metric.container_health.enabled=true` label | `true`        |

## Example Docker Compose
```yaml
services:
  docker-prometheus-exporter:
    container_name: docker-prometheus-exporter
    image: ghcr.io/calum4/docker-prometheus-exporter:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    environment:
      - RUST_LOG=info,docker_prometheus_exporter=info
    expose:
      - "9000:9000"
    ports:
      - "127.0.0.1:9000:9000"
    labels:
      "docker-prometheus-exporter.metric.container_health.enabled": true
    restart: unless-stopped
```
