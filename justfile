set dotenv-load

# Default: show available commands
default:
    @just --list

# Start both API and web dev servers concurrently
dev:
    #!/usr/bin/env bash
    set -e
    trap 'kill 0' EXIT
    cargo run -p intrada-api &
    trunk serve --config crates/intrada-web/Trunk.toml &
    wait

# Start only the API server
dev-api:
    cargo run -p intrada-api

# Start only the web dev server
dev-web:
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
    cd e2e && npm install && npx playwright test --project=chromium

# ─────────────────────────────────────────────
# Type Generation
# ─────────────────────────────────────────────

# Regenerate Swift types from Rust core (run after changing intrada-core types)
typegen:
    cargo build -p shared_types
    @echo "✓ Swift types generated: crates/shared_types/generated/swift/"

# Check that generated Swift types are up to date (CI use)
typegen-check:
    #!/usr/bin/env bash
    set -euo pipefail
    # Snapshot current generated output
    BEFORE=$(find crates/shared_types/generated -name '*.swift' -exec md5sum {} + 2>/dev/null | sort || true)
    # Regenerate
    cargo build -p shared_types
    # Compare
    AFTER=$(find crates/shared_types/generated -name '*.swift' -exec md5sum {} + 2>/dev/null | sort || true)
    if [ "$BEFORE" != "$AFTER" ]; then
        echo "❌ Generated Swift types are out of date!"
        echo "   Run 'just typegen' and commit the changes."
        exit 1
    fi
    echo "✓ Generated Swift types are up to date."

# ─────────────────────────────────────────────
# iOS
# ─────────────────────────────────────────────

# Build Rust for iOS (device + simulator), generate Swift types, regenerate Xcode project
ios:
    bash scripts/build-ios.sh
    cd ios && xcodegen generate

# Build for device only + regenerate project
ios-device:
    bash scripts/build-ios.sh --device
    cd ios && xcodegen generate

# Build for simulator only + regenerate project
ios-sim:
    bash scripts/build-ios.sh --sim
    cd ios && xcodegen generate

# Generate Swift types only (no Rust cross-compilation) + regenerate project
ios-types:
    bash scripts/build-ios.sh --types
    cd ios && xcodegen generate

# Debug build for simulator (faster iteration)
ios-debug:
    bash scripts/build-ios.sh --sim --debug
    cd ios && xcodegen generate

# Regenerate Xcode project from project.yml
ios-project:
    cd ios && xcodegen generate

# Full build + regenerate project + open Xcode
ios-dev: ios
    open ios/Intrada.xcodeproj

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
ios-check: ios-device
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

# Clean iOS build artifacts
ios-clean:
    rm -rf ios/Intrada.xcodeproj
    rm -rf ios/Intrada/Generated
    rm -rf target/aarch64-apple-ios
    rm -rf target/aarch64-apple-ios-sim
    rm -rf ~/Library/Developer/Xcode/DerivedData/Intrada-*
