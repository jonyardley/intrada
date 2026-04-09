#!/usr/bin/env bash
# build-ios.sh — Cross-compile the shared crate for iOS and generate Swift bindings.
#
# Usage:
#   ./scripts/build-ios.sh          # Build for device + simulator, generate types
#   ./scripts/build-ios.sh --device  # Build for device only
#   ./scripts/build-ios.sh --sim     # Build for simulator only
#   ./scripts/build-ios.sh --types   # Generate Swift types only (no Rust build)
#
# Prerequisites:
#   - Rust iOS targets: rustup target add aarch64-apple-ios aarch64-apple-ios-sim
#   - Xcode + command line tools

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Output directories — inside the Xcode source root so XcodeGen picks them up
GENERATED_DIR="$ROOT_DIR/ios/Intrada/Generated"
BINDINGS_DIR="$GENERATED_DIR/UniFFI"

# Where facet typegen outputs the Swift package
TYPEGEN_OUTPUT="$ROOT_DIR/crates/shared_types/generated/swift/SharedTypes"

# Targets
DEVICE_TARGET="aarch64-apple-ios"
SIM_TARGET="aarch64-apple-ios-sim"

# Defaults
BUILD_DEVICE=true
BUILD_SIM=true
GENERATE_TYPES=true
PROFILE="release"

# Parse args
for arg in "$@"; do
    case $arg in
        --device)
            BUILD_SIM=false
            ;;
        --sim)
            BUILD_DEVICE=false
            ;;
        --types)
            BUILD_DEVICE=false
            BUILD_SIM=false
            ;;
        --debug)
            PROFILE="dev"
            ;;
        --help)
            echo "Usage: $0 [--device|--sim|--types] [--debug]"
            exit 0
            ;;
    esac
done

PROFILE_DIR="release"
PROFILE_FLAG="--release"
if [ "$PROFILE" = "dev" ]; then
    PROFILE_DIR="debug"
    PROFILE_FLAG=""
fi

echo "=== Intrada iOS Build ==="
echo "  Root:    $ROOT_DIR"
echo "  Profile: $PROFILE"
echo ""

# ───────────────────────────────────────────────────
# Pre-flight: verify Xcode (not just Command Line Tools)
# ───────────────────────────────────────────────────

XCODE_PATH=$(xcode-select -p 2>/dev/null || echo "")
if [[ "$XCODE_PATH" == */CommandLineTools* ]] || [ -z "$XCODE_PATH" ]; then
    echo "❌ Full Xcode installation required for iOS builds." >&2
    echo "   Current developer dir: ${XCODE_PATH:-<not set>}" >&2
    echo "" >&2
    echo "   If Xcode is installed, run:" >&2
    echo "     sudo xcode-select -s /Applications/Xcode.app/Contents/Developer" >&2
    echo "" >&2
    echo "   If Xcode is not installed, download it from the App Store or" >&2
    echo "   https://developer.apple.com/xcode/" >&2
    exit 1
fi

if [ "$BUILD_DEVICE" = true ] && ! xcrun --sdk iphoneos --show-sdk-path &>/dev/null; then
    echo "❌ iPhoneOS SDK not found." >&2
    echo "   Current developer dir: $XCODE_PATH" >&2
    echo "   Try: sudo xcode-select -s /Applications/Xcode.app/Contents/Developer" >&2
    exit 1
fi

if [ "$BUILD_SIM" = true ] && ! xcrun --sdk iphonesimulator --show-sdk-path &>/dev/null; then
    echo "❌ iPhone Simulator SDK not found." >&2
    echo "   Install via: Xcode → Settings → Platforms → iOS Simulator" >&2
    echo "   Or skip simulator with: $0 --device" >&2
    exit 1
fi

# ───────────────────────────────────────────────────
# Step 1: Cross-compile shared crate for iOS targets
# ───────────────────────────────────────────────────

if [ "$BUILD_DEVICE" = true ]; then
    echo "→ Building for device ($DEVICE_TARGET)..."
    cargo build -p shared --features uniffi $PROFILE_FLAG --target "$DEVICE_TARGET"
    echo "  ✓ Device library: target/$DEVICE_TARGET/$PROFILE_DIR/libshared.a"
fi

if [ "$BUILD_SIM" = true ]; then
    echo "→ Building for simulator ($SIM_TARGET)..."
    cargo build -p shared --features uniffi $PROFILE_FLAG --target "$SIM_TARGET"
    echo "  ✓ Simulator library: target/$SIM_TARGET/$PROFILE_DIR/libshared.a"
fi

# ───────────────────────────────────────────────────
# Step 2: Generate Swift type bindings via facet typegen
# ───────────────────────────────────────────────────

