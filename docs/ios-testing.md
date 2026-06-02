# Testing the native iOS app on the simulator

How to build, run, snapshot-test, and UI-test the native SwiftUI app
(`ios/`) on the iOS Simulator — including the tooling an AI agent uses to
do it, and the host gotchas that waste the most time.

## The tooling

### XcodeBuildMCP (checked into the repo)

`.mcp.json` registers the **XcodeBuildMCP** server so an MCP client
(Claude Code, etc.) can drive Xcode builds, simulators, and tests:

```jsonc
{
  "mcpServers": {
    "xcodebuild": { "type": "stdio", "command": "npx", "args": ["xcodebuildmcp@latest"] }
  }
}
```

It runs on demand via `npx` — no install step beyond Node. It exposes tools
to build a scheme, boot/list simulators, install + launch the app, run
tests, and capture screenshots. Upstream:
<https://github.com/cameroncooke/XcodeBuildMCP>.

> There is also an optional **Xcode-app driver** MCP (`mcp__xcode__*`:
> `XcodeListWindows`, `GetTestList`, `RunSomeTests`, `XcodeRead`, …) that
> automates an *open* Xcode window. It is configured per-developer (not in
> this repo). It's a fallback only — GUI builds hit unresolved-SwiftPM-package
> and code-signing errors that the CLI's `CODE_SIGNING_ALLOWED=NO` avoids, so
> prefer the CLI/`just` path below.

### Everything the MCP does, you can do with `just` + `xcodebuild` + `simctl`

These are the reliable primitives the tooling wraps. Bindings are a build
precondition (`ios/generated`, gitignored) — `just` regenerates them only when
the core changed.

```bash
just ios            # regen bindings if core changed → xcodegen → open Xcode
just ios-run        # regen → xcodegen → build + launch on sim + screenshot (SEED=1)
SEED=0 just ios-run # …against your real on-device data, not the demo seed
just ios-gen        # force-regenerate the Swift bindings (after a core change)
just ios-snapshots-optimize   # oxipng -o max every reference (run before commit)
just ios-snapshots-check      # orphan + 200 KB-ceiling guard (same as CI)
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
- **Clean up** the throwaway sims you create: `xcrun simctl delete <udid>`.

## CI

`.github/workflows/ci.yml` → **Native iOS (build + test)** runs the same
`xcodebuild test` on a pinned `macos-26` / Xcode 26.5 runner (clean host, no
pty contention), plus **Snapshot Hygiene**. If snapshots/UI tests are green
there, the local pty errors above were host-only.
