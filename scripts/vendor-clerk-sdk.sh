#!/usr/bin/env bash
# Vendor the Clerk browser SDK locally.
#
# Why
# ───
# `cdn.jsdelivr.net` runtime loads block every Clerk-dependent action
# (sign-in, getToken, isSignedIn) on first visit. On slow mobile networks
# this is 5-15s — particularly painful on iOS, where the OAuth flow runs
# in Safari via ASWebAuthenticationSession and pays the same CDN tax
# every sign-in (#580). Self-hosting from myintrada.com lets the browser
# fetch the SDK in parallel with the WASM bundle from the same origin.
#
# What
# ────
# Downloads the pinned Clerk SDK bundle into the static dir that Trunk
# copies into `dist/`. Run this whenever you bump the version.
#
# How
# ───
#   ./scripts/vendor-clerk-sdk.sh
#
# Or with a specific version:
#
#   CLERK_SDK_VERSION=5.125.10 ./scripts/vendor-clerk-sdk.sh
#
# After running, also update the version number in the comment in
# `crates/intrada-web/index.html` so future readers know what's
# vendored.

set -euo pipefail

# Pin to the version currently shipped in production. Bump deliberately
# — newer versions may add behaviour we haven't reviewed.
CLERK_SDK_VERSION="${CLERK_SDK_VERSION:-5.125.10}"
CLERK_SDK_BUNDLE="clerk.browser.js"
DEST_DIR="crates/intrada-web/static/clerk"
DEST_FILE="$DEST_DIR/$CLERK_SDK_BUNDLE"

URL="https://cdn.jsdelivr.net/npm/@clerk/clerk-js@${CLERK_SDK_VERSION}/dist/${CLERK_SDK_BUNDLE}"

mkdir -p "$DEST_DIR"
echo "Downloading Clerk SDK v${CLERK_SDK_VERSION}…"
curl -sLf "$URL" -o "$DEST_FILE"

# Clerk-specific token from the minified bundle, not just "clerk" anywhere.
if ! head -c 8192 "$DEST_FILE" | grep -q "__clerk_modal_state"; then
    echo "❌ Downloaded file doesn't look like the Clerk SDK — aborting."
    rm -f "$DEST_FILE"
    exit 1
fi

printf '✓ Vendored %s (%s)\n' "$DEST_FILE" "$(wc -c < "$DEST_FILE" | awk '{printf "%.1fKB", $1/1024}')"
printf '  Version : %s\n' "$CLERK_SDK_VERSION"
