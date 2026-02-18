set dotenv-load

# Default: show available commands
default:
    @just --list

# Start both API and web dev servers concurrently
dev:
    #!/usr/bin/env bash
    set -e
    trap 'kill 0' EXIT
    cargo run -p intrada-api &
    trunk serve --config crates/intrada-web/Trunk.toml &
    wait

# Start only the API server
dev-api:
    cargo run -p intrada-api

# Start only the web dev server
dev-web:
    trunk serve --config crates/intrada-web/Trunk.toml

# Run all tests
test:
    cargo test --workspace

# Run clippy
lint:
    cargo clippy --workspace

# Format code
fmt:
    cargo fmt --all

# Check everything (test + lint + format check)
check:
    cargo test --workspace
    cargo clippy --workspace
    cargo fmt --all -- --check

# Seed development data (API must be running)
seed:
    bash scripts/seed-dev-data.sh

# Build WASM for production or E2E testing
build:
    trunk build --config crates/intrada-web/Trunk.toml

# Run E2E tests (builds WASM first)
e2e: build
    cd e2e && npm install && npx playwright test --project=chromium
