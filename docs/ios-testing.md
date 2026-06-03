# Testing the native iOS app on the simulator

How to build, run, snapshot-test, and UI-test the native SwiftUI app
(`ios/`) on the iOS Simulator — including the tooling an AI agent uses to
do it, and the host gotchas that waste the most time.

## The tooling

**The reliable, primary path is the Xcode CLI: `just` + `xcodebuild` + `xcrun
simctl`.** That's what builds, runs the simulator, takes screenshots, and runs
the tests below. Two MCP servers are *available* as conveniences, but neither is
required and the CLI is what you should reach for first.

### Xcode CLI (what to use)

Bindings are a build precondition (`ios/generated`, gitignored) — `just`
regenerates them only when the core changed.

```bash
# Simulator control + screenshots
UDID=$(xcrun simctl create snap "iPhone 16" "iOS26.5")
xcrun simctl boot "$UDID"
xcrun simctl install "$UDID" /path/to/Intrada.app
xcrun simctl launch "$UDID" com.intrada.native --seed-sample-data
xcrun simctl io "$UDID" screenshot shot.png      # ← how to screenshot the sim
```

### MCP servers (optional)

- **XcodeBuildMCP** — registered in `.mcp.json` (`npx xcodebuildmcp@latest`, runs
  on demand, no install beyond Node). It exposes MCP tools that wrap the same
  build/simulator/test/screenshot actions for an MCP client. Upstream:
  <https://github.com/cameroncooke/XcodeBuildMCP>. Available, but the CLI above
  is the path of record.
- **Xcode-app driver** (`mcp__xcode__*`: `XcodeListWindows`, `GetTestList`,
  `RunSomeTests`, `GetBuildLog`, …) automates an *open* Xcode window. Configured
  per-developer (not in this repo). Fallback only — GUI builds hit
  unresolved-SwiftPM-package and code-signing errors that the CLI's
  `CODE_SIGNING_ALLOWED=NO` avoids.

### `just` recipes

```bash
just ios            # regen bindings if core changed → xcodegen → open Xcode
just ios-run        # regen → xcodegen → build + launch on sim + screenshot (SEED=1)
SEED=0 just ios-run # …against your real on-device data, not the demo seed
just ios-gen        # force-regenerate the Swift bindings (after a core change)
just ios-snapshots-optimize   # oxipng -o max every reference (run before commit)
just ios-snapshots-check      # orphan + 200 KB-ceiling guard (same as CI)
just ios-test                 # full IntradaTests suite on a per-worktree sim (CI parity)
just ios-test-sim-clean       # delete this worktree's ios-test sim
```

## Snapshot tests (the per-PR UI regression gate)

References live in `ios/IntradaTests/__Snapshots__/**` and are recorded on
**iPhone 16 / iOS 26.5** to match CI (renderer-specific — see
`.github/workflows/ci.yml` and CLAUDE.md → "Snapshot test hygiene").

```bash
# Create the CI-matching simulator
UDID=$(xcrun simctl create snap "iPhone 16" "iOS26.5")

# Run / record (a missing reference auto-records and "fails" the first run)
xcodebuild test -project ios/Intrada.xcodeproj -scheme Intrada -sdk iphonesimulator \
  -destination "id=$UDID" CODE_SIGNING_ALLOWED=NO \
  -only-testing:IntradaTests/ScreenSnapshotTests/testLibraryScreen

# After recording: optimise (drops Xcode's opaque alpha, ~75% smaller) and re-run
just ios-snapshots-optimize
```

Re-record after any intentional UI change; delete a test → delete its PNG
(orphans fail CI). Optimise before committing or the Snapshot Hygiene job fails.

## UI tests (gesture / interaction — what snapshots can't cover)

`ios/IntradaUITests/` drives the *running* app (e.g. type in search → assert the
list filters). Launch args seed deterministic data:

```bash
xcodebuild test -project ios/Intrada.xcodeproj -scheme Intrada -sdk iphonesimulator \
  -destination "id=$UDID" CODE_SIGNING_ALLOWED=NO \
  -only-testing:IntradaUITests
```

The app seeds the 6-item demo set when launched with `--seed-sample-data`
(`XCUIApplication().launchArguments = ["--seed-sample-data"]`). Use a UI test —
not a snapshot — to prove an *interaction* works; a snapshot can't catch that a
gesture (e.g. pull-to-reveal) never fired.

