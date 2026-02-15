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

## Project Structure

```text
crates/
  intrada-core/   # Pure Crux core (no I/O)
  intrada-web/    # Web shell (Leptos CSR + WASM)
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
- 015-rework-sessions: Added Rust stable (1.75+, 2021 edition) + crux_core 0.17.0-rc2 (workspace), leptos 0.8.x (CSR), leptos_router 0.8.x, web-sys (Storage+Window), wasm-bindgen, serde/serde_json 1, ulid 1, chrono 0.4, send_wrapper 0.6
- 015-rework-sessions: Added [if applicable, e.g., PostgreSQL, CoreData, files or N/A]
- 014-remove-cli: Removed CLI shell crate and its workspace dependencies (`clap`, `anyhow`, `dirs`). Project now focuses on core + web only.


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
