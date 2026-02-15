# Intrada

A music practice library manager built with [Crux](https://redbadger.github.io/crux/) for cross-platform Rust. Track the pieces and exercises you're working on, log practice sessions, tag items, and search your library.

Runs as a **web app** built with Leptos (CSR mode) and compiled to WASM. Data persists in the browser via localStorage.

## Prerequisites

- Rust stable (2021 edition)
- [Trunk](https://trunkrs.dev/) (for building and serving the web app)

## Getting started

All commands should be run from the project root (`intrada/`).

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy
```

### Web app

```bash
# Install trunk if you haven't already
cargo install trunk

# Serve the web app with hot reload
trunk serve --config crates/intrada-web/Trunk.toml
```

## Project structure

```
crates/
  intrada-core/       # Pure core library (no I/O)
    src/
      app.rs          # Crux app: Event -> update Model -> Command<Effect>
      model.rs        # Application state and ViewModel
      validation.rs   # Input validation rules and shared constants
      error.rs        # Error types
      domain/
        piece.rs      # Piece event handlers
        exercise.rs   # Exercise event handlers
        types.rs      # Domain types (Piece, Exercise, Tempo, etc.)
  intrada-web/        # Web shell (Leptos CSR + WASM)
    src/
      lib.rs          # Library crate root (shared modules)
      main.rs         # Binary entry point
      app.rs          # Leptos app root and routing
      core_bridge.rs  # Connects Crux effects to web shell
      data.rs         # Stub data for first-run seeding
      helpers.rs      # Form parsing utilities
      validation.rs   # Client-side validation (uses core constants)
      types.rs        # Shared types
      components/     # Reusable UI primitives
      views/          # Page-level views (list, detail, add, edit, sessions)
    tests/
      wasm.rs         # WASM integration tests (localStorage round-trips)
e2e/                  # Playwright E2E tests
specs/                # SpecKit design artifacts
```

## Architecture

Intrada follows the Crux pure-core pattern:

- **Core** (`intrada-core`) contains all business logic with zero side effects. Events go in, commands (effects) come out. Validation constants are exported so shells stay in sync.
- **Web shell** (`intrada-web`) runs the same core in the browser via WASM. Built with Leptos (CSR mode) and Tailwind CSS. Uses client-side routing, context-based state management, and a shared component library.

The core never touches a database, network, or DOM directly. This makes it testable in isolation and portable across shells without changing any business logic.

## Data storage

**Web:** Library data and practice sessions are stored in the browser's localStorage under the keys `intrada:library` and `intrada:sessions`. On first run, stub data is seeded automatically. All data is stored as compact JSON. IDs are ULIDs.

## License

TBD
