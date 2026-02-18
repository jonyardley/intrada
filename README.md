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
- [Trunk](https://trunkrs.dev/) (`cargo install trunk`)
- [just](https://github.com/casey/just) (`brew install just` or `cargo install just`)

## Quick start

```bash
# 1. Set up environment
cp .env.example .env
# Edit .env with your Turso credentials

# 2. Start both API and web dev servers
just dev
# → API on :3001, web on :8080 (proxies /api/* to API)

# 3. Open http://localhost:8080
```

## Available commands

Run `just` to see all commands. Key ones:

```bash
just dev        # Start API + web dev servers concurrently
just dev-api    # Start only the API server
just dev-web    # Start only the Trunk dev server
just test       # Run all tests
just lint       # Run clippy
just fmt        # Format code
just check      # Run test + clippy + format check
just seed       # Seed development data (API must be running)
just build      # Build WASM for production/E2E
just e2e        # Build + run Playwright E2E tests
```

## Getting started (manual)

If you prefer not to use `just`, all commands should be run from the project root:

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
trunk serve --config crates/intrada-web/Trunk.toml
# → http://localhost:8080 (proxies /api/* to localhost:3001)
```

### API server (requires Turso credentials)

```bash
export TURSO_DATABASE_URL="libsql://intrada-<your-org>.turso.io"
export TURSO_AUTH_TOKEN="<your-token>"
export ALLOWED_ORIGIN="http://localhost:8080"
export PORT=3001

cargo run -p intrada-api
```

See [SETUP.md](SETUP.md) for full deployment and configuration instructions.

### Seed data (local development)

To populate the API with realistic sample data (8 pieces, 5 exercises, ~25 practice sessions spanning 35 days), run the seed script while the API server is running:

```bash
# Requires: curl, jq
just seed
# or: bash scripts/seed-dev-data.sh
```

This creates a realistic dataset with score progression, practice streaks, varied session lengths, and mixed completion statuses — useful for seeing how the library, session history, and analytics dashboard look with real data.

```bash
# Seed the live environment (Fly.io) — prompts for confirmation
bash scripts/seed-dev-data.sh --live

# Point at a different API server
API_URL=https://your-api.example.com bash scripts/seed-dev-data.sh

# Delete all existing data first, then re-seed
bash scripts/seed-dev-data.sh --clean
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
scripts/              # Development utilities
  seed-dev-data.sh    # Populate API with realistic sample data
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
