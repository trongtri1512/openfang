# syntax=docker/dockerfile:1
FROM rust:1-slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
# Use BuildKit cache mount to persist cargo registry & build artifacts across builds
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release --bin portal && \
    cp target/release/portal /usr/local/bin/portal

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/portal /usr/local/bin/
EXPOSE 4201
VOLUME /data
ENV PORTAL_DATA_DIR=/data
ENV PORTAL_LISTEN=0.0.0.0:4201
ENTRYPOINT ["portal"]
