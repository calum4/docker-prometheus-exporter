# Docker Prometheus Exporter

Exports basic metrics from Docker to the defined endpoint with path `/` or `/metrics` for scraping by Prometheus.

## V1 Breaking Changes 

- [`254a698`](https://github.com/Calum4/docker-prometheus-exporter/commit/254a698bf7ff0f02545208ff512a98ee5ef3cce6) - Removes `DOCKER_HOST` environment variable. Now
  connects via Unix Socket or Windows Pipe
- [`f7652c7`](https://github.com/Calum4/docker-prometheus-exporter/commit/f7652c7123f5d29774938d2c5af700f85cc7d516) - Default behaviour for `container_health` metric now only reports the health status on containers with the following label `docker-prometheus-exporter.metric.container_health.enabled`. This behaviour can be configured with the `CONTAINER_HEALTH_FILTER_LABEL` environment variable
- [`608a1eb`](https://github.com/Calum4/docker-prometheus-exporter/commit/608a1eb26b13a7667b28584d0a087ddc8f043d68) - Default listen address changed to `127.0.0.1`

## Available Metrics
| Metric Name        | Description                                    | Units/Values                                                                                | Labels                                          |
|--------------------|------------------------------------------------|---------------------------------------------------------------------------------------------|-------------------------------------------------|
| `docker_up`        | Reports the state of Docker                    | 0 - Offline<br/>1 - Online                                                                  | N/A                                             |
| `container_health` | Reports the health state of a Docker container | 0 - Unknown<br/>1 - Stopped<br/>2 - Alive, no healthcheck<br/>3 - Unhealthy<br/>4 - Healthy | `id` - Container ID<br/>`name` - Container Name |

## Environment Variables

| Name                            | Description                                                                                                                                                | Default       |
|---------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------|
| `RUST_LOG`                      | Sets logging verbosity, see [documentation](https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/filter/struct.EnvFilter.html#directives)          | `info`        |
| `LISTEN_ADDR`                   | Metrics endpoint listen address                                                                                                                            | `127.0.0.1`   |
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
      - LISTEN_ADDR=0.0.0.0
    expose:
      - "9000:9000"
    ports:
      - "127.0.0.1:9000:9000"
    labels:
      "docker-prometheus-exporter.metric.container_health.enabled": true
    restart: unless-stopped
```

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
