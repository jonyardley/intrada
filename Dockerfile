FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
# cmake is required by aws-lc-sys (transitive dep of jsonwebtoken's
# `aws_lc_rs` feature, which we use to avoid the rsa Marvin advisory).
# Installed in the base stage so both `chef prepare` and `chef cook` see it.
RUN apt-get update && apt-get install -y --no-install-recommends cmake \
    && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies — this is the caching Docker layer.
# --bin intrada-api scopes to only intrada-api's dependency graph, excluding
# crates/intrada-mobile's Tauri/GTK chain (gdk-sys etc.) which doesn't build
# in this minimal Rust container.
RUN cargo chef cook --release --recipe-path recipe.json --bin intrada-api
# Build application
COPY . .
# Baked into the binary at compile time via `option_env!("GIT_SHA")` and
# reported to Sentry as the `release` so events tie to deploys. Defaulted
# empty so local `docker build` without --build-arg still works.
# Placed AFTER cargo-chef cook so changes to GIT_SHA per deploy don't bust
# the dependency-build cache (the most expensive layer); only the final
# application-build layer re-runs, gated by build.rs's
# `cargo:rerun-if-env-changed=GIT_SHA`.
ARG GIT_SHA=""
ENV GIT_SHA=$GIT_SHA
RUN cargo build --release --bin intrada-api

# Runtime image — no Rust toolchain needed.
# Trixie matches the cargo-chef builder's glibc (binary built against glibc 2.38+
# crashes on bookworm's 2.36 with `version `GLIBC_2.38' not found`).
FROM debian:trixie-slim AS runtime
WORKDIR /app
# Install CA certificates for HTTPS connections to Turso
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/intrada-api /usr/local/bin
ENTRYPOINT ["/usr/local/bin/intrada-api"]
