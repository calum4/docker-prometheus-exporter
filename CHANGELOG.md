# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.1] - 2025-04-20

### Fixed
- Misc documentation fixes

## [1.1.0] - 2025-04-19

### Added
- Returned support for the `DOCKER_HOST` environment variable
- Bundled compose file now utilises [docker-socket-proxy](https://github.com/linuxserver/docker-socket-proxy)
  for enhanced security
- Automatically negotiate API version with the connected docker daemon
- Blacklist container health reporting for a container by applying the `docker-prometheus-exporter.metric.container_health.enabled=false` 
  label

## [1.0.0] - 2025-04-15

### Added
- `/ping` endpoint, intended for a lightweight healthcheck
- Configurable client ip source for cases when running behind a reverse proxy. Configurable via the `CLIENT_IP_SOURCE` 
  environment variable, see [README.md](README.md#environment-variables)

### Changed
- BREAKING: Default behaviour for container_health metric now only reports the health status on containers with the 
  following label `docker-prometheus-exporter.metric.container_health.enabled`. Configurable with the 
  `CONTAINER_HEALTH_FILTER_LABEL` environment variable, see [README.md](README.md#environment-variables)
- BREAKING: Default listen address changed to `127.0.0.1`
- Migrated from [docker-api](https://github.com/vv9k/docker-api-rs) to `bollard`(https://github.com/fussybeaver/bollard)
- `container_health` metric collection performance enhanced on high container count hosts

### Removed
- BREAKING: `DOCKER_HOST` environment variable. Now connects via Unix Socket or Windows Pipe
