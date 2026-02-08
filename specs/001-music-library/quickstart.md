# Quickstart: Music Library

**Feature Branch**: `001-music-library`
**Date**: 2026-02-08

## Prerequisites

- Rust toolchain (rustup) — stable channel, 1.88+
- No external dependencies (SQLite is bundled via rusqlite)

## Setup

```bash
# Clone and checkout feature branch
git clone <repo-url> intrada
cd intrada
git checkout 001-music-library

# Build all crates
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -p intrada-cli -- --help
```

## Project Layout

```
intrada/
├── Cargo.toml              # Virtual workspace manifest
├── crates/
│   ├── intrada-core/       # Crux App — pure business logic (no I/O)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs       # Public API re-exports
│   │       ├── app.rs       # Intrada App impl, Event/Effect enums
│   │       ├── model.rs     # Model + ViewModel
│   │       ├── domain/
│   │       │   ├── mod.rs
│   │       │   ├── piece.rs     # Piece type + PieceEvent handler
│   │       │   ├── exercise.rs  # Exercise type + ExerciseEvent handler
│   │       │   └── types.rs     # Shared types (Tempo, LibraryItem, etc.)
│   │       ├── validation.rs    # Pure validation functions
│   │       └── error.rs         # LibraryError (Validation, NotFound)
│   └── intrada-cli/         # CLI shell (handles I/O + SQLite)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs      # Entry point, clap setup
│           ├── shell.rs     # Crux shell: processes Effects
│           ├── storage.rs   # SQLite persistence
│           └── display.rs   # Terminal output formatting
└── specs/                   # Feature specifications (this directory)
```

## Key Dependencies

| Crate | Version | Used in | Purpose |
|-------|---------|---------|---------|
| `crux_core` | 0.16.2 | intrada-core | Cross-platform app framework |
| `facet` | 0.28 | intrada-core | FFI type generation |
| `ulid` | latest | intrada-core | ID generation |
| `serde` | latest | intrada-core | Serialization |
| `thiserror` | latest | intrada-core | Typed error definitions |
| `chrono` | latest | intrada-core | Timestamp handling |
| `rusqlite` | latest | intrada-cli | SQLite storage (bundled) |
| `clap` | 4.5 | intrada-cli | CLI argument parsing (derive) |
| `anyhow` | latest | intrada-cli | Application error handling |
| `dirs` | 5 | intrada-cli | XDG data directory |

## Quick Usage

```bash
# Add a piece
cargo run -p intrada-cli -- add piece "Clair de Lune" --composer "Debussy" --key "Db Major" --tempo-marking "Andante"

# Add an exercise
cargo run -p intrada-cli -- add exercise "C Major Scale" --category "Scales" --key "C Major"

# List everything
cargo run -p intrada-cli -- list

# Search
cargo run -p intrada-cli -- search "debussy"

# Filter by type and tag
cargo run -p intrada-cli -- list --type exercise --tag "warm-up"

# View details
cargo run -p intrada-cli -- show <id>

# Edit
cargo run -p intrada-cli -- edit <id> --title "Updated Title"

# Tag
cargo run -p intrada-cli -- tag <id> "exam prep" "romantic era"

# Delete
cargo run -p intrada-cli -- delete <id>
```

## Testing

```bash
# Run all tests
cargo test

# Run only core tests (pure, no I/O — fast)
cargo test -p intrada-core

# Run only CLI tests (uses in-memory SQLite)
cargo test -p intrada-cli

# Run with output
cargo test -- --nocapture
```

## Architecture Notes

- **intrada-core** is a pure Crux App with no I/O. It processes Events, updates Model, and returns Commands with Effects.
- **intrada-cli** is the shell — it parses CLI arguments, sends Events to the core, processes Effects (SQLite storage, terminal output).
- The core is side-effect free: all tests are pure unit tests with no mocking needed.
- The SQLite database is stored at `~/.local/share/intrada/library.db` (XDG Base Directory).
- The same core crate can be used by future iOS (via UniFFI) and web (via wasm-bindgen) shells with zero changes.
