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

# Run all tests: nextest + doc tests, same as CI's `test` job.
# Local green must mean CI green: keep these flags in lockstep with ci.yml.
# (CI adds --profile ci for junit output only; assertions are identical.)
test:
    cargo nextest run --workspace
    cargo test --doc --workspace

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

# Check everything (fmt → clippy → test → hygiene, cheapest first)
check: fmt-check lint test hygiene

# Alias for check — catches errors before the 3-min CI roundtrip
pre-push: check

# Full gate: Rust (fmt/clippy/test) + the native iOS build & test suite.
# Slower — builds the iOS app — so run it before pushing changes under `ios/`.
# Plain `just check` stays Rust-only for fast Rust-only iterations.
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

# Open the app in Xcode (regenerates bindings first if the core changed).
[group('iOS')]
ios: _ios-sync
    cd ios && xcodegen generate
    xed ios/Intrada.xcodeproj

# Build + launch on a simulator and screenshot (regen if the core changed).
[group('iOS')]
ios-run: _ios-sync
    cd ios && xcodegen generate
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
    cd ios && xcodegen generate
    rm -f ios/generated/.gen-stamp
    @echo "✓ release core ready — opening Xcode. Select your device, then Product → Profile (⌘I). Next 'just ios' rebuilds the debug core."
    xed ios/Intrada.xcodeproj

# Build a signed Release .ipa and upload it to TestFlight (internal testing).
# Mirrors the release-testflight.yml CI lane for local debugging. Needs Ruby >=3
# (system Ruby 2.6 is too old — use rbenv) + the ASC_*/MATCH_* env set, and a
# one-time `fastlane match appstore` bootstrap. See specs/ios-testflight-cicd.md.
[group('iOS')]
testflight: ios-typegen (ios-package "release")
    cd ios && xcodegen generate
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

# Build + run the whole IntradaTests suite (snapshots + unit tests) on the
# pinned iPhone 16 / iOS 26.5 sim — the same command CI's `native-ios` job runs.
# Catches iOS-only breakage (e.g. a Swift type collision) locally instead of via
# the ~5-min macOS CI roundtrip. Regenerates bindings first if the core changed.
# The device pin must match the recorded snapshot references (renderer-specific).
[group('iOS')]
ios-test: _ios-sync
    #!/usr/bin/env bash
    set -euo pipefail
    cd ios
    xcodegen generate
    name="$(just _ios-test-sim-name)"
    udid="$(just _ios-test-sim-udid)"
    [ -n "$udid" ] || udid=$(xcrun simctl create "$name" "iPhone 16" "iOS26.5")
    xcodebuild test -project Intrada.xcodeproj -scheme Intrada -sdk iphonesimulator \
        -destination "id=$udid" -derivedDataPath build/dd \
        -clonedSourcePackagesDirPath build/spm -quiet \
        COMPILER_INDEX_STORE_ENABLE=NO CODE_SIGNING_ALLOWED=NO

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
