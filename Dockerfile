FROM rust:bookworm as builder
ARG BUILD_ENVIRONMENT
WORKDIR /usr/src/docker-prometheus-exporter
COPY Cargo.lock Cargo.toml ./
COPY src/ src/
RUN echo "$BUILD_ENVIRONMENT" > .env && cargo install --path .

FROM debian:bookworm as app
RUN apt-get update && apt install -y ca-certificates && apt install -y openssl
WORKDIR /docker-prometheus-exporter
COPY --from=builder /usr/local/cargo/bin/docker-prometheus-exporter ./docker-prometheus-exporter
CMD ["./docker-prometheus-exporter"]
