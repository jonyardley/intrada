# intrada Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-02-08

## Active Technologies
- Rust stable (currently 1.89.0, project minimum 1.75+, 2021 edition) + GitHub Actions, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2` (002-ci-cd)
- Rust stable (1.75+, 2021 edition) — same as existing workspace + leptos 0.8.x (csr), crux_core 0.17.0-rc2 (workspace), tailwindcss v4 (standalone CLI), trunk 0.21.x, console_error_panic_hook, wasm-bindgen (003-leptos-app-mvp)
- N/A (stub data in-memory; no browser persistence) (003-leptos-app-mvp)

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
- 003-leptos-app-mvp: Added Rust stable (1.75+, 2021 edition) — same as existing workspace + leptos 0.8.x (csr), crux_core 0.17.0-rc2 (workspace), tailwindcss v4 (standalone CLI), trunk 0.21.x, console_error_panic_hook, wasm-bindgen
- 002-ci-cd: Added Rust stable (currently 1.89.0, project minimum 1.75+, 2021 edition) + GitHub Actions, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`

- 001-music-library: Added Rust stable (1.75+, 2021 edition) + rusqlite (bundled), clap 4.5 (derive), ulid, serde, thiserror, anyhow, chrono

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
