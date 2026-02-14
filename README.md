# Intrada

A music practice library manager built with [Crux](https://redbadger.github.io/crux/) for cross-platform Rust. Track the pieces and exercises you're working on, tag them, and search your library.

Ships as a **CLI** (with SQLite storage) and a **web app** (Leptos/WASM, in-memory stub data). An iOS shell is planned.

## Prerequisites

- Rust stable (2021 edition)
- [Trunk](https://trunkrs.dev/) (for the web app only)

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

## CLI usage

When running from source, prefix all commands with `cargo run --bin intrada --`.

```bash
# Add a piece (title is positional, composer is a required flag)
cargo run --bin intrada -- add piece "Clair de Lune" --composer "Debussy" \
  --key "Db Major" --tempo-marking "Andante" --tempo-bpm 72 \
  --notes "Work on dynamics" --tag romantic --tag piano

# Add an exercise (title is positional, everything else is optional)
cargo run --bin intrada -- add exercise "Hanon No. 1" --composer "Hanon" \
  --category "Technique" --tempo-bpm 120 --tag warmup

# List everything
cargo run --bin intrada -- list

# Filter by type, key, category, or tag
cargo run --bin intrada -- list --type piece
cargo run --bin intrada -- list --tag baroque

# Show details of an item (use the ID from list output)
cargo run --bin intrada -- show <ID>

# Edit an item
cargo run --bin intrada -- edit <ID> --title "Clair de Lune (Suite bergamasque)" --key "Db Major"

# Delete (will prompt for confirmation, use -y to skip)
cargo run --bin intrada -- delete <ID>

# Tag / untag
cargo run --bin intrada -- tag <ID> baroque "sight reading"
cargo run --bin intrada -- untag <ID> baroque

# Search across title, composer, notes, category
cargo run --bin intrada -- search "Debussy"
```

Use `--help` on any command to see all available flags:

```bash
cargo run --bin intrada -- add piece --help
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
  intrada-cli/        # CLI shell (SQLite + terminal I/O)
    src/
      main.rs         # CLI argument parsing (clap)
      shell.rs        # Wires Core to storage and display
      storage.rs      # SQLite persistence
      display.rs      # Terminal output formatting
  intrada-web/        # Web shell (Leptos CSR + WASM)
    src/
      app.rs          # Leptos app root and routing
      core_bridge.rs  # Connects Crux effects to web shell
      data.rs         # Stub data for development
      helpers.rs      # Form parsing utilities
      validation.rs   # Client-side validation (uses core constants)
      components/     # Reusable UI primitives (14 components)
      views/          # Page-level views (list, detail, add, edit)
specs/                # SpecKit design artifacts
```

## Architecture

Intrada follows the Crux pure-core pattern:

- **Core** (`intrada-core`) contains all business logic with zero side effects. Events go in, commands (effects) come out. Validation constants are exported so shells stay in sync.
- **CLI shell** (`intrada-cli`) handles SQLite storage and terminal I/O. It feeds events to the core, executes the returned effects, and renders the view.
- **Web shell** (`intrada-web`) runs the same core in the browser via WASM. Built with Leptos (CSR mode) and Tailwind CSS. Uses client-side routing, context-based state management, and a shared component library.

The core never touches a database, network, or DOM directly. This makes it testable in isolation and portable across shells without changing any business logic.

## Data storage

**CLI:** SQLite database at `~/.local/share/intrada/library.db`. Tags are stored as JSON arrays. IDs are ULIDs.

**Web:** In-memory stub data (no persistence yet).

## License

TBD
