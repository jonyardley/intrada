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

    # Remove old hand-written SharedTypes.swift (replaced by typegen)
    if [ -f "$GENERATED_DIR/SharedTypes.swift" ]; then
        rm "$GENERATED_DIR/SharedTypes.swift"
        echo "  ✓ Removed old hand-written SharedTypes.swift"
    fi

    # Copy generated SharedTypes into the iOS project
    mkdir -p "$GENERATED_DIR/SharedTypes"
    cp -R "$TYPEGEN_OUTPUT/Sources/SharedTypes/"*.swift "$GENERATED_DIR/SharedTypes/"

    # Strip `import Serde` — generated as a separate module but we compile in a single target
    sed -i '' '/^import Serde$/d' "$GENERATED_DIR/SharedTypes/"*.swift
    echo "  ✓ Copied to: $GENERATED_DIR/SharedTypes/ (stripped import Serde)"

    # Append struct Codable conformances to SharedTypes.swift.
    # Swift 6 requires auto-synthesised Codable to be in the same file as the
    # struct definition. Enum Codable (with explicit implementations) stays in
    # CodableExtensions.swift because cross-file explicit conformance is fine.
    cat >> "$GENERATED_DIR/SharedTypes/SharedTypes.swift" << 'CODABLE'

// MARK: - Codable (appended by build-ios.sh for REST API JSON serialisation)

extension Item: Codable {}
extension Tempo: Codable {}
extension PracticeSession: Codable {}
extension SetlistEntry: Codable {}
extension Routine: Codable {}
extension RoutineEntry: Codable {}
extension Goal: Codable {}
extension ActiveSession: Codable {}
CODABLE
    echo "  ✓ Appended struct Codable conformances to SharedTypes.swift"

    # Copy Serde runtime (BCS serialization helpers)
    mkdir -p "$GENERATED_DIR/Serde"
    cp -R "$TYPEGEN_OUTPUT/Sources/Serde/"*.swift "$GENERATED_DIR/Serde/"
    echo "  ✓ Serde runtime: $GENERATED_DIR/Serde/"

    # Generate UniFFI bindings (CoreFFI + CoreJson interface)
    mkdir -p "$BINDINGS_DIR"

    # Build uniffi-bindgen and generate Swift bindings
    if [ "$BUILD_SIM" = true ]; then
        LIB_PATH="$ROOT_DIR/target/$SIM_TARGET/$PROFILE_DIR/libshared.dylib"
    elif [ "$BUILD_DEVICE" = true ]; then
        LIB_PATH="$ROOT_DIR/target/$DEVICE_TARGET/$PROFILE_DIR/libshared.dylib"
    else
        # Types-only mode — build for host to generate bindings
        cargo build -p shared --features uniffi
        LIB_PATH="$ROOT_DIR/target/$PROFILE_DIR/libshared.dylib"
    fi

    cargo run -p shared --features uniffi-bindgen -- \
        generate --library "$LIB_PATH" \
        --language swift \
        --out-dir "$BINDINGS_DIR" 2>/dev/null || {
        echo "  ⚠ UniFFI binding generation skipped (library not available as dylib)"
        echo "  → Swift bindings will be generated during Xcode build"
    }

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
