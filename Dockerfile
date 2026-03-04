# syntax=docker/dockerfile:1
FROM rust:1-slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# ── Step 1: Cache dependencies ──
# Copy only Cargo manifests + lock first (rarely changes)
COPY Cargo.toml Cargo.lock ./
COPY crates/openfang-api/Cargo.toml crates/openfang-api/Cargo.toml
COPY crates/openfang-channels/Cargo.toml crates/openfang-channels/Cargo.toml
COPY crates/openfang-cli/Cargo.toml crates/openfang-cli/Cargo.toml
COPY crates/openfang-desktop/Cargo.toml crates/openfang-desktop/Cargo.toml
COPY crates/openfang-extensions/Cargo.toml crates/openfang-extensions/Cargo.toml
COPY crates/openfang-hands/Cargo.toml crates/openfang-hands/Cargo.toml
COPY crates/openfang-kernel/Cargo.toml crates/openfang-kernel/Cargo.toml
COPY crates/openfang-memory/Cargo.toml crates/openfang-memory/Cargo.toml
COPY crates/openfang-migrate/Cargo.toml crates/openfang-migrate/Cargo.toml
COPY crates/openfang-runtime/Cargo.toml crates/openfang-runtime/Cargo.toml
COPY crates/openfang-skills/Cargo.toml crates/openfang-skills/Cargo.toml
COPY crates/openfang-types/Cargo.toml crates/openfang-types/Cargo.toml
COPY crates/openfang-wire/Cargo.toml crates/openfang-wire/Cargo.toml
COPY xtask/Cargo.toml xtask/Cargo.toml

# Create dummy source files so cargo can resolve the dependency tree
RUN for dir in crates/*/; do mkdir -p "$dir/src" && echo "" > "$dir/src/lib.rs"; done && \
    mkdir -p crates/openfang-cli/src && echo "fn main(){}" > crates/openfang-cli/src/main.rs && \
    mkdir -p xtask/src && echo "fn main(){}" > xtask/src/main.rs

# Build dependencies only (this layer is cached until Cargo.toml/Cargo.lock changes)
RUN cargo build --release --bin openfang 2>/dev/null || true

# ── Step 2: Build actual source ──
# Copy real source code (invalidates only this layer on code changes)
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
ENTRYPOINT ["openfang"]
CMD ["start"]
