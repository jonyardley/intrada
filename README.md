# Intrada

A music practice library manager built with [Crux](https://redbadger.github.io/crux/) for cross-platform Rust. Track the pieces and exercises you're working on, tag them, and search your library.

Currently ships as a CLI. iOS and web shells are planned.

## Prerequisites

- Rust stable (2021 edition)

## Getting started

All commands should be run from the project root (`intrada/`).

```bash
# Build
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy
```

## Usage

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
  intrada-core/     # Pure core library (no I/O)
    src/
      app.rs        # Crux app: Event → update Model → Command<Effect>
      model.rs      # Application state
      validation.rs # Input validation rules
      error.rs      # Error types
      domain/
        piece.rs    # Piece event handlers
        exercise.rs # Exercise event handlers
        types.rs    # Domain types (Piece, Exercise, Tempo, etc.)
  intrada-cli/      # CLI shell (handles I/O)
    src/
      main.rs       # CLI argument parsing (clap)
      shell.rs      # Wires Core to storage and display
      storage.rs    # SQLite persistence
      display.rs    # Terminal output formatting
specs/              # SpecKit design artifacts
```

## Architecture

Intrada follows the Crux pure-core pattern:

- **Core** (`intrada-core`) contains all business logic with zero side effects. Events go in, commands (effects) come out.
- **Shell** (`intrada-cli`) handles the real world — SQLite storage and terminal I/O. It feeds events to the core, executes the returned effects, and renders the view.

The core never touches the database or terminal directly. This makes it testable in isolation and portable to other shells (iOS, web) without changing any business logic.

## Data storage

The SQLite database is stored at `~/.local/share/intrada/library.db`. Tags are stored as JSON arrays. IDs are ULIDs.

## License

TBD
