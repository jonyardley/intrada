set dotenv-load

# Default: show available commands
default:
    @just --list

# Kills any stale processes first so port conflicts don't serve old builds.
# Start the API dev server
dev:
    #!/usr/bin/env bash
    set -e
    pkill -f "intrada-api" 2>/dev/null || true
    sleep 0.3
    cargo run -p intrada-api

# Start only the API server (alias for `dev`)
dev-api: dev

# Type-check only (no codegen) — fastest feedback for "does it compile?"
check-fast:
    cargo check --workspace

# Run all tests: nextest, same as CI's `test` job.
# Local green must mean CI green: keep these flags in lockstep with ci.yml.
# (CI adds --profile ci for junit output only; assertions are identical.)
# Doc tests dropped (#1198): they compiled three crates to run zero tests —
# revisit if doc tests ever exist.
test:
    cargo nextest run --workspace

# Clippy with -D warnings: same targets as CI's `clippy` job.
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Format check only (what CI's fmt job runs)
fmt-check:
    cargo fmt --all -- --check

# Spell check + unused deps (what CI's Security & hygiene job runs).
# Both tools come from mise.toml (`mise install`) or brew.
hygiene:
    typos
    cargo-shear

# Check everything (fmt → clippy → test → hygiene, cheapest first). Mirrors
# the iOS test-tier green-stamp (#1200): skips on a clean, already-green HEAD
# (#1204). Delete `target/.check-stamp` to force a re-run.
check:
    #!/usr/bin/env bash
    set -euo pipefail
    stamp=target/.check-stamp
    sha="$(git rev-parse HEAD)"
    if [ -z "$(git status --porcelain)" ] && [ -f "$stamp" ] && [ "$(cat "$stamp")" = "$sha" ]; then
        echo "✓ HEAD $sha already green — skipping. Delete $stamp to force a re-run."
        exit 0
    fi
    just fmt-check lint test hygiene
    # Stamp only the exact tree we tested: a green run over uncommitted edits,
    # or one HEAD moved under, says nothing about $sha (#1204).
    if [ -z "$(git status --porcelain)" ] && [ "$(git rev-parse HEAD)" = "$sha" ]; then
        mkdir -p target
        echo "$sha" > "$stamp"
    fi

# Alias for check — catches errors before the 3-min CI roundtrip
pre-push: check

# Full gate: Rust (fmt/clippy/test) + the native iOS unit/snapshot tier.
# Slower — builds the iOS app — so run it before pushing changes under `ios/`.
# Plain `just check` stays Rust-only for fast Rust-only iterations. Runs the
# fast `ios-test` tier only; `ship` and CI additionally gate on
# `ios-test-full` (XCUITests too) before merge — see #1198.
check-all: check ios-test

# Seed development data (API must be running)
seed:
    bash scripts/seed-dev-data.sh

# ─────────────────────────────────────────────
# Diagnostics & cleanup
# ─────────────────────────────────────────────

# Helps diagnose "Address already in use" errors when a previous dev session
# didn't shut down cleanly. Pair with `dev` / `dev-api`, which already pkill
# stale processes — use this when those scripts can't reach the holder (e.g.
# a foreign process holding the port).
# Show what's listening on the dev ports we use (API).
ports:
    #!/usr/bin/env bash
    for PORT in 3001; do
        echo "Port $PORT:"
        lsof -nP -iTCP:$PORT -sTCP:LISTEN 2>/dev/null || echo "  (free)"
        echo
    done


# ─────────────────────────────────────────────
# iOS — native SwiftUI app (on the Crux core)
# ─────────────────────────────────────────────
# Daily loop: `just ios` (Xcode) or `just ios-run` (headless). Both regenerate
# the Swift bindings ONLY when the core changed, so they stay in sync without
# slowing pure-Swift edits. ios/generated is a build precondition (gitignored,
# regenerated) — never hand-edit it; fix the Rust type and regenerate.
# `--use-cache` (#1202) skips the project rewrite when project.yml is
# unchanged; verified it also invalidates on a changed SENTRY_DSN_NATIVE, so
# it's safe on every call site here.

# Open the app in Xcode (regenerates bindings first if the core changed).
[group('iOS')]
ios: _ios-sync
    cd ios && xcodegen generate --use-cache
    xed ios/Intrada.xcodeproj

# Build + launch on a simulator and screenshot (regen if the core changed).
[group('iOS')]
ios-run: _ios-sync
    cd ios && xcodegen generate --use-cache
    bash scripts/ios-run-sim.sh

# Stream the app's logs from the booted simulator, filtered to our subsystem —
# drops the UIKit/keyboard/gesture noise so first-party signal is visible.
# `report(_:)` (Core/Logging.swift) logs swallowed FFI errors here (#846 class).
[group('iOS')]
ios-logs:
    xcrun simctl spawn booted log stream --predicate 'subsystem == "com.intrada.native"'

# Force a full regenerate of both Swift packages + refresh the change-stamp.
[group('iOS')]
ios-gen: ios-typegen (ios-package "debug")
    @mkdir -p ios/generated
    @just _ios-src-hash > ios/generated/.gen-stamp
    @echo "✓ bindings regenerated"

# Prep an on-device PERFORMANCE build: regenerate the Crux core optimized for
# release, then in Xcode select your device and Profile (⌘I). `ios`/`ios-gen`
# build the core in debug (cargo-swift's default) — 10–100× slower in hot paths,
# so misleading for perf work; this rebuilds it with `--release`. ⌘I builds the
# Swift app Release, signs with project.yml's team, and opens Instruments.
# Clears the gen-stamp so the next plain `just ios` rebuilds the debug core
# (never link a release core into a routine debug run).
[group('iOS')]
ios-release: ios-typegen (ios-package "release")
    cd ios && xcodegen generate --use-cache
    rm -f ios/generated/.gen-stamp
    @echo "✓ release core ready — opening Xcode. Select your device, then Product → Profile (⌘I). Next 'just ios' rebuilds the debug core."
    xed ios/Intrada.xcodeproj

# Build a signed Release .ipa and upload it to TestFlight (internal testing).
# Mirrors the release-testflight.yml CI lane for local debugging. Needs Ruby >=3
# (system Ruby 2.6 is too old — use rbenv) + the ASC_*/MATCH_* env set, and a
# one-time `fastlane match appstore` bootstrap. See specs/ios-testflight-cicd.md.
[group('iOS')]
testflight: ios-typegen (ios-package "release")
    cd ios && xcodegen generate --use-cache
    rm -f ios/generated/.gen-stamp
    bundle exec fastlane ios beta

# Losslessly shrink snapshot references — drops Xcode's redundant all-opaque
# alpha channel (keeps pixels + sRGB), ~75% smaller. Run after (re)recording
# snapshots, before committing. CI's Snapshot Hygiene job enforces this.
[group('iOS')]
ios-snapshots-optimize:
    find ios/IntradaTests/__Snapshots__ -name '*.png' -exec oxipng -o max --quiet {} +
    @echo "✓ snapshots optimized — review the git diff and commit"

# Orphan + size-ceiling check on snapshot references (same as CI).
[group('iOS')]
ios-snapshots-check:
    bash scripts/check-snapshots.sh

# ios/generated is excluded from both fmt recipes: generated bindings, never
# hand-edited, so never formatted. Toolchain-bundled swift-format, default config.
# Format the hand-written Swift trees in place (fixes what ios-fmt-check flags).
[group('iOS')]
ios-fmt:
    swift format --in-place --recursive --parallel ios/Intrada ios/IntradaTests ios/IntradaUITests

# Swift formatting check, lint mode (same as CI). Run before pushing ios/** changes.
[group('iOS')]
ios-fmt-check:
    swift format lint --strict --recursive --parallel ios/Intrada ios/IntradaTests ios/IntradaUITests

# Fast tier: IntradaTests only (unit + snapshot) on the pinned iPhone 16 /
# iOS 26.5 sim. Seconds once built — catches wire breaks, codecs, upgrade
# paths. XCUITests are NOT run here: 204 unit/snapshot tests are the local
# signal that matters, the 18 XCUITests add ~2 min plus flake and caught
# nothing locally in the #1194 session (#1198). `ship` and CI additionally
# run `ios-test-full` before merge, so nothing merges without the UI tier.
# Regenerates bindings first if the core changed. The device pin must match
# the recorded snapshot references (renderer-specific).
[group('iOS')]
ios-test: _ios-sync (_ios-test-run "fast")

# Full gate: IntradaTests + IntradaUITests. What `ship` and CI run before
# merge — see #1198.
[group('iOS')]
ios-test-full: _ios-sync (_ios-test-run "full")

# Shared build+test body for both tiers. Splits `build-for-testing` from
# `test-without-building` (#1198) so a flake retry or test-only change reruns
# in seconds instead of rebuilding the whole app. Skips the run entirely when
# HEAD is clean and already stamped green at this tier (or better) — kills
# re-verifying an unchanged tree, e.g. re-running a gate a test-runner subagent
# already ran green (#1192). Delete `ios/build/.ios-test-stamp` to force.
[private]
_ios-test-run tier:
    #!/usr/bin/env bash
    set -euo pipefail
    stamp=ios/build/.ios-test-stamp
    sha="$(git rev-parse HEAD)"
    if [ -z "$(git status --porcelain)" ] && [ -f "$stamp" ]; then
        read -r stamped_sha stamped_tier < "$stamp" || true
        if [ "${stamped_sha:-}" = "$sha" ] && { [ "${stamped_tier:-}" = "full" ] || [ "${stamped_tier:-}" = "{{tier}}" ]; }; then
            echo "✓ HEAD $sha already green at tier '$stamped_tier' on a clean tree — skipping. Delete $stamp to force a re-run."
            exit 0
        fi
    fi
    just _ios-test-guard
    cd ios
    xcodegen generate --use-cache
    name="$(just _ios-test-sim-name)"
    udid="$(just _ios-test-sim-udid)"
    [ -n "$udid" ] || udid=$(xcrun simctl create "$name" "iPhone 16" "iOS26.5")
    only=""
    retry=""
    if [ "{{tier}}" = "fast" ]; then
        only="-only-testing:IntradaTests"
    else
        # Relaunch-in-new-process, not in-process: the flake this recovers
        # from kills the runner process (#1203). Fast tier (unit/snapshot,
        # deterministic) stays strict.
        retry="-retry-tests-on-failure -test-iterations 2 -test-repetition-relaunch-enabled YES"
    fi
    xcodebuild build-for-testing -project Intrada.xcodeproj -scheme Intrada -sdk iphonesimulator \
        -destination "id=$udid" -derivedDataPath build/dd \
        -clonedSourcePackagesDirPath build/spm -quiet \
        COMPILER_INDEX_STORE_ENABLE=NO CODE_SIGNING_ALLOWED=NO
    xcodebuild test-without-building -project Intrada.xcodeproj -scheme Intrada -sdk iphonesimulator \
        -destination "id=$udid" -derivedDataPath build/dd \
        -clonedSourcePackagesDirPath build/spm -quiet \
        $only $retry \
        COMPILER_INDEX_STORE_ENABLE=NO CODE_SIGNING_ALLOWED=NO
    cd ..
    # Same exact-tree guard as `check` above (#1204).
    if [ -z "$(git status --porcelain)" ] && [ "$(git rev-parse HEAD)" = "$sha" ]; then
        printf '%s %s\n' "$sha" "{{tier}}" > "$stamp"
    fi

# Refuse to start while another xcodebuild/XCTestAgent is already running
# against THIS checkout — two overlapping full-suite runs in one checkout
# share a simulator and crash each other's XCUITests (#1192). `xcodebuild` is
# invoked with a relative `-derivedDataPath` (from `cd ios`), so it never
# appears in the process's own command line — every checkout's argv is
# identical text. Match on each candidate process's cwd via `lsof` instead;
# parallel worktrees (distinct cwds, and already on distinct sims) are
# unaffected. Warns rather than blocking on uncommitted `crates/` changes: a
# concurrent core edit by another writer can red this gate with a compile
# error unrelated to the diff under test.
[private]
_ios-test-guard:
    #!/usr/bin/env bash
    set -euo pipefail
    here="$(pwd)/ios"
    for pid in $(pgrep -f 'xcodebuild|XCTestAgent' 2>/dev/null || true); do
        # `|| true`: the pid can exit between pgrep and lsof (a routine race,
        # not exotic) — lsof then fails, and under pipefail that failure
        # propagates through the assignment and aborts the whole script.
        cwd="$(lsof -a -p "$pid" -d cwd -Fn 2>/dev/null | sed -n 's/^n//p')" || true
        if [ "$cwd" = "$here" ]; then
            echo "✗ another xcodebuild/XCTestAgent (pid $pid) is already running in this checkout ($here)." >&2
            echo "  Concurrent full-suite runs in one checkout are never intentional (#1192) — wait for it to finish." >&2
            echo "  Check with: pgrep -fl 'xcodebuild|XCTestAgent'" >&2
            exit 1
        fi
    done
    if [ -n "$(git status --porcelain -- crates/ 2>/dev/null)" ]; then
        echo "⚠ uncommitted changes under crates/ — if another writer is mid-edit to intrada-core, a build failure below may be theirs, not this diff's (#1192)." >&2
    fi

# Per-worktree sim name (basename of the checkout, sanitised to simctl-safe
# chars) so parallel worktrees don't share one device. The device model is
# irrelevant to snapshot output — swift-snapshot-testing pins `.iPhone13`; only
# the iOS 26.5 runtime affects the pixels — so any distinct device is safe.
[private]
_ios-test-sim-name:
    @printf 'intrada-test-26-5-%s\n' "$(basename "$(git rev-parse --show-toplevel)" | tr -c 'A-Za-z0-9_-' '-' | sed 's/-*$//')"

# UDID of THIS worktree's snapshot sim, or empty if it doesn't exist yet.
[private]
_ios-test-sim-udid:
    @xcrun simctl list devices --json | python3 -c "import json,sys; d=json.load(sys.stdin)['devices']; print(next((x['udid'] for v in d.values() for x in v if x['name']=='$(just _ios-test-sim-name)'), ''))"

# Delete THIS worktree's snapshot sim (created by `ios-test`). Only ever removes
# the device named for the current worktree — never another worktree's or the
# main checkout's, and never a global reset (see the shared-simulator rule).
[group('iOS')]
ios-test-sim-clean:
    #!/usr/bin/env bash
    set -euo pipefail
    name="$(just _ios-test-sim-name)"
    udid="$(just _ios-test-sim-udid)"
    if [ -n "$udid" ]; then
        # `simctl delete` refuses a booted device, and `ios-test` leaves its sim
        # booted — shut it down first (ignore "already shutdown").
        xcrun simctl shutdown "$udid" 2>/dev/null || true
        xcrun simctl delete "$udid" && echo "✓ deleted $name ($udid)"
    else
        echo "✓ no sim named $name — nothing to clean"
    fi

# Facet typegen → ios/generated/SharedTypes (Event/Effect/ViewModel + bincode).
[group('iOS')]
ios-typegen:
    # Pre-clean so a renamed/removed core type can't leave an orphan Swift file
    # (crux's swift typegen overwrites but never deletes) — keeps typegen in sync.
    rm -rf ios/generated/SharedTypes
    RUST_LOG=info cargo run -p intrada-ffi --bin codegen --features codegen -- --output-dir ios/generated

# cargo-swift → ios/generated/IntradaCoreFFI (CoreFFI + RustFramework.xcframework).
[group('iOS')]
ios-package profile="debug":
    #!/usr/bin/env bash
    set -euo pipefail
    cd crates/intrada-ffi
    if [ "{{profile}}" = "release" ]; then rel="--release"; else rel=""; fi
    cargo swift package --name IntradaCoreFFI --platforms ios --lib-type static --features uniffi $rel --accept-all
    rm -rf ../../ios/generated/IntradaCoreFFI
    mkdir -p ../../ios/generated
    mv IntradaCoreFFI ../../ios/generated/IntradaCoreFFI
    # Requires cargo-swift 0.9.0 (`cargo install cargo-swift --version =0.9.0`):
    # its bundled uniffi-bindgen matches our uniffi=0.29.4 crate's runtime
    # contract; newer cargo-swift crashes the app with a contract mismatch.
    # cargo-swift nests the modulemap+header one level too deep; the
    # xcframework Info.plist declares HeadersPath=Headers, so canImport fails
    # and the FFI types vanish. Move them up (crux counter example's 0.9 fix).
    xcf=../../ios/generated/IntradaCoreFFI/RustFramework.xcframework
    moved=0
    for slice in "$xcf"/*/; do
        hd="$slice/headers"
        if [ -d "$hd/RustFramework" ]; then
            mv "$hd/RustFramework/"* "$hd/"; rmdir "$hd/RustFramework"; moved=1
        fi
    done
    [ "$moved" = 1 ] || echo "⚠️  cargo-swift header layout changed — verify canImport(intrada_ffiFFI)"
    echo "✓ ios/generated/IntradaCoreFFI"

# Regenerate bindings only if intrada-core / intrada-ffi changed since last gen.
[private]
_ios-sync:
    #!/usr/bin/env bash
    set -euo pipefail
    stamp=ios/generated/.gen-stamp
    current=$(just _ios-src-hash)
    if [ ! -d ios/generated/IntradaCoreFFI ] || [ ! -d ios/generated/SharedTypes ] || [ "$(cat "$stamp" 2>/dev/null)" != "$current" ]; then
        echo "↻ core changed (or no bindings) — regenerating…"
        just ios-gen
    else
        echo "✓ bindings up to date"
    fi

[private]
_ios-src-hash:
    @find crates/intrada-core/src crates/intrada-ffi/src crates/intrada-core/Cargo.toml crates/intrada-ffi/Cargo.toml -type f -exec shasum {} \; | shasum | cut -d' ' -f1
