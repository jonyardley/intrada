FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

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
