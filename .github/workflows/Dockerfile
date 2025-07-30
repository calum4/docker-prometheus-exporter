FROM --platform=$BUILDPLATFORM rust:1-alpine3.22 AS chef

WORKDIR /app/

RUN apk add --no-cache musl-dev openssl-dev wget tar xz minisign zig

RUN cargo install --locked cargo-zigbuild cargo-chef
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl riscv64gc-unknown-linux-musl

FROM --platform=$BUILDPLATFORM chef AS planner

COPY Cargo.lock Cargo.toml ./
COPY src/ src/

RUN cargo chef prepare --recipe-path recipe.json

FROM --platform=$BUILDPLATFORM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json --zigbuild --target x86_64-unknown-linux-musl \
    --target aarch64-unknown-linux-musl --target riscv64gc-unknown-linux-musl

COPY Cargo.lock Cargo.toml ./
COPY src/ src/

RUN cargo zigbuild --release --target x86_64-unknown-linux-musl --target aarch64-unknown-linux-musl \
    --target riscv64gc-unknown-linux-musl

RUN mkdir /target && \
    cd target/ && \
    cp --parents */release/docker-prometheus-exporter /target/

FROM alpine:3.22 AS selector

ARG TARGETARCH

COPY --from=builder /target/ /target/

RUN case $TARGETARCH in \
      amd64) \
        target="x86_64-unknown-linux-musl" \
        ;; \
      arm64) \
        target="aarch64-unknown-linux-musl" \
        ;; \
      riscv64) \
        target="riscv64gc-unknown-linux-musl" \
        ;; \
      *) \
        echo "unknown arch" \
        exit 1 \
        ;; \
    esac && \
    mv "/target/$target/release/docker-prometheus-exporter" /docker-prometheus-exporter

FROM alpine:3.22 AS docker-prometheus-exporter

WORKDIR /app

LABEL org.opencontainers.image.source="https://github.com/Calum4/docker-prometheus-exporter"
LABEL org.opencontainers.image.description="Exports basic metrics from Docker for scraping by Prometheus"
LABEL org.opencontainers.image.licenses="MIT OR Apache2"

RUN apk add --no-cache curl

COPY --from=selector /docker-prometheus-exporter docker-prometheus-exporter

HEALTHCHECK --interval=15s --timeout=1s --retries=10 --start-period=15s \
    CMD curl -sSf -o /dev/null "http://${LISTEN_ADDR:-127.0.0.1}:${LISTEN_PORT:-9000}/ping" || exit 1

CMD ["/app/docker-prometheus-exporter"]