if [ "$GENERATE_TYPES" = true ]; then
    echo ""
    echo "→ Generating Swift type bindings (facet typegen)..."

    # Build shared_types crate — its build.rs runs TypeRegistry to generate Swift
    cargo build -p shared_types
    echo "  ✓ SharedTypes generated: $TYPEGEN_OUTPUT"

    # Remove old hand-written files (replaced by fully automated typegen)
    for old_file in "$GENERATED_DIR/SharedTypes.swift" "$GENERATED_DIR/CodableExtensions.swift"; do
        if [ -f "$old_file" ]; then
            rm "$old_file"
            echo "  ✓ Removed old $(basename "$old_file")"
        fi
    done

    # ─── 2a. Copy generated SharedTypes into the iOS project ───

    mkdir -p "$GENERATED_DIR/SharedTypes"
    cp -R "$TYPEGEN_OUTPUT/Sources/SharedTypes/"*.swift "$GENERATED_DIR/SharedTypes/"

    # Strip `import Serde` — generated as a separate module but we compile in a single target
    sed -i '' '/^import Serde$/d' "$GENERATED_DIR/SharedTypes/"*.swift

    # Prepend "DO NOT EDIT" header to generated files
    for f in "$GENERATED_DIR/SharedTypes/"*.swift; do
        sed -i '' '1s;^;// ⚠️  AUTO-GENERATED by `just typegen` — DO NOT EDIT manually.\n// Changes will be overwritten on next `just typegen` or `build-ios.sh` run.\n\n;' "$f"
    done

    echo "  ✓ Copied to: $GENERATED_DIR/SharedTypes/ (stripped import Serde)"

    # ─── 2b. Generate TypeExtensions.swift ───
    #
    # Minimal extensions needed for Swift 6 concurrency compliance.
    # JSON Codable is NOT needed — all JSON serialization happens in Rust
    # via crux_http. The shell only passes opaque byte arrays.

    cat > "$GENERATED_DIR/SharedTypes/TypeExtensions.swift" << 'TYPE_EXTENSIONS'
// ⚠️  AUTO-GENERATED by build-ios.sh — DO NOT EDIT manually.
// Changes will be overwritten on next `just typegen` or `build-ios.sh` run.

import Foundation

// MARK: - Indirect Property Wrapper Extensions

// Auto-generated structs use @Indirect for all properties. This Sendable
// conformance lets them cross concurrency boundaries in Swift 6.

extension Indirect: @unchecked Sendable where T: Sendable {}
TYPE_EXTENSIONS
    echo "  ✓ Generated TypeExtensions.swift (Indirect Sendable)"

    # ─── 2c. Copy Serde runtime (BCS serialisation helpers) ───

    mkdir -p "$GENERATED_DIR/Serde"
    cp -R "$TYPEGEN_OUTPUT/Sources/Serde/"*.swift "$GENERATED_DIR/Serde/"

    # Prepend "DO NOT EDIT" header to Serde files
    for f in "$GENERATED_DIR/Serde/"*.swift; do
        sed -i '' '1s;^;// ⚠️  AUTO-GENERATED by `just typegen` — DO NOT EDIT manually.\n\n;' "$f"
    done

    echo "  ✓ Serde runtime: $GENERATED_DIR/Serde/"

    # ─── 2d. Generate UniFFI bindings (CoreFFI + CoreJson interface) ───

    mkdir -p "$BINDINGS_DIR"

    # Build for host to get a dylib — iOS cross-compilation only produces
    # static .a files, but uniffi-bindgen needs a dylib to extract metadata.
    echo "  → Building host dylib for UniFFI binding generation..."
    cargo build -p shared --features uniffi
    LIB_PATH="$ROOT_DIR/target/debug/libshared.dylib"

    cargo run -p shared --features uniffi-bindgen --bin uniffi-bindgen -- \
        generate --library "$LIB_PATH" \
        --language swift \
        --out-dir "$BINDINGS_DIR"

    echo "  ✓ UniFFI bindings: $BINDINGS_DIR"
fi

# ───────────────────────────────────────────────────
# Step 3: Summary
# ───────────────────────────────────────────────────

echo ""
echo "=== Build Complete ==="

if [ "$BUILD_DEVICE" = true ]; then
    DEVICE_LIB="$ROOT_DIR/target/$DEVICE_TARGET/$PROFILE_DIR/libshared.a"
    if [ -f "$DEVICE_LIB" ]; then
        echo "  Device lib:  $DEVICE_LIB ($(du -h "$DEVICE_LIB" | cut -f1))"
    fi
fi

if [ "$BUILD_SIM" = true ]; then
    SIM_LIB="$ROOT_DIR/target/$SIM_TARGET/$PROFILE_DIR/libshared.a"
    if [ -f "$SIM_LIB" ]; then
        echo "  Sim lib:     $SIM_LIB ($(du -h "$SIM_LIB" | cut -f1))"
    fi
fi

echo "  Types:       $GENERATED_DIR/SharedTypes/"
echo "  Serde:       $GENERATED_DIR/Serde/"
echo "  Bindings:    $BINDINGS_DIR"
echo ""
echo "Next: Open ios/Intrada.xcodeproj in Xcode or run 'xcodegen' to regenerate the project."
