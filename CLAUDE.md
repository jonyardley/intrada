# intrada Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-02-15

## Active Technologies
- Rust stable (currently 1.89.0, project minimum 1.75+, 2021 edition) + GitHub Actions, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2` (002-ci-cd)
- Rust stable (1.75+, 2021 edition) + leptos 0.8.x (CSR), leptos_router 0.8.x, crux_core 0.17.0-rc2 (workspace), tailwindcss v4 (standalone CLI), trunk 0.21.x, console_error_panic_hook, wasm-bindgen, ulid, chrono, send_wrapper 0.6, getrandom 0.3 (008-url-routing)
- localStorage (web: `intrada:library` key, `intrada:sessions` key) (011-json-persistence, 012-practice-sessions)
- Rust stable (1.75+, 2021 edition) + TypeScript (Playwright E2E tests) + `wasm-bindgen-test` 0.3 (WASM tests), Playwright (E2E tests), existing workspace deps (crux_core, leptos, web-sys) (013-web-testing)
- Workspace dependencies: crux_core 0.17.0-rc2, serde 1, serde_json 1, ulid 1, chrono 0.4, thiserror 1 (014-remove-cli)
- Rust stable (1.75+, 2021 edition) + crux_core 0.17.0-rc2 (workspace), leptos 0.8.x (CSR), leptos_router 0.8.x, web-sys (Storage+Window), wasm-bindgen, serde/serde_json 1, ulid 1, chrono 0.4, send_wrapper 0.6 (015-rework-sessions)
- localStorage (web: `intrada:sessions` key for completed sessions, `intrada:session-in-progress` key for crash recovery) (015-rework-sessions)
- Rust stable (1.75+, 2021 edition) + leptos 0.8.x (CSR), crux_core 0.17.0-rc2, Tailwind CSS v4 (standalone CLI v4.1.18), trunk 0.21.x (016-glassmorphism-responsive)
- N/A (no storage changes — visual only) (016-glassmorphism-responsive)
- GitHub Actions YAML (no Rust code changes) + GitHub Actions (checkout@v4, upload-artifact@v4, download-artifact@v4, rust-toolchain, rust-cache@v2, trunk-action, wrangler-action@v3) (019-improve-cicd)
- Rust stable (1.89.0 in CI; workspace MSRV 1.75, axum 0.8 requires 1.78+) + axum 0.8, libsql 0.9 (remote feature), libsql_migration 0.2.2 (content feature), tower-http 0.6 (cors), tokio 1, serde/serde_json 1, ulid 1, chrono 0.4, thiserror 1, tracing 0.1 (020-api-server)
- Turso (managed libsql/SQLite) via HTTP protocol — `libsql` crate with `remote` feature (020-api-server)
- Rust stable (1.89.0 in CI; workspace MSRV 1.75+, 2021 edition) + leptos 0.8.x (CSR), crux_core 0.17.0-rc2, gloo-net 0.6 (NEW), wasm-bindgen-futures 0.4 (NEW), serde/serde_json 1 (021-api-sync)
- REST API (Turso/libsql via intrada-api) for pieces, exercises, sessions; localStorage for session-in-progress crash recovery only (021-api-sync)

## Project Structure

```text
crates/
  intrada-core/   # Pure Crux core (no I/O)
  intrada-web/    # Web shell (Leptos CSR + WASM)
  intrada-api/    # REST API server (Axum + Turso)
e2e/              # Playwright E2E tests
specs/            # SpecKit design artifacts
```

## Commands

```bash
cargo test
cargo clippy
```

## Code Style

Rust stable (1.75+, 2021 edition): Follow standard conventions

## Recent Changes
- 021-api-sync: Added Rust stable (1.89.0 in CI; workspace MSRV 1.75+, 2021 edition) + leptos 0.8.x (CSR), crux_core 0.17.0-rc2, gloo-net 0.6 (NEW), wasm-bindgen-futures 0.4 (NEW), serde/serde_json 1
- 020-api-server: Added Rust stable (1.89.0 in CI; workspace MSRV 1.75, axum 0.8 requires 1.78+) + axum 0.8, libsql 0.9 (remote feature), libsql_migration 0.2.2 (content feature), tower-http 0.6 (cors), tokio 1, serde/serde_json 1, ulid 1, chrono 0.4, thiserror 1, tracing 0.1
- 019-improve-cicd: Added GitHub Actions YAML (no Rust code changes) + GitHub Actions (checkout@v4, upload-artifact@v4, download-artifact@v4, rust-toolchain, rust-cache@v2, trunk-action, wrangler-action@v3)


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
