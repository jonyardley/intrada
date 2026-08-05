# Intrada

A **practice coach** for musicians, built with [Crux](https://redbadger.github.io/crux/) for cross-platform Rust. The app decides what you practise, gates every block on evidence (tap-verdicts against countable criteria — tempo, key, scope), and tells you when you're done. Around the coaching loop: a music library, timed sessions, and progress tracking, organised as three pillars — **Plan** (decide what to practise), **Practice** (play with intention), and **Track** (see the process working).

See [`specs/intrada-practice-coach-design.md`](specs/intrada-practice-coach-design.md) for the coach vision and pedagogy, [`docs/roadmap.md`](docs/roadmap.md) for the phase plan, [`docs/status.md`](docs/status.md) for what's in flight right now, and [`VISION.md`](VISION.md) for the research foundation.

The only platform is the **native SwiftUI iOS app**, offline-first: on-device SQLite is the source of truth and the app works fully with no network and no account. A Leptos web shell and Tauri iOS host previously shared this core; both were removed (see [`docs/rebuild-review.md`](docs/rebuild-review.md)) in the pivot to the native-iOS-only practice coach.

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
- Xcode 26+, iOS 17.0+ target, [xcodegen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`), and the iOS Simulator runtime (Xcode → Settings → Platforms → iOS Simulator)

Git hooks (a pre-push check that refuses pushes onto an already-merged PR
branch) install automatically in Claude Code sessions; for other shells run
`bash scripts/install-git-hooks.sh` once (sets `core.hooksPath = .githooks`;
bypass a specific push with `SKIP_PR_CHECK=1 git push`, opt out entirely with
`git config --unset core.hooksPath`).

Forking this repo: update the development team in `ios/project.yml` to your
own Apple Team ID (developer.apple.com → Membership, or Xcode → Settings →
Accounts). `just ios`/`just ios-run` read `INTRADA_API_URL` and
`CLERK_PUBLISHABLE_KEY` from your shell or `.env` (`set dotenv-load` in the
justfile) — without them the build uses defaults and Clerk auth won't work.

## Quick start

The app is offline-first — you don't need the API running to use it.

```bash
# 1. Open the iOS app
just ios
# → regenerates Swift bindings if the core changed, then opens Xcode; Cmd+R

# 2. (Optional) run the API — only needed for auth/sync work
cp .env.example .env   # edit with your Turso credentials
just dev
# → API on :3001
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
just check        # fmt-check + clippy + tests + hygiene (mirrors CI)

# iOS (native SwiftUI)
just ios              # Regenerate bindings (if core changed) + open in Xcode
just ios-run          # Build + launch on a simulator + screenshot (seeds demo data)
SEED=0 just ios-run   # …launch against your real on-device data instead of demo data
just ios-test         # Build + run the snapshot/unit test suite (fast tier)
just ios-test-full    # …+ XCUITests (full gate; what ship/CI run before merge)

# Data
just seed         # Seed development data (API must be running)
```

## Project structure

```
crates/
  intrada-core/       # Pure Crux core (no I/O, no side effects) — incl. the coach engine
  intrada-ffi/        # UniFFI bridge — generates the Swift bindings
  intrada-api/         # REST API (Axum + Turso)
ios/                  # Native SwiftUI app (Intrada.xcodeproj via xcodegen)
content/              # Practice-coach authored content (read by the engine)
design/               # Claude Design system (intrada-design-system.dc.html)
docs/                 # Roadmap, status, and documentation
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
| [`specs/intrada-practice-coach-design.md`](specs/intrada-practice-coach-design.md) | The practice-coach vision, pedagogy model, and phased build plan |
| [`docs/status.md`](docs/status.md) | What's in flight right now — updated in every scope-changing PR |
| [`docs/design-principles.md`](docs/design-principles.md) | Interaction and design principles (how the app should feel) |
| [`docs/roadmap.md`](docs/roadmap.md) | Direction and the phase plan; issues/board carry scope and timing |
| [`VISION.md`](VISION.md) | Product vision |
| [`docs/research-foundation.md`](docs/research-foundation.md) | Research basis for design decisions |
| [`docs/rebuild-review.md`](docs/rebuild-review.md) | Rebuild-vs-pivot review against the practice-coach design |
| [`SETUP.md`](SETUP.md) | Deployment & configuration (Fly.io, Turso, TestFlight) |

## CI/CD

GitHub Actions runs on every push:

- **PR checks**: test, clippy, fmt, security & hygiene, native iOS build + snapshot tests, API Docker build
- **Push to main**: all checks + deploy the API (Fly.io) + native iOS release build (TestFlight lane runs separately on tag/dispatch)

## License

All rights reserved. See [`LICENSE`](LICENSE).
