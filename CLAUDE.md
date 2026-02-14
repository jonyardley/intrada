# intrada Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-02-08

## Active Technologies

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

- 001-music-library: Added Rust stable (1.75+, 2021 edition) + rusqlite (bundled), clap 4.5 (derive), ulid, serde, thiserror, anyhow, chrono

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
