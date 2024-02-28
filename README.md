# Docker Prometheus Exporter

Exports basic metrics from Docker to the defined endpoint with path `/` or `/metrics` for scraping by Prometheus.

**Disclaimer** - I'm still new to Rust, feel free to make an issue and tell me why what I'm doing is dumb.

## Available Metrics
| Metric Name        | Description                                    | Units/Values                                                                                | Labels                                          |
|--------------------|------------------------------------------------|---------------------------------------------------------------------------------------------|-------------------------------------------------|
| `docker_up`        | Reports the state of Docker                    | 0 - Offline<br/>1 - Online                                                                  | N/A                                             |
| `container_health` | Reports the health state of a Docker container | 0 - Unknown<br/>1 - Stopped<br/>2 - Alive, no healthcheck<br/>3 - Unhealthy<br/>4 - Healthy | `id` - Container ID<br/>`name` - Container Name |

## Environment Variables

| Name          | Description                                                                                                                                       | Default                                                                 |
|---------------|---------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------|
| `RUST_LOG`    | Sets logging verbosity, see [documentation](https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/filter/struct.EnvFilter.html#directives) | `info`                                                                  |
| `DOCKER_HOST` | URI to the Docker API                                                                                                                             | Unix - `unix:///var/run/docker.sock`<br/>Other - `tcp://127.0.0.1:2376` |
| `LISTEN_ADDR` | Metrics endpoint listen address                                                                                                                   | `0.0.0.0`                                                               |
| `LISTEN_PORT` | Metrics endpoint listen port                                                                                                                      | `9000`                                                                  |

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
    restart: unless-stopped

```
