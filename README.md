# Intrada

An intentional practice companion for musicians, built with [Crux](https://redbadger.github.io/crux/) for cross-platform Rust. Organise your music library, run timed practice sessions with scoring, build reusable routines, and track your progress.

Intrada is organised around three activity pillars: **Plan** (decide what to practise), **Practice** (play with intention), and **Track** (see the process working). See [`docs/roadmap.md`](docs/roadmap.md) for the full roadmap and [`VISION.md`](VISION.md) for the research foundation.

The active platform is the **native SwiftUI iOS app**. A Leptos web shell and Tauri iOS host previously shared this core; both were removed (see [`docs/rebuild-review.md`](docs/rebuild-review.md)) as the product pivots to a practice-coach vision that is native-iOS-only.

## Architecture

```
┌──────────────────────┐    UniFFI/bincode    ┌──────────────────┐
│  SwiftUI (iOS)       │ ───────────────────→ │  intrada-core    │
│  Store: Event in,    │ ←─────────────────── │  (Crux, no I/O)  │
│  ViewModel out        │    Effects           └──────────────────┘
└──────────────────────┘                               │
        │  HTTPS/REST                                  │ GRDB (on-device)
        ▼                                               ▼
┌──────────────────┐    libsql    ┌──────────┐   ┌──────────────┐
│  Fly.io (Axum)   │ ───────────→ │  Turso   │   │  SQLite      │
│  intrada-api     │               │  (SQLite)│   │  (local-first)│
└──────────────────┘               └──────────┘   └──────────────┘
```

Intrada follows the **Crux pure-core pattern**: `intrada-core` contains all business logic with zero side effects. Events go in, effects come out. The native SwiftUI shell is a dumb pipe: it sends `Event`s, fulfils effects (HTTP via `URLSession`, persistence via GRDB), and renders the `ViewModel`. No domain logic lives in Swift.

- **Shell**: SwiftUI (iOS 17.0+), bindings generated via UniFFI + facet typegen
- **Core**: Crux 0.19 (Rust), zero I/O
- **API**: Axum 0.8 REST server on Fly.io
- **Database**: Turso (managed libsql/SQLite) via HTTP; on-device GRDB/SQLite for local-first
- **Auth**: Clerk (Google OAuth) in the browser flow, exchanged for a long-lived PAT on iOS; JWT RS256

## Prerequisites

- Rust stable (2021 edition, 1.75+)
- [just](https://github.com/casey/just) (`brew install just` or `cargo install just`)
- Xcode 26+, iOS 17.0+ target, [xcodegen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`)

## Quick start

```bash
# 1. Set up environment
cp .env.example .env
# Edit .env with your Turso credentials

# 2. Start the API dev server
just dev
# → API on :3001

# 3. Open the iOS app
just ios
# → regenerates Swift bindings if the core changed, then opens Xcode
```

## Available commands

Run `just` to see all commands. Key ones:

```bash
# Development
just dev          # Start the API dev server

# Quality
just test         # Run all tests
just lint         # Run clippy
just fmt          # Format code
just check        # Run test + clippy + format check

# iOS (native SwiftUI)
just ios              # Regenerate bindings (if core changed) + open in Xcode
just ios-run          # Build + launch on a simulator + screenshot (seeds demo data)
SEED=0 just ios-run   # …launch against your real on-device data instead of demo data
just ios-test         # Build + run the snapshot/unit test suite

# Data
just seed         # Seed development data (API must be running)
```

## Project structure

```
crates/
  intrada-core/       # Pure Crux core (no I/O, no side effects)
  intrada-ffi/        # UniFFI bridge — generates the Swift bindings
  intrada-api/         # REST API (Axum + Turso)
ios/                  # Native SwiftUI app (Intrada.xcodeproj via xcodegen)
design/               # Claude Design system (intrada-design-system.dc.html)
docs/                 # Product roadmap and documentation
scripts/              # Development utilities (seed data, simulator helpers)
specs/                # Design specs for major features
```

## Data storage

- **On-device (GRDB/SQLite)**: local-first source of truth for items and sessions — the app works fully offline. `updated_at` + soft-delete tombstones on every table.
- **API server (Turso)**: sync target and MCP server backing store; migrations run automatically on server startup.
- **iOS UserDefaults**: crash-recovery of an in-progress session, and the persisted library sort order.
- **IDs**: client-minted ULIDs.

## Documentation

| Document | Purpose |
|----------|---------|
| [`CLAUDE.md`](CLAUDE.md) | Development guidelines, architecture, design system rules |
| [`docs/development-workflow.md`](docs/development-workflow.md) | End-to-end feature workflow |
| [`docs/roadmap.md`](docs/roadmap.md) | Product roadmap (Plan/Practice/Track pillars) — **single source of truth** |
| [`VISION.md`](VISION.md) | Product vision |
| [`docs/research-foundation.md`](docs/research-foundation.md) | Research basis for design decisions |
| [`docs/rebuild-review.md`](docs/rebuild-review.md) | Rebuild-vs-pivot review against the practice-coach design |
| [`SETUP.md`](SETUP.md) | Deployment & configuration (Fly.io, Turso, TestFlight) |

## CI/CD

GitHub Actions runs on every push:

- **PR checks**: test, clippy, fmt, security & hygiene, native iOS build + snapshot tests, API Docker build
- **Push to main**: all checks + deploy the API (Fly.io) + native iOS release build (TestFlight lane runs separately on tag/dispatch)

## License

TBD
