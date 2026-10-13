# Intrada

A **practice notebook** for musicians, built with [Crux](https://redbadger.github.io/crux/) for cross-platform Rust. Build a session from your music library, group and reorder what you'll practise, play it through with a timer and rep counting, and score how it went. Organised as three pillars: **Plan** (decide what to practise), **Practice** (play with intention), and **Track** (see the process working).

See [`docs/roadmap.md`](docs/roadmap.md) for direction, `just status` for what's in flight right now, and [`VISION.md`](VISION.md) for the research foundation. (A practice-coach direction was built July to August 2026 and reversed; its design record lives in [`specs/intrada-practice-coach-design.md`](specs/intrada-practice-coach-design.md), bannered.)

intrada ships **two native apps on one Rust core**: SwiftUI for iPhone in `ios/` and Compose for Android in `android/`, kept at parity, so every new feature lands on both. Both are offline-first: on-device SQLite is the source of truth and each app works fully with no network and no account. A Leptos web shell and Tauri iOS host previously shared this core; both were removed in 2026-07 (see [`docs/rebuild-review.md`](docs/rebuild-review.md)).

## Architecture

```
┌──────────────────────┐    UniFFI/bincode    ┌──────────────────┐
│  SwiftUI (iOS)       │ ───────────────────→ │  intrada-core    │
│  Store: Event in,    │ ←─────────────────── │  (Crux, no I/O)  │
│  ViewModel out        │    Effects           └──────────────────┘
└──────────────────────┘                               │
                                                       │ intrada-store
                                                       ▼
                                                ┌──────────────┐
                                                │  SQLite      │
                                                │  (local-first)│
                                                └──────────────┘
```

Intrada follows the **Crux pure-core pattern**: `intrada-core` contains all business logic with zero side effects. Events go in, effects come out. Each native shell is a dumb pipe: it sends `Event`s, fulfils persistence effects through the shared Rust store (`intrada-store`), and renders the `ViewModel`. No domain logic lives in Swift or Kotlin.

- **Shells**: SwiftUI (iOS 17.0+) and Compose (Android 9, API 28+), bindings generated via UniFFI and facet typegen
- **Core**: Crux 0.20 (Rust), zero I/O
- **Database**: on-device SQLite through `intrada-store`, shared by both phones, local-first

## Prerequisites

Full detail, including the optional build cache, is in [`SETUP.md`](SETUP.md#4-local-development).

**Everyone**

- Rust via [rustup](https://rustup.rs); the pinned toolchain is in
  `rust-toolchain.toml`. Do not also install Rust from Homebrew:
  `/opt/homebrew/bin` sits ahead of `~/.cargo/bin` on a default PATH, so the brew
  `cargo` shadows rustup, ignores the pin, and has no iOS or Android targets
- [mise](https://mise.jdx.dev), then `mise install` in the repo: it provisions
  everything pinned in `mise.toml` (just, xcodegen, cargo-nextest, the hygiene
  tools, Ruby for fastlane, and cargo-swift at exactly 0.9.0, whose bundled
  uniffi-bindgen matches our uniffi 0.29.4 runtime)
- [gh](https://cli.github.com), signed in with `gh auth login`, and `jq`:
  claiming an issue uses both, and opening a PR uses gh
- [worktrunk](https://worktrunk.dev) (`brew install worktrunk`),
  which `just worktree-new` wraps; its machine-local config is in
  [`docs/worktrees.md`](docs/worktrees.md)
- Optional: `cargo-deny` (`cargo install cargo-deny`) and `gitleaks`
  (`brew install gitleaks`). `just check` skips either when it is missing and
  says so; CI always runs both

**iPhone**

- Xcode 26+ and the iOS Simulator runtime, in Xcode's Settings under Platforms
- The iOS Rust targets:
  `rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios`

**Android**

- [Android Studio](https://developer.android.com/studio): the recipes take its
  bundled JDK and the SDK at `~/Library/Android/sdk` when `JAVA_HOME` and
  `ANDROID_HOME` are unset
- From the SDK Manager: Command-line Tools (latest), the Emulator, Platform
  Tools, the SDK Platform and Build Tools CI installs (the `android` job in
  `.github/workflows/ci.yml`), the NDK at the version pinned as
  `intrada.ndkVersion` in `android/gradle.properties`, and the Android 36
  Google APIs arm64 system image. `just android-run` creates the `intrada-api36` emulator from it the
  first time
- [cargo-ndk](https://github.com/bbqsrc/cargo-ndk), at the version CI pins:
  `cargo install cargo-ndk --version 4.1.2`
  and the Android Rust targets:
  `rustup target add aarch64-linux-android x86_64-linux-android`
- `just android-gen` once, to build the bindings every other Android recipe needs

**Agent tooling (optional, for Claude Code sessions)**

- The language servers, as Claude Code plugins from the official marketplace:
  `rust-analyzer-lsp`, `swift-lsp` and `kotlin-lsp`. The Swift plugin also
  needs `brew install xcode-build-server`, and the Kotlin plugin
  `brew install --cask kotlin-lsp`. Then run `just lsp-setup` once per
  machine, and once per worktree that wants Swift. What each needs and why is
  under code intelligence in
  [`docs/working-with-agents.md`](docs/working-with-agents.md)
- [graphify](https://github.com/Graphify-Labs/graphify) (`uv tool install graphifyy`,
  two y's), the knowledge graph kept in the main checkout; see
  [`docs/reference.md`](docs/reference.md#knowledge-graph-graphify)
- [rtk](https://www.rtk-ai.app) (`brew install rtk`, then `rtk init -g --hook-only --auto-patch`),
  a global hook that shortens command output before it reaches the agent.
  Machine tooling: nothing in the repo depends on it

Git hooks (a pre-push check that refuses pushes onto an already-merged PR
branch) install automatically in Claude Code sessions; for other shells run
`bash scripts/install-git-hooks.sh` once (sets `core.hooksPath = .githooks`;
bypass a specific push with `SKIP_PR_CHECK=1 git push`, opt out entirely with
`git config --unset core.hooksPath`).

Forking this repo: update the development team in `ios/project.yml` to your
own Apple Team ID (developer.apple.com → Membership, or Xcode → Settings →
Accounts).

## Quick start

The apps are offline-first: everything works with no network and no account.

```bash
just ios          # regenerates the Swift bindings if the core changed, then opens Xcode; Cmd+R
just android-run  # regenerates the Android bindings, boots the emulator, installs and screenshots
```

## Available commands

Run `just` to see all commands. Key ones:

```bash
# Quality
just test         # Run all tests
just lint         # Run clippy
just fmt          # Format code
just check        # fmt-check + clippy + tests + hygiene (mirrors CI)

# iOS (native SwiftUI)
just ios              # Regenerate bindings (if core changed) + open in Xcode
just ios-run          # Build + launch on a simulator + screenshot (seeds demo data)
SEED=0 just ios-run   # ...launch against your real on-device data instead of demo data
just ios-test         # Build + run the snapshot/unit test suite (fast tier)
just ios-test-full    # ...plus XCUITests (full tier; CI runs these on every PR)

# Android (native Compose)
just android-gen      # Regenerate the Kotlin types, bridge and native libraries
just android-run      # Build, launch on the intrada-api36 emulator, screenshot
just android-test     # JVM unit and Roborazzi snapshot tests, no emulator
just android-test-ui  # Tap-through UI tests on the emulator
just android-check    # CI's Android gates: ktfmt, warnings as errors, lint, detekt
```

## Project structure

```
crates/
  intrada-core/       # Pure Crux core (no I/O, no side effects)
  intrada-ffi/        # UniFFI bridge, generates the Swift and Kotlin bindings
  intrada-store/      # The on-device SQLite store both apps share
ios/                  # Native SwiftUI app (Intrada.xcodeproj via xcodegen)
android/              # Native Compose app (Gradle; bindings in android/generated)
design/               # Claude Design system (intrada-design-system.dc.html)
docs/                 # Roadmap, status, and documentation
scripts/              # Development utilities (simulator helpers)
specs/                # Design specs for major features
```

## Data storage

- **On-device (SQLite, `intrada-store`)**: local-first source of truth for items and sessions, so the app works fully offline. `updated_at` + soft-delete tombstones on every table.
- **iOS UserDefaults**: crash-recovery of an in-progress session, and the persisted library sort order.
- **IDs**: client-minted ULIDs.

## Documentation

| Document | Purpose |
|----------|---------|
| [`CLAUDE.md`](CLAUDE.md) | Development guidelines and architecture; path-scoped rules in `.claude/rules/`, workflow skills in `.claude/skills/` |
| [`docs/reference.md`](docs/reference.md) | The how and why behind CLAUDE.md: commands, environment variables, gotchas in full |
| [`docs/ios-testing.md`](docs/ios-testing.md) | Building, running, snapshot-testing and UI-testing the app on the simulator |
| [`docs/worktrees.md`](docs/worktrees.md) | One session per worktree: what claims a checkout and what that stops |
| [`specs/README.md`](specs/README.md) | Which specs are live and which are records of shipped behaviour |
| [`docs/where-we-are.md`](docs/where-we-are.md) | Which phase we are in: hand-written, changed when it changes |
| [`docs/design-principles.md`](docs/design-principles.md) | Interaction and design principles (how the app should feel) |
| [`docs/decisions.md`](docs/decisions.md) | One line per past decision whose full record was removed; git history holds the record |
| [`docs/roadmap.md`](docs/roadmap.md) | Direction and the ranking of what gets built next |
| [`docs/how-the-work-runs.md`](docs/how-the-work-runs.md) | How issues, epics, the project board and releases are organised |
| [`VISION.md`](VISION.md) | Product vision |
| [`docs/research-foundation.md`](docs/research-foundation.md) | Research basis for design decisions |
| [`docs/rebuild-review.md`](docs/rebuild-review.md) | Historical record: the 2026-07 pivot assessment and the retired coach design ([`specs/intrada-practice-coach-design.md`](specs/intrada-practice-coach-design.md)) |
| [`SETUP.md`](SETUP.md) | Configuration (Sentry, CI, TestFlight) and local setup in full |
| [`docs/working-with-agents.md`](docs/working-with-agents.md) | One unit of work in this repo, step by step: claim, isolate, route, plan, build, review, ship, measure |
| [Working with an agent: what good looks like](https://claude.ai/code/artifact/153ece12-8089-40c8-9f3d-5ce484503015) | The generic guide to Claude Code on any repo, written for a newcomer and shared as a Claude Doc: the ten rules, the shape of a unit, where the money goes, the gotchas, trust, and how to build a harness. No intrada specifics |

## CI/CD

GitHub Actions runs on every push:

- **PR checks**: test, clippy, fmt, security & hygiene, native iOS build + snapshot tests
- **Push to main**: all checks + native iOS release build (TestFlight lane runs separately on tag/dispatch)

## License

All rights reserved. See [`LICENSE`](LICENSE).
