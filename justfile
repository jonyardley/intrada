set dotenv-load

# Default: show available commands
default:
    @just --list

# Kills any stale processes first so port conflicts don't serve old builds.
# Start both API and web dev servers concurrently
dev:
    #!/usr/bin/env bash
    set -e
    # Kill stale dev processes to avoid port conflicts / serving old WASM
    pkill -f "trunk serve" 2>/dev/null || true
    pkill -f "intrada-api" 2>/dev/null || true
    sleep 0.5
    trap 'kill 0' EXIT
    echo "Starting API server..."
    cargo run -p intrada-api &
    echo "Starting web dev server (trunk serve — watches for changes)..."
    trunk serve --config crates/intrada-web/Trunk.toml --filehash false &
    wait

# Start only the API server
dev-api:
    #!/usr/bin/env bash
    set -e
    pkill -f "intrada-api" 2>/dev/null || true
    sleep 0.3
    cargo run -p intrada-api

# Start only the web dev server
[group('Web (on hold)')]
dev-web:
    #!/usr/bin/env bash
    set -e
    pkill -f "trunk serve" 2>/dev/null || true
    sleep 0.3
    trunk serve --config crates/intrada-web/Trunk.toml --filehash false

# Type-check only (no codegen) — fastest feedback for "does it compile?"
check-fast:
    cargo check --workspace

# Type-check just the web WASM target
[group('Web (on hold)')]
check-web:
    cargo check -p intrada-web --target wasm32-unknown-unknown

# Run all tests
test:
    cargo test --workspace

# Run clippy with -D warnings (matches CI)
lint:
    cargo clippy --workspace -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check everything (fmt → clippy → test, cheapest first)
check:
    cargo fmt --all -- --check
    cargo clippy --workspace -- -D warnings
    cargo test --workspace

# Alias for check — catches errors before the 3-min CI roundtrip
pre-push: check

# Full gate: Rust (fmt/clippy/test) + the native iOS build & test suite.
# Slower — builds the iOS app — so run it before pushing changes under `ios/`.
# Plain `just check` stays Rust-only for fast Rust-only iterations.
check-all: check ios-test

# Seed development data (API must be running)
seed:
    bash scripts/seed-dev-data.sh

# Build WASM for production or E2E testing
[group('Web (on hold)')]
build:
    trunk build --config crates/intrada-web/Trunk.toml

# Kills any stale trunk-serve on 8080 — Playwright spins up its own preview
# server, so a leftover trunk would either steal the port or serve old WASM.
# Run E2E tests (builds WASM first).
[group('Web (on hold)')]
e2e: build
    #!/usr/bin/env bash
    set -e
    pkill -f "trunk serve" 2>/dev/null || true
    sleep 0.3
    cd e2e
    # `npm install` is idempotent against an existing lockfile and skips
    # work when node_modules is already in sync. `npm ci` reinstalls every
    # time, which adds ~10s to a green run.
    npm install
    npx playwright test --project=chromium

# ─────────────────────────────────────────────
# Tauri/Leptos iOS shell — ON HOLD (being replaced by the native app above)
# ─────────────────────────────────────────────
# First-time setup:
#   cargo install tauri-cli --version "^2" --locked
#   brew install cocoapods
#   just tauri-init

# Generate the Xcode project and apply post-init patches.
# Run once after cloning, or after `cargo tauri ios init` regenerates gen/apple/.
[group('Tauri (on hold)')]
tauri-init:
    #!/usr/bin/env bash
    set -e
    MOBILE="crates/intrada-mobile"
    cd "$MOBILE/src-tauri" && cargo tauri ios init
    cd - > /dev/null
    echo "Applying post-init patches..."
    ruby "$MOBILE/scripts/fix-ios-build-config.rb"
    ruby "$MOBILE/scripts/add-live-activity-target.rb"
    echo "✓ iOS project ready. Run: just tauri-dev"

