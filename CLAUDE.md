# intrada Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-02-08

## Active Technologies
- Rust stable (currently 1.89.0, project minimum 1.75+, 2021 edition) + GitHub Actions, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2` (002-ci-cd)
- Rust stable (1.75+, 2021 edition) — same as existing workspace + leptos 0.8.x (csr), crux_core 0.17.0-rc2 (workspace), tailwindcss v4 (standalone CLI), trunk 0.21.x, console_error_panic_hook, wasm-bindgen (003-leptos-app-mvp)
- N/A (stub data in-memory; no browser persistence) (003-leptos-app-mvp)
- Rust stable (1.75+, 2021 edition) + leptos 0.8.x (CSR), crux_core 0.17.0-rc2, tailwindcss v4 (standalone CLI), trunk 0.21.x, console_error_panic_hook, wasm-bindgen, ulid, chrono (004-library-detail-editing)
- N/A (in-memory stub data; no persistence) (004-library-detail-editing)
- Rust stable (1.75+, 2021 edition) + leptos 0.7 (CSR), crux_core 0.17.0-rc2 (workspace), send_wrapper 0.6, wasm-bindgen, console_error_panic_hook (005-component-architecture)
- N/A (stub data in-memory; no persistence changes) (005-component-architecture)
- Rust stable (1.75+, 2021 edition) + crux_core 0.17.0-rc2 (unchanged), leptos 0.7 → 0.8 (upgrade), send_wrapper 0.6 (unchanged) (007-crux-leptos-upgrade)
- N/A (in-memory stub data; no persistence changes) (007-crux-leptos-upgrade)
- Rust stable (1.75+, 2021 edition) + leptos 0.8.x (CSR), leptos_router 0.8.x, crux_core 0.17.0-rc2 (workspace), tailwindcss v4 (standalone CLI), trunk 0.21.x, console_error_panic_hook, wasm-bindgen, ulid, chrono, send_wrapper 0.6, getrandom 0.3 (008-url-routing)
- Rust stable (1.75+, 2021 edition) + leptos 0.7 (CSR), leptos_router 0.8, crux_core 0.17.0-rc2 (workspace), send_wrapper 0.6, tailwindcss v4 (standalone CLI), trunk 0.21.x (009-unified-library-form)

- Rust stable (1.75+, 2021 edition) + rusqlite (bundled), clap 4.5 (derive), ulid, serde, thiserror, anyhow, chrono (001-music-library)

## Project Structure

```text
crates/
  intrada-core/   # Pure Crux core (no I/O)
  intrada-cli/    # CLI shell (SQLite + terminal)
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
- 009-unified-library-form: Added Rust stable (1.75+, 2021 edition) + leptos 0.7 (CSR), leptos_router 0.8, crux_core 0.17.0-rc2 (workspace), send_wrapper 0.6, tailwindcss v4 (standalone CLI), trunk 0.21.x
- 008-url-routing: Added Rust stable (1.75+, 2021 edition) + leptos 0.8.x (CSR), leptos_router 0.8.x, crux_core 0.17.0-rc2 (workspace), tailwindcss v4 (standalone CLI), trunk 0.21.x, console_error_panic_hook, wasm-bindgen, ulid, chrono, send_wrapper 0.6, getrandom 0.3
- 007-crux-leptos-upgrade: Added Rust stable (1.75+, 2021 edition) + crux_core 0.17.0-rc2 (unchanged), leptos 0.7 → 0.8 (upgrade), send_wrapper 0.6 (unchanged)


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
