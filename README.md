# Intrada

A **practice notebook** for musicians: build a session from your library, play
it through with a timer and rep counting, and score how it went (**Plan**,
**Practice**, **Track**).

Two native apps on one Rust core ([Crux](https://redbadger.github.io/crux/)),
kept at parity: SwiftUI for iPhone in `ios/` and Compose for Android in
`android/`. Both are offline-first: on-device SQLite is the source of truth, and
neither needs a network or an account. The core owns all logic; each app
renders its view model and fulfils effects.

Read next: [`CLAUDE.md`](CLAUDE.md) for the rules,
[`docs/roadmap.md`](docs/roadmap.md) for direction, `just status` for what is in
flight, [`SETUP.md`](SETUP.md) for accounts and secrets, and
[`docs/reference.md`](docs/reference.md) for commands. Run `just` to list the
recipes.

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

**Android**

- [Android Studio](https://developer.android.com/studio); the recipes use its
  JDK and `~/Library/Android/sdk` when `JAVA_HOME` and `ANDROID_HOME` are unset
- From the SDK Manager: Command-line Tools, the Emulator, Platform Tools, the
  SDK Platform and Build Tools the `android` job in `.github/workflows/ci.yml`
  installs, the NDK pinned as `intrada.ndkVersion` in
  `android/gradle.properties`, and the Android 36 Google APIs arm64 image
- `cargo install cargo-ndk --version 4.1.2` and
  `rustup target add aarch64-linux-android x86_64-linux-android`
- `just android-gen` once, to build the bindings

**Agent tooling (optional)**

- The Claude Code plugins `rust-analyzer-lsp`, `swift-lsp` and `kotlin-lsp`,
  plus `brew install xcode-build-server` and `brew install --cask kotlin-lsp`,
  then `just lsp-setup`; see code intelligence in
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
just android-run  # regenerates the Android bindings, boots the emulator, installs and screenshots
```

All rights reserved. See [`LICENSE`](LICENSE).
