# Intrada

A music practice library manager built with [Crux](https://redbadger.github.io/crux/) for cross-platform Rust. Track the pieces and exercises you're working on, log practice sessions, tag items, and search your library.

## Architecture

```
┌─────────────────────┐     HTTPS      ┌──────────────────┐     libsql     ┌──────────┐
│  Cloudflare Workers │ ─────────────→ │  Fly.io (Axum)   │ ────────────→ │  Turso   │
│  (static WASM app)  │   REST API     │  intrada-api     │               │  (SQLite) │
└─────────────────────┘                └──────────────────┘               └──────────┘
```

- **Frontend**: Leptos CSR + WASM, deployed as static files to Cloudflare Workers
- **API**: Axum 0.8 REST server, deployed to Fly.io via Docker
- **Database**: Turso (managed libsql/SQLite), accessed via HTTP
- **Core**: Pure Rust business logic shared across shells via Crux

Intrada follows the Crux pure-core pattern: the core (`intrada-core`) contains all business logic with zero side effects. Events go in, commands (effects) come out. The web shell (`intrada-web`) runs the core in the browser via WASM, communicating with the API server for data persistence. The core never touches a database, network, or DOM directly — making it testable in isolation and portable across shells.

## Prerequisites

- Rust stable (2021 edition, 1.75+)
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

### Web app (local development)

```bash
# Install trunk if you haven't already
cargo install trunk

# Serve the web app with hot reload
trunk serve --config crates/intrada-web/Trunk.toml
```

### API server (requires Turso credentials)

```bash
export TURSO_DATABASE_URL="libsql://intrada-<your-org>.turso.io"
export TURSO_AUTH_TOKEN="<your-token>"
export ALLOWED_ORIGIN="http://localhost:8080"

cargo run -p intrada-api
```

See [SETUP.md](SETUP.md) for full deployment and configuration instructions.

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
      core_bridge.rs  # Connects Crux effects to web APIs (HTTP + localStorage)
      api_client.rs   # Async HTTP client (gloo-net) for REST API
      helpers.rs      # Form parsing utilities
      validation.rs   # Client-side validation (uses core constants)
      types.rs        # Shared types (SharedCore, IsLoading, etc.)
      components/     # Reusable UI primitives
      views/          # Page-level views (list, detail, add, edit, sessions)
    tests/
      wasm.rs         # WASM integration tests
  intrada-api/        # REST API server (Axum + Turso)
    src/
      main.rs         # Server entry point (Axum + CORS + migrations)
      routes/         # HTTP handlers (pieces, exercises, sessions, health)
      db/             # Database layer (libsql CRUD operations)
      migrations.rs   # Auto-run schema setup
      error.rs        # API error types → HTTP responses
      state.rs        # Shared app state (Arc<Database>)
e2e/                  # Playwright E2E tests
  fixtures/           # API mocking (page.route interception)
  tests/              # Test specs (smoke, add-item, detail, navigation, sessions)
specs/                # SpecKit design artifacts
```

## Data storage

- **API server**: Pieces, exercises, and completed sessions are stored in Turso (managed SQLite) via the REST API. Migrations run automatically on server startup.
- **Browser (localStorage)**: Only used for crash recovery of in-progress sessions (`intrada:session-in-progress` key). All other data flows through the API.
- **IDs**: ULIDs generated server-side.

## CI/CD

GitHub Actions runs on every push:

- **PR checks**: test, clippy, fmt, WASM build, WASM tests, E2E tests (Playwright)
- **Push to main**: all checks + deploy frontend (Cloudflare Workers) + deploy API (Fly.io)

## License

TBD