## Host gotchas (these eat hours)

- **Quit Xcode before `xcodebuild test`.** A persistent
  `Pseudo Terminal Setup Error / Device not configured` ("Failed to install or
  launch the test runner") is almost always Xcode.app holding the simulator
  while the CLI also wants it. `osascript -e 'quit app "Xcode"'`, then re-run.
- **Transient runner flake** → restart the sim service:
  `killall com.apple.CoreSimulator.CoreSimulatorService Simulator`, then re-boot
  the sim. UI-test *runners* trip this first; if unit tests pass but the UI test
  fails on launch, it's the host, not the test.
- **Stale bindings after pulling/rebasing** onto a main with core changes →
  `extra argument` / `cannot find type` Swift errors. Run `just ios-gen`
  (see CLAUDE.md → Native iOS Shell, and the memory note on rebase+regen).
- **Clean up** the throwaway sims you create: `xcrun simctl delete <udid>`
  (or `just ios-test-sim-clean` for the sim `just ios-test` made in this worktree).

## Running alongside another checkout (worktrees)

Git worktrees and the main checkout are **isolated on disk** — separate working
trees, DerivedData (keyed by project *path*), `ios/generated` bindings, cargo
`target/`, and snapshot PNGs. Building or recording in one never overwrites the
other's files.

**The simulator is the exception.** The iOS Simulator and
`CoreSimulatorService` are **one per macOS login, shared across every checkout**.
That's the only real clash surface, and the recovery commands above are global
sledgehammers — `killall com.apple.CoreSimulator.CoreSimulatorService`,
`simctl shutdown all`, `simctl erase|delete` will **kill or wipe a sim another
checkout is using**.

Rules to keep two checkouts from colliding:

- **`just ios-test` is safe to run in parallel across worktrees.** It names its
  sim per worktree (`intrada-test-26-5-<worktree-basename>`), so each checkout
  with a distinct basename gets its own device — no serialization. (Two
  worktree dirs that sanitise to the same name — e.g. `foo.1` and `foo-1` —
  would share a sim; slug-like worktree names avoid this.) The device model is irrelevant to
  snapshot output (swift-snapshot-testing pins `.iPhone13`; only the iOS 26.5
  runtime affects the pixels), so per-worktree devices change nothing about
  pass/fail. `just ios-test-sim-clean` deletes only the current worktree's sim.
  This removes *blocking*, not resource load — N booted sims + N Swift builds is
  heavy, so the practical ceiling is how many parallel agents the host can take.
- **Ad-hoc `xcodebuild` / `simctl` sessions that share one device still
  serialize.** If you run the raw `xcodebuild test` snippets above (not via
  `just ios-test`), give each session a **worktree-scoped sim** targeted by
  UDID, instead of a bare `"iPhone 16"` both checkouts might grab:
  ```bash
  UDID=$(xcrun simctl create "snap-$(basename "$PWD")" "iPhone 16" "iOS26.5")
  ```
  Two sessions pointed at the *same* device produce the pty contention errors
  above; distinct devices run concurrently.
- **Only touch sims you created.** Delete *your* UDID (or `just
  ios-test-sim-clean` for the recipe's sim) when done; never `shutdown all` /
  `delete unavailable` / restart `CoreSimulatorService` blind.
- **Check before any global op or a fresh test run** whether another session is
  live:
  ```bash
  xcrun simctl list devices | grep Booted     # sims someone may be using
  pgrep -fl 'xcodebuild|XCTestAgent'           # a build/test already running
  pgrep -x Xcode                               # Xcode open (may hold a sim)
  ```
  If any of those show activity you didn't start, **stop and ask** before
  resetting the sim service or shutting sims down — assume it's the other
  checkout's. (For agents: this is a hard rule — see CLAUDE.md → Native iOS.)

## CI

`.github/workflows/ci.yml` → **Native iOS (build + test)** runs the same
`xcodebuild test` on a pinned `macos-26` / Xcode 26.5 runner (clean host, no
pty contention), plus **Snapshot Hygiene**. If snapshots/UI tests are green
there, the local pty errors above were host-only.
