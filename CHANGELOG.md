# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Support for cli args, try `--help`

### Changed
- BREAKING CHANGE: recoverable panics now instead exit with an error message 

### Security
- Updated `tracing-subscriber` to patch [CVE-2025-58160 / GHSA-xwfj-jgwm-7wp5](https://github.com/tokio-rs/tracing/security/advisories/GHSA-xwfj-jgwm-7wp5)

## [1.1.3] - 2025-08-29

### Changed
- Updated misc dependencies

### Security
- Updated `slab` to patch [RUSTSEC-2025-0047 / GHSA-qx2v-8332-m4fv](https://github.com/tokio-rs/slab/security/advisories/GHSA-qx2v-8332-m4fv)

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
