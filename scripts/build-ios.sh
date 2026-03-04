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
TYPES_DIR="$GENERATED_DIR/SharedTypes"
BINDINGS_DIR="$GENERATED_DIR/UniFFI"

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
# Step 2: Generate Swift type bindings via codegen CLI
# ───────────────────────────────────────────────────

if [ "$GENERATE_TYPES" = true ]; then
    echo ""
    echo "→ Generating Swift type bindings..."

    # Build the codegen binary (host target)
    cargo build -p shared --features codegen

    # Generate SharedTypes (Event, Effect, ViewModel, domain types)
    mkdir -p "$TYPES_DIR"
    cargo run -p shared --features codegen -- \
        --language swift \
        --output-dir "$TYPES_DIR"
    echo "  ✓ Swift types generated: $TYPES_DIR"

    # Generate UniFFI bindings (CoreFFI interface)
    mkdir -p "$BINDINGS_DIR"

    # Use uniffi-bindgen to generate Swift bindings from the compiled library
    if [ "$BUILD_SIM" = true ]; then
        LIB_PATH="$ROOT_DIR/target/$SIM_TARGET/$PROFILE_DIR/libshared.dylib"
    elif [ "$BUILD_DEVICE" = true ]; then
        LIB_PATH="$ROOT_DIR/target/$DEVICE_TARGET/$PROFILE_DIR/libshared.dylib"
    else
        # Types-only mode — build for host to generate bindings
        cargo build -p shared --features uniffi
        LIB_PATH="$ROOT_DIR/target/$PROFILE_DIR/libshared.dylib"
    fi

    # Generate the UniFFI Swift bindings
    cargo run -p shared --features codegen -- \
        generate uniffi swift \
        --library-path "$LIB_PATH" \
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

echo "  Types:       $TYPES_DIR"
echo "  Bindings:    $BINDINGS_DIR"
echo ""
echo "Next: Open ios/Intrada.xcodeproj in Xcode or run 'xcodegen' to regenerate the project."
