FROM rust:alpine AS builder
ARG BUILD_ENVIRONMENT
WORKDIR /app/

RUN apk add --no-cache musl-dev openssl-dev

COPY Cargo.lock Cargo.toml ./
COPY src/ src/

RUN echo "$BUILD_ENVIRONMENT" > .env && cargo install --path .

FROM alpine:latest AS docker-prometheus-exporter

WORKDIR /app

LABEL org.opencontainers.image.source="https://github.com/Calum4/docker-prometheus-exporter"
LABEL org.opencontainers.image.description="Exports basic metrics from Docker for scraping by Prometheus"
LABEL org.opencontainers.image.licenses="MIT OR Apache2"

RUN apk add --no-cache curl

COPY --from=builder /usr/local/cargo/bin/docker-prometheus-exporter docker-prometheus-exporter

HEALTHCHECK --interval=15s --timeout=1s --retries=10 --start-period=15s \
    CMD curl -sSf -o /dev/null "http://${LISTEN_ADDR:-127.0.0.1}:${LISTEN_PORT:-9000}/ping" || exit 1

CMD ["/app/docker-prometheus-exporter"]
