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
    trunk serve --config crates/intrada-web/Trunk.toml &
    wait

# Start only the API server
dev-api:
    #!/usr/bin/env bash
    set -e
    pkill -f "intrada-api" 2>/dev/null || true
    sleep 0.3
    cargo run -p intrada-api

# Start only the web dev server
dev-web:
    #!/usr/bin/env bash
    set -e
    pkill -f "trunk serve" 2>/dev/null || true
    sleep 0.3
    trunk serve --config crates/intrada-web/Trunk.toml

# Run all tests
test:
    cargo test --workspace

# Run clippy with -D warnings (matches CI)
lint:
    cargo clippy --workspace -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check everything (test + lint + format check)
check:
    cargo test --workspace
    cargo clippy --workspace -- -D warnings
    cargo fmt --all -- --check

# Seed development data (API must be running)
seed:
    bash scripts/seed-dev-data.sh

# Build WASM for production or E2E testing
build:
    trunk build --config crates/intrada-web/Trunk.toml

# Kills any stale trunk-serve on 8080 — Playwright spins up its own preview
# server, so a leftover trunk would either steal the port or serve old WASM.
# Run E2E tests (builds WASM first).
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
# Type Generation
# ─────────────────────────────────────────────

# Automatically invalidates Xcode's incremental cache to avoid stale-type build errors.
# Regenerate Swift types from Rust core (run after changing intrada-core types).
typegen:
    cargo build -p shared_types
    @rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Intermediates.noindex 2>/dev/null || true
    @echo "✓ Swift types generated (Xcode incremental cache invalidated)"


# ─────────────────────────────────────────────
# iOS — Tauri/Leptos shell (active)
# ─────────────────────────────────────────────
# First-time setup:
#   cargo install tauri-cli --version "^2" --locked
#   brew install cocoapods
#   cd crates/intrada-mobile/src-tauri && cargo tauri ios init

# Runs trunk serve (web) in background, then tauri ios dev.
# Pre-boots the simulator before handing off to tauri so `simctl install`
# doesn't race against a Shutdown sim (the "Unable to lookup in current
# state: Shutdown" 405 error).
# Start the Tauri iOS dev session on simulator.
ios-dev:
    #!/usr/bin/env bash
    set -e
    trap 'kill 0' EXIT
    pkill -f "xcodebuild.*intrada-mobile" 2>/dev/null || true
    pkill -f "trunk serve" 2>/dev/null || true
    sleep 0.3
    echo "Starting trunk dev server..."
    trunk serve --config crates/intrada-web/Trunk.toml --address 0.0.0.0 &
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
    cd crates/intrada-mobile/src-tauri && cargo tauri ios dev "$SIM"
    wait

# Device must be connected via USB and trusted. Requires Wi-Fi for the
# dev server (device can't reach localhost — uses the host's LAN IP).
# Start the Tauri iOS dev session on a connected physical device.
ios-dev-device:
    #!/usr/bin/env bash
    set -e
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
    INTRADA_API_URL="http://$LAN_IP:8080" trunk serve --config crates/intrada-web/Trunk.toml --address 0.0.0.0 &
    echo "Starting Tauri iOS dev (device)..."
    cd crates/intrada-mobile/src-tauri && cargo tauri ios dev --host "$LAN_IP" "$DEVICE"
    wait

# Build Tauri iOS app for physical device (Xcode sideload — no TestFlight).
ios-build:
    cd crates/intrada-mobile/src-tauri && cargo tauri ios build

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
web-clean:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf crates/intrada-web/dist
    echo "✓ Removed crates/intrada-web/dist"

# ─────────────────────────────────────────────
# iOS — SwiftUI shell (ON HOLD)
# See specs/tauri-leptos-ios-shell.md. Do not use for active development.
# ─────────────────────────────────────────────

# Full iOS build: Rust (device debug), types, UniFFI bindings, Xcode project
[group('ios-swiftui (on hold)')]
ios-swiftui:
    bash scripts/build-ios.sh --device --debug
    @rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Intermediates.noindex 2>/dev/null || true
    cd ios && xcodegen generate

# Full build + open Xcode (SwiftUI)
[group('ios-swiftui (on hold)')]
ios-swiftui-dev: ios-swiftui
    open ios/Intrada.xcodeproj

# Release build for device CI (SwiftUI — on hold)
[group('ios-swiftui (on hold)')]
ios-release:
    bash scripts/build-ios.sh --device
    @rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Intermediates.noindex 2>/dev/null || true
    cd ios && xcodegen generate

# Release build for device + simulator (CI)
[group('ios-swiftui (on hold)')]
ios-release-all:
    bash scripts/build-ios.sh
    @rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Intermediates.noindex 2>/dev/null || true
    cd ios && xcodegen generate

