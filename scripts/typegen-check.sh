#!/usr/bin/env bash
set -euo pipefail

# Check that generated Swift types are up to date.
# Used by both CI and `just typegen-check`.
# Exits 0 if types are fresh, 1 if they need regeneration.

GENERATED_DIR="crates/shared_types/generated"

# Snapshot current generated output
BEFORE=$(find "$GENERATED_DIR" -name '*.swift' -exec md5sum {} + 2>/dev/null | sort || true)

# Regenerate
cargo build -p shared_types

# Compare
AFTER=$(find "$GENERATED_DIR" -name '*.swift' -exec md5sum {} + 2>/dev/null | sort || true)

if [ "$BEFORE" != "$AFTER" ]; then
    echo "❌ Generated Swift types are out of date!"
    echo "   Run 'just typegen' and commit the changes."
    exit 1
fi

echo "✓ Generated Swift types are up to date."