# Runs trunk serve (web) in background, then tauri ios dev.
# Pre-boots the simulator before handing off to tauri so `simctl install`
# doesn't race against a Shutdown sim (the "Unable to lookup in current
# state: Shutdown" 405 error).
# Start the Tauri iOS dev session on simulator.
[group('Tauri (on hold)')]
tauri-dev:
    #!/usr/bin/env bash
    set -e
    # Tag local mobile builds with the current commit so events from the
    # simulator land in Sentry attributed to a release. Empty if not in a
    # git checkout — `option_env!` filter in src-tauri/src/lib.rs treats
    # empty as no-release.
    export GIT_SHA="$(git rev-parse HEAD 2>/dev/null || echo '')"
    trap 'kill 0' EXIT
    pkill -f "xcodebuild.*intrada-mobile" 2>/dev/null || true
    pkill -f "trunk serve" 2>/dev/null || true
    sleep 0.3
    echo "Starting trunk dev server..."
    trunk serve --config crates/intrada-web/Trunk.toml --address 0.0.0.0 --filehash false &
    TRUNK_PID=$!
    echo "  Waiting for trunk to be ready on :8080..."
    until curl -sf http://localhost:8080/ > /dev/null 2>&1; do
        if ! kill -0 $TRUNK_PID 2>/dev/null; then
            echo "❌ trunk exited before becoming ready"; exit 1
        fi
        sleep 2
    done
    echo "  ✓ trunk ready"
    echo "Starting Tauri iOS dev (simulator)..."
    SIMS=()
    while IFS= read -r line; do SIMS+=("$line"); done < <(
        xcrun simctl list devices available 2>/dev/null \
            | grep "iPhone" \
            | awk -F'(' '{gsub(/^[[:space:]]+|[[:space:]]+$/, "", $1); print $1}' \
            | sort -u
    )
    if [ ${#SIMS[@]} -eq 0 ]; then
        echo "❌ No iPhone simulator found. Install one in Xcode → Settings → Platforms → iOS Simulator"
        exit 1
    elif [ ${#SIMS[@]} -eq 1 ]; then
        SIM="${SIMS[0]}"
    elif command -v fzf &>/dev/null; then
        SIM=$(printf '%s\n' "${SIMS[@]}" | fzf --prompt="Select simulator: ")
    else
        echo "Select simulator:"
        select SIM in "${SIMS[@]}"; do [ -n "$SIM" ] && break; done
    fi
    echo "  Using: $SIM"
    SIM_UDID=$(xcrun simctl list devices available -j | python3 -c "
    import json, sys
    name = sys.argv[1]
    data = json.load(sys.stdin)
    for runtime, devices in data['devices'].items():
        for d in devices:
            if d['name'] == name and d['isAvailable']:
                print(d['udid'])
                sys.exit(0)
    " "$SIM")
    if [ -z "$SIM_UDID" ]; then
        echo "❌ Could not resolve UDID for simulator: $SIM"
        exit 1
    fi
    echo "  Booting simulator (UDID: $SIM_UDID)..."
    # bootstatus -b boots the device if shutdown then waits for full boot.
    # Idempotent — no-op if already booted. Avoids the install-before-boot race.
    xcrun simctl bootstatus "$SIM_UDID" -b
    cd crates/intrada-mobile/src-tauri && cargo tauri ios dev --no-dev-server-wait "$SIM"
    wait

# Device must be connected via USB and trusted. Requires Wi-Fi for the
# dev server (device can't reach localhost — uses the host's LAN IP).
# Start the Tauri iOS dev session on a connected physical device.
[group('Tauri (on hold)')]
tauri-dev-device:
    #!/usr/bin/env bash
    set -e
    # Tag local device builds with the current commit so events from the
    # device land in Sentry attributed to a release. Same defaulting as
    # `ios-dev`.
    export GIT_SHA="$(git rev-parse HEAD 2>/dev/null || echo '')"
    trap 'kill 0' EXIT
    pkill -f "xcodebuild.*intrada-mobile" 2>/dev/null || true
    pkill -f "trunk serve" 2>/dev/null || true
    sleep 0.3
    # xcrun xctrace output format: "  DeviceName (Model) (UDID)"
    # cut -d'(' -f1 takes everything before the first '(' — assumes device
    # names don't contain parentheses (holds for all standard Apple device names).
    DEVICES=()
    while IFS= read -r line; do DEVICES+=("$line"); done < <(
        xcrun xctrace list devices 2>/dev/null \
            | grep -E "(iPhone|iPad)" \
            | grep -v "Simulator" \
            | cut -d'(' -f1 \
            | sed 's/^[[:space:]]*//' \
            | sed 's/[[:space:]]*$//' \
            | grep -v '^$'
    )
    if [ ${#DEVICES[@]} -eq 0 ]; then
        echo "❌ No physical iOS device found. Connect a device via USB and trust this Mac."
        exit 1
    elif [ ${#DEVICES[@]} -eq 1 ]; then
        DEVICE="${DEVICES[0]}"
    elif command -v fzf &>/dev/null; then
        DEVICE=$(printf '%s\n' "${DEVICES[@]}" | fzf --prompt="Select device: ")
    else
        echo "Select device:"
        select DEVICE in "${DEVICES[@]}"; do [ -n "$DEVICE" ] && break; done
    fi
    echo "  Using: $DEVICE"
    LAN_IP=$(ipconfig getifaddr en0 2>/dev/null || ipconfig getifaddr en1 2>/dev/null || echo "")
    if [ -z "$LAN_IP" ]; then
        echo "❌ Could not detect LAN IP. Connect to Wi-Fi and try again."
        exit 1
    fi
    echo "  Dev server: http://$LAN_IP:8080"
    echo "Starting trunk dev server..."
    # Override INTRADA_API_URL with the LAN IP so the WASM (compiled into the
    # device) can reach Trunk over Wi-Fi. localhost won't work — the device's
    # localhost is itself, not the Mac. build.rs detects the env change and
    # rebuilds. The Trunk proxy then forwards /api/* to localhost:3001.
    INTRADA_API_URL="http://$LAN_IP:8080" trunk serve --config crates/intrada-web/Trunk.toml --address 0.0.0.0 --filehash false &
    TRUNK_PID=$!
    echo "  Waiting for trunk to be ready on :8080..."
    until curl -sf "http://$LAN_IP:8080/" > /dev/null 2>&1; do
        if ! kill -0 $TRUNK_PID 2>/dev/null; then
            echo "❌ trunk exited before becoming ready"; exit 1
        fi
        sleep 2
    done
    echo "  ✓ trunk ready"
    echo "Starting Tauri iOS dev (device)..."
    # --no-dev-server-wait: we already verified trunk above, skip Tauri's
    # own 180 s poll (which also mis-resolves the port to :80).
    # --config overrides devUrl with the LAN IP so the device WebView loads
    # the correct address (localhost is unreachable from the physical device).
    cd crates/intrada-mobile/src-tauri && cargo tauri ios dev \
        --no-dev-server-wait \
        --config "{\"build\":{\"devUrl\":\"http://$LAN_IP:8080\"}}" \
        "$DEVICE"
    wait

# Build Tauri iOS app for physical device (Xcode sideload — no TestFlight).
# Tags the build with the current commit so events land in Sentry
# attributed to a release. Empty if not in a git checkout — the
# `option_env!` filter in src-tauri/src/lib.rs treats empty as
# no-release.
# Production iOS build. Compile-time env vars are set explicitly here
# so the build is correct regardless of what's in .env (which is for
# local dev). Always clean-rebuilds the frontend to avoid stale WASM.
[group('Tauri (on hold)')]
tauri-build:
    #!/usr/bin/env bash
    set -e
    export INTRADA_API_URL="https://intrada-api.fly.dev"
    export CLERK_PUBLISHABLE_KEY="${CLERK_PUBLISHABLE_KEY:?Set CLERK_PUBLISHABLE_KEY in .env}"
    export SENTRY_DSN_MOBILE="${SENTRY_DSN_MOBILE:-}"
    export GIT_SHA="$(git rev-parse HEAD 2>/dev/null || echo '')"
    echo "Building frontend (INTRADA_API_URL=$INTRADA_API_URL)..."
    rm -rf crates/intrada-web/dist
    trunk build --config crates/intrada-web/Trunk.toml
    echo "Building Tauri iOS (release)..."
    cd crates/intrada-mobile/src-tauri && cargo tauri ios build

# Build for production and install on a connected physical device.
# Requires ios-deploy (`brew install ios-deploy`).
[group('Tauri (on hold)')]
tauri-run-device: tauri-build
    #!/usr/bin/env bash
    set -e
    APP=$(find crates/intrada-mobile/src-tauri/gen/apple/build -name "Intrada.app" -path "*/Products/Applications/*" 2>/dev/null | head -1)
    if [ -z "$APP" ]; then
        echo "❌ No .app found — did ios-build succeed?"
        exit 1
    fi
    echo "Installing $APP on device..."
    ios-deploy --bundle "$APP" --no-wifi
    echo "✓ Installed and launched on device"

# ─────────────────────────────────────────────
# Diagnostics & cleanup
# ─────────────────────────────────────────────

# Helps diagnose "Address already in use" errors when a previous dev session
# didn't shut down cleanly. Pair with `dev` / `dev-api` / `dev-web` which
# already pkill stale processes — use this when those scripts can't reach
# the holder (e.g. a foreign process holding the port).
# Show what's listening on the dev ports we use (8080 trunk, 3001 API).
ports:
    #!/usr/bin/env bash
    for PORT in 8080 3001; do
        echo "Port $PORT:"
        lsof -nP -iTCP:$PORT -sTCP:LISTEN 2>/dev/null || echo "  (free)"
        echo
    done

# Use when a stale build cache is suspected — trunk's incremental cache is
# usually reliable, but a `git clean`-equivalent is occasionally useful when
# diagnosing weird wasm-bindgen mismatches after a major dep bump.
# Remove the trunk build output for the web app.
[group('Web (on hold)')]
web-clean:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf crates/intrada-web/dist
    echo "✓ Removed crates/intrada-web/dist"


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