# Simulator debug build
[group('ios-swiftui (on hold)')]
ios-sim:
    bash scripts/build-ios.sh --sim --debug
    @rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Intermediates.noindex 2>/dev/null || true
    cd ios && xcodegen generate

# Generate Swift types + UniFFI bindings only (no Rust cross-compilation)
[group('ios-swiftui (on hold)')]
ios-types:
    bash scripts/build-ios.sh --types
    @rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Intermediates.noindex 2>/dev/null || true
    cd ios && xcodegen generate

# Regenerate Xcode project from project.yml (no Rust build)
[group('ios-swiftui (on hold)')]
ios-project:
    cd ios && xcodegen generate

# Build for simulator, regenerate project, and run in simulator
[group('ios-swiftui (on hold)')]
ios-run: ios-sim
    #!/usr/bin/env bash
    set -euo pipefail

    # Find or boot the latest iPhone 16 simulator
    DEVICE_ID=$(xcrun simctl list devices available -j \
        | python3 -c "
    import json, sys
    data = json.load(sys.stdin)
    for runtime, devices in data['devices'].items():
        for d in devices:
            if d['name'] == 'iPhone 16' and d['isAvailable']:
                print(d['udid'])
    " | tail -1)

    if [ -z "$DEVICE_ID" ]; then
        echo "Error: No available iPhone 16 simulator found"
        exit 1
    fi

    echo "Using simulator: iPhone 16 ($DEVICE_ID)"
    xcrun simctl boot "$DEVICE_ID" 2>/dev/null || true

    cd ios
    xcodebuild build \
        -project Intrada.xcodeproj \
        -scheme Intrada \
        -destination "platform=iOS Simulator,id=$DEVICE_ID" \
        -configuration Debug \
        | xcpretty --color 2>/dev/null || true

    APP_PATH=$(find ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Products/Debug-iphonesimulator/Intrada.app -maxdepth 0 2>/dev/null | head -1)
    if [ -n "$APP_PATH" ]; then
        xcrun simctl install "$DEVICE_ID" "$APP_PATH"
        xcrun simctl launch "$DEVICE_ID" com.intrada.app
    else
        echo "Note: App not found in DerivedData. Build from Xcode for first run."
    fi

# Build for device without code signing (CI-style check)
[group('ios-swiftui (on hold)')]
ios-check: ios-release
    #!/usr/bin/env bash
    set -euo pipefail
    cd ios
    xcodebuild build \
        -project Intrada.xcodeproj \
        -scheme Intrada \
        -destination 'generic/platform=iOS' \
        -configuration Release \
        CODE_SIGNING_ALLOWED=NO \
        CODE_SIGN_IDENTITY="" \
        COMPILER_INDEX_STORE_ENABLE=NO \
        | xcpretty --color 2>/dev/null || true

# Use after modifying any Swift files to catch compile errors fast (~30s vs ~5min)
# Pass --clean to force a clean build (slower but avoids stale cache false positives)
# Automatically falls back to device target if simulator SDK is unavailable.
# Quick Swift-only build check (no Rust cross-compilation).
[group('ios-swiftui (on hold)')]
ios-swift-check *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cd ios
    if ! command -v xcodegen &>/dev/null; then
        echo "❌ xcodegen not installed (brew install xcodegen)" >&2
        exit 1
    fi

    # Check Xcode is available (not just Command Line Tools)
    XCODE_PATH=$(xcode-select -p 2>/dev/null || echo "")
    if [[ "$XCODE_PATH" == */CommandLineTools* ]] || [ -z "$XCODE_PATH" ]; then
        echo "❌ Full Xcode installation required (not just Command Line Tools)" >&2
        echo "   Current developer dir: $XCODE_PATH" >&2
        echo "   Fix: sudo xcode-select -s /Applications/Xcode.app/Contents/Developer" >&2
        exit 1
    fi

    xcodegen generate --quiet 2>/dev/null || xcodegen generate

    # Clean if requested
    if [[ " {{ ARGS }} " == *" --clean "* ]]; then
        echo "  Cleaning DerivedData..."
        xcodebuild clean -project Intrada.xcodeproj -scheme Intrada -quiet 2>/dev/null || true
    fi

    # Detect whether simulator SDK is available; fall back to device target if not
    if xcrun --sdk iphonesimulator --show-sdk-path &>/dev/null; then
        DESTINATION='generic/platform=iOS Simulator'
        echo "  Building for iOS Simulator..."
    else
        DESTINATION='generic/platform=iOS'
        echo "  Simulator SDK not available — building for device instead..."
    fi

    BUILD_LOG=$(mktemp)
    trap "rm -f $BUILD_LOG" EXIT

    set +e
    xcodebuild build \
        -project Intrada.xcodeproj \
        -scheme Intrada \
        -destination "$DESTINATION" \
        -configuration Debug \
        CODE_SIGNING_ALLOWED=NO \
        CODE_SIGN_IDENTITY="" \
        COMPILER_INDEX_STORE_ENABLE=NO \
        2>&1 | tee "$BUILD_LOG" | tail -20
    BUILD_EXIT=${PIPESTATUS[0]}
    set -e

    if [ $BUILD_EXIT -ne 0 ]; then
        echo ""
        echo "❌ iOS Swift build FAILED"
        ERRORS=$(grep -E "\.swift:[0-9]+:[0-9]+: error:" "$BUILD_LOG" || true)
        if [ -n "$ERRORS" ]; then
            echo ""
            echo "Swift errors:"
            echo "$ERRORS"
        fi
        exit 1
    fi
    echo "✓ iOS Swift build check passed"

    # Also validate iPad simulator build if simulators are available
    if xcrun --sdk iphonesimulator --show-sdk-path &>/dev/null; then
        IPAD_UDID=$(xcrun simctl list devices available 2>/dev/null | grep "iPad" | head -1 | grep -oE '[A-F0-9-]{36}' || echo "")
        if [ -n "$IPAD_UDID" ]; then
            echo "  Checking iPad build..."
            IPAD_LOG=$(mktemp)
            trap "rm -f $IPAD_LOG" EXIT
            set +e
            xcodebuild build \
                -project Intrada.xcodeproj \
                -scheme Intrada \
                -destination "platform=iOS Simulator,id=$IPAD_UDID" \
                -configuration Debug \
                CODE_SIGNING_ALLOWED=NO \
                CODE_SIGN_IDENTITY="" \
                COMPILER_INDEX_STORE_ENABLE=NO \
                -quiet 2>"$IPAD_LOG"
            IPAD_EXIT=$?
            set -e
            if [ $IPAD_EXIT -ne 0 ]; then
                echo ""
                echo "❌ iPad build FAILED"
                ERRORS=$(grep -E "\.swift:[0-9]+:[0-9]+: error:" "$IPAD_LOG" || true)
                if [ -n "$ERRORS" ]; then
                    echo ""
                    echo "Swift errors:"
                    echo "$ERRORS"
                fi
                exit 1
            fi
            echo "✓ iPad build check passed"
        fi
    fi

# Pass --clean to force a clean build.
# Falls back to device target if simulator SDK is unavailable.
# SwiftUI preview validation — checks all preview providers compile.
[group('ios-swiftui (on hold)')]
ios-preview-check *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cd ios
    if ! command -v xcodegen &>/dev/null; then
        echo "❌ xcodegen not installed (brew install xcodegen)" >&2
        exit 1
    fi

    XCODE_PATH=$(xcode-select -p 2>/dev/null || echo "")
    if [[ "$XCODE_PATH" == */CommandLineTools* ]] || [ -z "$XCODE_PATH" ]; then
        echo "❌ Full Xcode installation required (not just Command Line Tools)" >&2
        echo "   Fix: sudo xcode-select -s /Applications/Xcode.app/Contents/Developer" >&2
        exit 1
    fi

    xcodegen generate --quiet 2>/dev/null || xcodegen generate

    if [[ " {{ ARGS }} " == *" --clean "* ]]; then
        echo "  Cleaning DerivedData..."
        xcodebuild clean -project Intrada.xcodeproj -scheme Intrada -quiet 2>/dev/null || true
    fi

    if xcrun --sdk iphonesimulator --show-sdk-path &>/dev/null; then
        DESTINATION='generic/platform=iOS Simulator'
    else
        DESTINATION='generic/platform=iOS'
        echo "  Simulator SDK not available — building for device instead..."
    fi

    BUILD_LOG=$(mktemp)
    trap "rm -f $BUILD_LOG" EXIT

    set +e
    xcodebuild build \
        -project Intrada.xcodeproj \
        -scheme Intrada \
        -destination "$DESTINATION" \
        -configuration Debug \
        CODE_SIGNING_ALLOWED=NO \
        CODE_SIGN_IDENTITY="" \
        COMPILER_INDEX_STORE_ENABLE=NO \
        ENABLE_PREVIEWS=YES \
        2>&1 | tee "$BUILD_LOG" | tail -20
    BUILD_EXIT=${PIPESTATUS[0]}
    set -e

    if [ $BUILD_EXIT -ne 0 ]; then
        echo ""
        echo "❌ iOS preview check FAILED"
        ERRORS=$(grep -E "\.swift:[0-9]+:[0-9]+: error:" "$BUILD_LOG" || true)
        if [ -n "$ERRORS" ]; then
            echo ""
            echo "Swift errors:"
            echo "$ERRORS"
        fi
        exit 1
    fi
    echo "✓ iOS preview check passed"

# Catches runtime errors (missing environment objects, bad modifier ordering, etc.)
# Requires a prior `just ios-sim` or `just ios` build and simulator runtimes installed.
# Smoke test: build for sim, install, launch, verify app doesn't crash on startup.
[group('ios-swiftui (on hold)')]
ios-smoke-test:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! xcrun --sdk iphonesimulator --show-sdk-path &>/dev/null; then
        echo "❌ ios-smoke-test requires iOS Simulator runtimes." >&2
        echo "   Install via: Xcode → Settings → Platforms → iOS Simulator" >&2
        echo "   For device testing, build with 'just ios' and run from Xcode." >&2
        exit 1
    fi

    DEVICE="iPhone 16"
    BUNDLE_ID="com.intrada.app"
    TIMEOUT=8

    # Find the built app — check local build dir first, then DerivedData
    APP_PATH="ios/build/Build/Products/Debug-iphonesimulator/Intrada.app"
    if [ ! -d "$APP_PATH" ]; then
        APP_PATH=$(find ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Products/Debug-iphonesimulator/Intrada.app -maxdepth 0 2>/dev/null | head -1)
    fi
    if [ -z "$APP_PATH" ] || [ ! -d "$APP_PATH" ]; then
        echo "❌ App not built for simulator. Run 'just ios-swift-check' or 'just ios-sim' first." >&2
        exit 1
    fi
    echo "  Using app at: $APP_PATH"

    echo "=== iOS Smoke Test ==="

    # Boot simulator (ignore if already booted)
    echo "  Booting $DEVICE..."
    xcrun simctl boot "$DEVICE" 2>/dev/null || true
    sleep 2

    # Terminate any existing instance
    xcrun simctl terminate booted "$BUNDLE_ID" 2>/dev/null || true

    # Install and launch
    echo "  Installing app..."
    xcrun simctl install booted "$APP_PATH"
    echo "  Launching app..."
    xcrun simctl launch booted "$BUNDLE_ID"

    # Wait and check if process is still alive
    echo "  Waiting ${TIMEOUT}s for crash..."
    sleep "$TIMEOUT"

    # Check if the app process is still running
    PROC_CHECK=$(xcrun simctl spawn booted launchctl list 2>/dev/null | grep "$BUNDLE_ID" || true)
    if [ -n "$PROC_CHECK" ]; then
        echo "  ✓ App is still running after ${TIMEOUT}s"
    else
        echo "  ❌ App crashed or terminated within ${TIMEOUT}s!"
        echo ""
        echo "  Recent crash log:"
        # Show the most recent crash log for our app
        CRASH_LOG=$(find ~/Library/Logs/DiagnosticReports -name "Intrada-*" -newer /tmp/.ios-smoke-marker 2>/dev/null | head -1)
        if [ -n "$CRASH_LOG" ]; then
            head -30 "$CRASH_LOG"
        else
            echo "  (no crash log found — check Xcode console)"
        fi
        exit 1
    fi

    # Terminate cleanly
    xcrun simctl terminate booted "$BUNDLE_ID" 2>/dev/null || true
    echo ""
    echo "✓ iOS smoke test passed"

# Nuclear clean — removes every iOS artifact to guarantee a fresh build
[group('ios-swiftui (on hold)')]
ios-clean:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== iOS Nuclear Clean ==="
    # Xcode project (regenerated by xcodegen)
    rm -rf ios/Intrada.xcodeproj
    echo "  ✓ Removed Xcode project"
    # All generated Swift (SharedTypes, Serde, UniFFI bindings)
    rm -rf ios/Intrada/Generated
    echo "  ✓ Removed generated Swift files"
    # Rust cross-compiled libraries (debug + release, device + simulator)
    rm -rf target/aarch64-apple-ios
    rm -rf target/aarch64-apple-ios-sim
    echo "  ✓ Removed iOS Rust libraries"
    # Host dylib used for UniFFI binding generation
    rm -f target/debug/libshared.dylib target/debug/libshared.d
    rm -f target/release/libshared.dylib target/release/libshared.d
    echo "  ✓ Removed host dylibs"
    # Xcode DerivedData (cached builds, index, old linked libraries)
    rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*
    echo "  ✓ Removed Xcode DerivedData"
    # Xcode SPM/module caches that can hold stale type info
    rm -rf ~/Library/Caches/org.swift.swiftpm/Intrada-*
    echo "  ✓ Removed Swift package caches"
    echo ""
    echo "Clean complete. Run 'just ios' to rebuild everything."
