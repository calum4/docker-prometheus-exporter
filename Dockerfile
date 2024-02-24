FROM rust:bookworm as builder
ARG BUILD_ENVIRONMENT
WORKDIR /usr/src/docker-prometheus-exporter
COPY Cargo.lock Cargo.toml ./
COPY src/ src/
RUN echo "$BUILD_ENVIRONMENT" > .env && cargo install --path .

FROM debian:bookworm as app
RUN apt-get update && apt install -y curl
WORKDIR /docker-prometheus-exporter
COPY --from=builder /usr/local/cargo/bin/docker-prometheus-exporter ./docker-prometheus-exporter

HEALTHCHECK --interval=15s --timeout=1s --retries=10 --start-period=15s \
    CMD curl -sSf -o /dev/null http://127.0.0.1:9000 || exit 1

CMD ["./docker-prometheus-exporter"]
