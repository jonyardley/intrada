# Intrada

A **practice notebook** for musicians: build a session from your library, play
it through with a timer and rep counting, and score how it went (**Plan**,
**Practice**, **Track**).

A native iPhone app in SwiftUI (`ios/`) on a Rust core
([Crux](https://redbadger.github.io/crux/)). It is offline-first: on-device
SQLite is the source of truth, and it needs no network or account. The core
owns all logic; the app renders its view model and fulfils effects.

Read next: [`CLAUDE.md`](CLAUDE.md) for the rules,
[`docs/roadmap.md`](docs/roadmap.md) for direction, `just status` for what is in
flight, [`SETUP.md`](SETUP.md) for accounts and secrets, and
[`docs/reference.md`](docs/reference.md) for the mechanics. `just --list` lists
the commands.

## Prerequisites

**Everyone**

- Rust via [rustup](https://rustup.rs), pinned in `rust-toolchain.toml`. Not
  Homebrew's Rust, whose `cargo` shadows rustup and has no mobile targets
- [mise](https://mise.jdx.dev), then `mise install`, for everything `mise.toml`
  pins (just, xcodegen, cargo-nextest, Ruby, cargo-swift 0.9.0 and more)
- [gh](https://cli.github.com), signed in with `gh auth login`, and `jq`
- [worktrunk](https://worktrunk.dev) (`brew install worktrunk`); its config is
  in [`docs/worktrees.md`](docs/worktrees.md)
- Optional: `cargo-deny` and `gitleaks`, which `just check` skips when missing

**iPhone**

- Xcode 26+ with the iOS Simulator runtime
- `rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios`

**Agent tooling (optional)**

- The Claude Code plugins `rust-analyzer-lsp` and `swift-lsp`, plus
  `brew install xcode-build-server`, then `just lsp-setup`; see code
  intelligence in
  [`docs/working-with-agents.md`](docs/working-with-agents.md)
- [graphify](https://github.com/Graphify-Labs/graphify)
  (`uv tool install graphifyy`); see
  [`docs/reference.md`](docs/reference.md#knowledge-graph-graphify)
- [rtk](https://www.rtk-ai.app) (`brew install rtk`, then
  `rtk init -g --hook-only --auto-patch`), which shortens command output

Git hooks install in Claude Code sessions; elsewhere run
`bash scripts/install-git-hooks.sh` once. Forking: set your Apple Team ID in
`ios/project.yml`.

## Quick start

```bash
just ios          # regenerates the Swift bindings if the core changed, then opens Xcode
```

All rights reserved. See [`LICENSE`](LICENSE).
