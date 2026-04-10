# Intrada

An intentional practice companion for musicians, built with [Crux](https://redbadger.github.io/crux/) for cross-platform Rust. Organise your music library, run timed practice sessions with scoring, build reusable routines, and track your progress.

Intrada is organised around three activity pillars: **Plan** (decide what to practise), **Practice** (play with intention), and **Track** (see the process working). See [`docs/roadmap.md`](docs/roadmap.md) for the full roadmap and [`VISION.md`](VISION.md) for the research foundation.

## Architecture

```
┌─────────────────────┐                    ┌──────────────────┐               ┌──────────┐
│  Cloudflare Workers │     HTTPS/REST     │  Fly.io (Axum)   │    libsql     │  Turso   │
│  (Leptos WASM app)  │ ─────────────────→ │  intrada-api     │ ────────────→ │  (SQLite) │
└─────────────────────┘                    └──────────────────┘               └──────────┘
                                                    ↑
┌─────────────────────┐     HTTPS/REST              │
│  iOS app (SwiftUI)  │ ───────────────────────────-┘
│  UniFFI + BCS       │
└─────────────────────┘
```

Intrada follows the **Crux pure-core pattern**: `intrada-core` contains all business logic with zero side effects. Events go in, commands (effects) come out. Platform shells (web + iOS) are dumb I/O pipes — they execute HTTP requests and render the ViewModel without understanding domain types.

- **Web shell**: Leptos CSR + WASM, deployed to Cloudflare Workers
- **iOS shell**: SwiftUI, using UniFFI with BCS serialization for the Crux bridge
- **API**: Axum 0.8 REST server on Fly.io
- **Database**: Turso (managed libsql/SQLite) via HTTP
- **Auth**: Clerk (Google OAuth), JWT RS256

## Prerequisites

- Rust stable (2021 edition, 1.75+)
- [Trunk](https://trunkrs.dev/) (`cargo install trunk`)
- [just](https://github.com/casey/just) (`brew install just` or `cargo install just`)
- For iOS: Xcode 16+, Swift 6.0, iOS 17.0+ target

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
# Development
just dev          # Start API + web dev servers concurrently
just dev-api      # Start only the API server
just dev-web      # Start only the Trunk dev server

# Quality
just test         # Run all tests
just lint         # Run clippy
just fmt          # Format code
just check        # Run test + clippy + format check

# Web
just build        # Build WASM for production/E2E
just e2e          # Build + run Playwright E2E tests

# iOS
just ios          # Cross-compile for iOS + generate types + UniFFI bindings
just ios-sim      # Build for iOS Simulator
just ios-run      # Build for sim + launch
just ios-smoke-test  # Build + launch, verify no crash

# Type generation
just typegen      # Regenerate Swift types from Rust domain types
just typegen-check  # Verify generated types are up to date (CI)

# Data
just seed         # Seed development data (API must be running)
```

## Project structure

```
crates/
  intrada-core/       # Pure Crux core (no I/O, no side effects)
  intrada-web/        # Web shell (Leptos CSR + WASM)
  intrada-api/        # REST API (Axum + Turso)
  shared/             # UniFFI bindings + CoreFfi/CoreJson bridges
  shared_types/       # Facet typegen — auto-generates Swift types with BCS
design/               # Pencil design system (intrada.pen)
docs/                 # Product roadmap and documentation
e2e/                  # Playwright E2E tests
ios/Intrada/          # iOS app (SwiftUI shell)
  Core/               # Crux bridge (effect processor)
  DesignSystem/       # Tokens (Color, Font, Spacing) + ViewModifiers
  Components/         # Reusable SwiftUI components
  Views/              # Feature screens
  Generated/          # Auto-generated types (SharedTypes, Serde, UniFFI)
scripts/              # Development utilities (seed data, build helpers)
specs/                # SpecKit design artifacts
```

## Data storage

- **API server**: Items (pieces and exercises), sessions, and routines are stored in Turso (managed SQLite) via the REST API. Migrations run automatically on server startup.
- **Browser (localStorage)**: Only used for crash recovery of in-progress sessions (`intrada:session-in-progress` key). All other data flows through the API.
- **iOS (UserDefaults)**: Same crash recovery pattern as web, single key only.
- **IDs**: ULIDs generated server-side.

## Documentation

| Document | Purpose |
|----------|---------|
| [`CLAUDE.md`](CLAUDE.md) | Development guidelines, architecture, design system rules |
| [`docs/development-workflow.md`](docs/development-workflow.md) | End-to-end feature workflow, SpecKit stages, AI subagents |
| [`docs/roadmap.md`](docs/roadmap.md) | Product roadmap (Plan/Practice/Track pillars) — **single source of truth** |
| [`VISION.md`](VISION.md) | Product vision |
| [`docs/research-foundation.md`](docs/research-foundation.md) | Research basis for design decisions |
| [`SETUP.md`](SETUP.md) | Deployment & configuration (Cloudflare, Fly.io, Turso, CI/CD) |

## CI/CD

GitHub Actions runs on every push:

- **PR checks**: test, clippy, fmt, WASM build, WASM tests, typegen freshness, E2E tests (Playwright)
- **Push to main**: all checks + deploy frontend (Cloudflare Workers) + deploy API (Fly.io)

## License

TBD
