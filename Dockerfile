# syntax=docker/dockerfile:1
FROM rust:1-slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY xtask ./xtask
COPY agents ./agents
COPY packages ./packages
RUN cargo build --release --bin openfang

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/openfang /usr/local/bin/
COPY --from=builder /build/agents /opt/openfang/agents
EXPOSE 4200
VOLUME /data
ENV OPENFANG_HOME=/data
ENV OPENFANG_LISTEN=0.0.0.0:4200
ENTRYPOINT ["/bin/sh", "-c", "\
    mkdir -p /data && \
    if [ -n \"$OPENFANG_API_KEY\" ] && [ ! -f /data/config.toml ]; then \
    printf '[kernel]\napi_key = \"%s\"\n' \"$OPENFANG_API_KEY\" > /data/config.toml; \
    elif [ -n \"$OPENFANG_API_KEY\" ] && ! grep -q 'api_key' /data/config.toml 2>/dev/null; then \
    printf '\n[kernel]\napi_key = \"%s\"\n' \"$OPENFANG_API_KEY\" >> /data/config.toml; \
    fi && \
    exec openfang start"]
