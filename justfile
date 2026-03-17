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
    pkill -f "intrada.api" 2>/dev/null || true
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
    pkill -f "intrada.api" 2>/dev/null || true
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

# Check that generated Swift types are up to date (CI use)
typegen-check:
    bash scripts/typegen-check.sh

# ─────────────────────────────────────────────
# iOS
# ─────────────────────────────────────────────

# Full iOS build: Rust (device debug), types, UniFFI bindings, Xcode project
# This is the main command for day-to-day iOS development.
# Automatically invalidates Xcode's incremental cache since generated types change.
ios:
    bash scripts/build-ios.sh --device --debug
    @rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*/Build/Intermediates.noindex 2>/dev/null || true
    cd ios && xcodegen generate

# Full build + open Xcode
ios-dev: ios
    open ios/Intrada.xcodeproj

# Release build for device (CI/TestFlight)
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
    cd ios
    xcodebuild build \
        -project Intrada.xcodeproj \
        -scheme Intrada \
        -destination 'platform=iOS Simulator,name=iPhone 16' \
        -configuration Debug \
        | xcpretty --color 2>/dev/null || true
    xcrun simctl boot "iPhone 16" 2>/dev/null || true
    xcrun simctl install booted build/Build/Products/Debug-iphonesimulator/Intrada.app 2>/dev/null || \
        echo "Note: Install from Xcode for first run"
    xcrun simctl launch booted com.intrada.app 2>/dev/null || \
        echo "Note: Launch from Xcode for first run"

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
ios-swift-check *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cd ios
    if ! command -v xcodegen &>/dev/null; then
        echo "❌ xcodegen not installed (brew install xcodegen)" >&2
        exit 1
    fi
    xcodegen generate --quiet 2>/dev/null || xcodegen generate

    # Clean if requested or if generated types changed since last build
    if [[ " {{ ARGS }} " == *" --clean "* ]]; then
        echo "  Cleaning DerivedData..."
        xcodebuild clean -project Intrada.xcodeproj -scheme Intrada -quiet 2>/dev/null || true
    fi

    BUILD_LOG=$(mktemp)
    trap "rm -f $BUILD_LOG" EXIT

    set +e
    xcodebuild build \
        -project Intrada.xcodeproj \
        -scheme Intrada \
        -destination 'generic/platform=iOS Simulator' \
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
        # Show all Swift errors for quick diagnosis
        ERRORS=$(grep -E "\.swift:[0-9]+:[0-9]+: error:" "$BUILD_LOG" || true)
        if [ -n "$ERRORS" ]; then
            echo ""
            echo "Swift errors:"
            echo "$ERRORS"
        fi
        exit 1
    fi
    echo "✓ iOS Swift build check passed"

# SwiftUI preview validation — checks all preview providers compile
# Pass --clean to force a clean build
ios-preview-check *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cd ios
    if ! command -v xcodegen &>/dev/null; then
        echo "❌ xcodegen not installed (brew install xcodegen)" >&2
        exit 1
    fi
    xcodegen generate --quiet 2>/dev/null || xcodegen generate

    if [[ " {{ ARGS }} " == *" --clean "* ]]; then
        echo "  Cleaning DerivedData..."
        xcodebuild clean -project Intrada.xcodeproj -scheme Intrada -quiet 2>/dev/null || true
    fi

    BUILD_LOG=$(mktemp)
    trap "rm -f $BUILD_LOG" EXIT

    set +e
    xcodebuild build \
        -project Intrada.xcodeproj \
        -scheme Intrada \
        -destination 'generic/platform=iOS Simulator' \
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
# Requires a prior `just ios-sim` or `just ios` build.
ios-smoke-test:
    #!/usr/bin/env bash
    set -euo pipefail

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
