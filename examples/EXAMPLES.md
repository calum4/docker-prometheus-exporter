# Run as Docker Container

All the Docker examples utilise Docker Compose, you can find an installation guide [here.](https://docs.docker.com/compose/install/)

## Usage

1. Copy one of the Docker Compose files from one of the methods listed below in the Methods section.
2. Paste the contents of the Docker Compose file to a new `compose.yml` file wherever you please.
3. Run `docker compose up -d`. Note depending on your Docker Compose installation, the command may instead be `docker-compose up -d`.
4. Docker Prometheus Exporter will now be available at [http://127.0.0.1:9000/metrics](http://127.0.0.1:9000/metrics).
5. Now you can set up Prometheus and enjoy your Docker metrics! 

## Methods

### Docker Socket Mounted (Not Recommended)

This method is not recommended for reasons described in the Security section of the README [here.](../README.md#security)

Compose file: [`compose.mounted.yml`](compose.mounted.yml)
