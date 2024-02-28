FROM rust:bookworm as builder
ARG BUILD_ENVIRONMENT
WORKDIR /usr/src/docker-prometheus-exporter
COPY Cargo.lock Cargo.toml ./
COPY src/ src/
RUN echo "$BUILD_ENVIRONMENT" > .env && cargo install --path .

FROM debian:bookworm-slim as app

RUN apt update \
    && apt install -y curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/docker-prometheus-exporter /docker-prometheus-exporter

HEALTHCHECK --interval=15s --timeout=1s --retries=10 --start-period=15s \
    CMD curl -sSf -o /dev/null "http://${LISTEN_ADDR:-127.0.0.1}:${LISTEN_PORT:-9000}" || exit 1

CMD ["/docker-prometheus-exporter"]
