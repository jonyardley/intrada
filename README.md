# Intrada

An intentional practice companion for musicians, built with [Crux](https://redbadger.github.io/crux/) for cross-platform Rust. Organise your music library, run timed practice sessions with scoring, build reusable routines, and track your progress.

Intrada is organised around three activity pillars: **Plan** (decide what to practise), **Practice** (play with intention), and **Track** (see the process working). See [`docs/roadmap.md`](docs/roadmap.md) for the full roadmap and [`VISION.md`](VISION.md) for the research foundation.

## Architecture

```
┌─────────────────────────┐                    ┌──────────────────┐               ┌──────────┐
│  Cloudflare Workers     │     HTTPS/REST     │  Fly.io (Axum)   │    libsql     │  Turso   │
│  Leptos WASM (web app)  │ ─────────────────→ │  intrada-api     │ ────────────→ │  (SQLite) │
└─────────────────────────┘                    └──────────────────┘               └──────────┘
            ▲                                          ▲
            │ same WASM bundle                         │
┌───────────┴─────────────┐    HTTPS/REST              │
│  Tauri 2 (iOS)          │ ───────────────────────────┘
│  WKWebView host         │
└─────────────────────────┘
```

Intrada follows the **Crux pure-core pattern**: `intrada-core` contains all business logic with zero side effects. Events go in, commands (effects) come out. The Leptos shell is the single UI codebase — it ships as the web app on Cloudflare Workers and as the iOS app inside a Tauri 2 WKWebView host.

- **UI shell**: Leptos 0.8 CSR + WASM
- **Web host**: Cloudflare Workers (static asset serving)
- **iOS host**: Tauri 2 (WKWebView, native plugins for haptics + deep links)
- **API**: Axum 0.8 REST server on Fly.io
- **Database**: Turso (managed libsql/SQLite) via HTTP
- **Auth**: Clerk (Google OAuth), JWT RS256

## Prerequisites

- Rust stable (2021 edition, 1.75+)
- [Trunk](https://trunkrs.dev/) (`cargo install trunk`)
- [just](https://github.com/casey/just) (`brew install just` or `cargo install just`)
- For iOS: Xcode 16+, iOS 17.0+ target, Tauri CLI (`cargo install tauri-cli --version "^2" --locked`), CocoaPods (`brew install cocoapods`)

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

# iOS (Tauri)
just ios-dev          # Run on simulator (trunk serve + tauri ios dev)
just ios-dev-device   # Run on a connected physical device
just ios-build        # Build for device (Xcode sideload, no TestFlight)

# Data
just seed         # Seed development data (API must be running)
```

## Project structure

```
crates/
  intrada-core/       # Pure Crux core (no I/O, no side effects)
  intrada-web/        # Leptos CSR + WASM — the UI shell (web + iOS)
  intrada-api/        # REST API (Axum + Turso)
  intrada-mobile/     # Tauri 2 iOS host — wraps intrada-web in WKWebView
    src-tauri/        #   Rust host, tauri.conf.json, Swift plugins
design/               # Pencil design system (intrada.pen)
docs/                 # Product roadmap and documentation
e2e/                  # Playwright E2E tests
scripts/              # Development utilities (seed data)
specs/                # SpecKit design artifacts
```

## Data storage

- **API server**: Items (pieces and exercises), sessions, and routines are stored in Turso (managed SQLite) via the REST API. Migrations run automatically on server startup.
- **Browser / WKWebView (localStorage)**: Only used for crash recovery of in-progress sessions (`intrada:session-in-progress` key). All other data flows through the API.
- **IDs**: ULIDs generated server-side.

## Documentation

| Document | Purpose |
|----------|---------|
| [`CLAUDE.md`](CLAUDE.md) | Development guidelines, architecture, design system rules |
| [`docs/development-workflow.md`](docs/development-workflow.md) | End-to-end feature workflow |
| [`docs/roadmap.md`](docs/roadmap.md) | Product roadmap (Plan/Practice/Track pillars) — **single source of truth** |
| [`VISION.md`](VISION.md) | Product vision |
| [`docs/research-foundation.md`](docs/research-foundation.md) | Research basis for design decisions |
| [`SETUP.md`](SETUP.md) | Deployment & configuration (Cloudflare, Fly.io, Turso, CI/CD) |

## CI/CD

GitHub Actions runs on every push:

- **PR checks**: test, clippy, fmt, WASM build, WASM tests, E2E tests (Playwright)
- **Push to main**: all checks + deploy frontend (Cloudflare Workers) + deploy API (Fly.io)

## License

TBD
