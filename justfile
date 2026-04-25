set dotenv-load

# Default: show available commands
default:
    @just --list

# Start both API and web dev servers concurrently
# Kills any stale processes first so port conflicts don't serve old builds.
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

# Run clippy
lint:
    cargo clippy --workspace

# Format code
fmt:
    cargo fmt --all

# Check everything (test + lint + format check)
check:
    cargo test --workspace
    cargo clippy --workspace
    cargo fmt --all -- --check

# Seed development data (API must be running)
seed:
    bash scripts/seed-dev-data.sh

# Build WASM for production or E2E testing
build:
    trunk build --config crates/intrada-web/Trunk.toml

# Run E2E tests (builds WASM first)
e2e: build
    cd e2e && npm ci && npx playwright test --project=chromium

# ─────────────────────────────────────────────
# Type Generation
# ─────────────────────────────────────────────

# Regenerate Swift types from Rust core (run after changing intrada-core types)
# Automatically invalidates Xcode's incremental cache to avoid stale-type build errors.
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

# Start the Tauri iOS dev session on simulator.
# Runs trunk serve (web) in background, then tauri ios dev.
ios-dev:
    #!/usr/bin/env bash
    set -e
    trap 'kill 0' EXIT
    echo "Starting trunk dev server..."
    trunk serve --config crates/intrada-web/Trunk.toml --address 0.0.0.0 &
    echo "Starting Tauri iOS dev (simulator)..."
    mapfile -t SIMS < <(xcrun simctl list devices available 2>/dev/null \
        | grep -E "^\s+iPhone" \
        | sed -E 's/^\s+(iPhone[^(]+).*/\1/' \
        | sed 's/[[:space:]]*$//' \
        | sort -u)
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
    cd crates/intrada-mobile/src-tauri && cargo tauri ios dev "$SIM"
    wait

# Build Tauri iOS app for physical device (Xcode sideload — no TestFlight).
ios-build:
    cd crates/intrada-mobile/src-tauri && cargo tauri ios build

# ─────────────────────────────────────────────
# iOS — SwiftUI shell (ON HOLD)
# See specs/tauri-leptos-ios-shell.md. Do not use for active development.
# ─────────────────────────────────────────────

# Full iOS build: Rust (device debug), types, UniFFI bindings, Xcode project
ios-swiftui:
    bash scripts/build-ios.sh --device --debug
    @rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Intermediates.noindex 2>/dev/null || true
    cd ios && xcodegen generate

# Full build + open Xcode (SwiftUI)
ios-swiftui-dev: ios-swiftui
    open ios/Intrada.xcodeproj

# Release build for device CI (SwiftUI — on hold)
ios-release:
    bash scripts/build-ios.sh --device
    @rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Intermediates.noindex 2>/dev/null || true
    cd ios && xcodegen generate

# Release build for device + simulator (CI)
ios-release-all:
    bash scripts/build-ios.sh
    @rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Intermediates.noindex 2>/dev/null || true
    cd ios && xcodegen generate

# Simulator debug build
ios-sim:
    bash scripts/build-ios.sh --sim --debug
    @rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Intermediates.noindex 2>/dev/null || true
    cd ios && xcodegen generate

# Generate Swift types + UniFFI bindings only (no Rust cross-compilation)
ios-types:
    bash scripts/build-ios.sh --types
    @rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Intermediates.noindex 2>/dev/null || true
    cd ios && xcodegen generate

# Regenerate Xcode project from project.yml (no Rust build)
ios-project:
    cd ios && xcodegen generate

# Build for simulator, regenerate project, and run in simulator
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

# Quick Swift-only build check (no Rust cross-compilation)
# Use after modifying any Swift files to catch compile errors fast (~30s vs ~5min)
# Pass --clean to force a clean build (slower but avoids stale cache false positives)
# Automatically falls back to device target if simulator SDK is unavailable.
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

# SwiftUI preview validation — checks all preview providers compile
# Pass --clean to force a clean build
# Falls back to device target if simulator SDK is unavailable.
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

# Smoke test: build for sim, install, launch, verify app doesn't crash on startup
# Catches runtime errors (missing environment objects, bad modifier ordering, etc.)
# Requires a prior `just ios-sim` or `just ios` build and simulator runtimes installed.
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
