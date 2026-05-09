#!/usr/bin/env bash
# Vendor the Sentry browser SDK locally.
#
# Why
# ───
# `browser.sentry-cdn.com` is on common content-blocker lists (uBlock
# Origin, Brave Shields, Pi-hole, AdGuard). Loading the SDK from the CDN
# means users with any of those get `window.Sentry` undefined, which
# silently disables every observability hook in `js_bridge.rs`. Self-
# hosting from our own origin (myintrada.com) bypasses URL-based
# blocklists.
#
# What
# ────
# Downloads the pinned Sentry SDK bundle into the static dir that
# Trunk copies into `dist/`. Run this whenever you bump the version.
#
# How
# ───
#   ./scripts/vendor-sentry-sdk.sh
#
# Or with a specific version:
#
#   SENTRY_SDK_VERSION=10.52.0 ./scripts/vendor-sentry-sdk.sh
#
# After running, also update the version number in the comment in
# `crates/intrada-web/index.html` so future readers know what's
# vendored.

set -euo pipefail

# Pin to the version currently shipped in production. Bump deliberately
# — newer versions may add behaviour we haven't reviewed.
SENTRY_SDK_VERSION="${SENTRY_SDK_VERSION:-10.51.0}"
SENTRY_SDK_BUNDLE="bundle.tracing.min.js"
DEST_DIR="crates/intrada-web/static/sentry"
DEST_FILE="$DEST_DIR/$SENTRY_SDK_BUNDLE"

URL="https://browser.sentry-cdn.com/${SENTRY_SDK_VERSION}/${SENTRY_SDK_BUNDLE}"

mkdir -p "$DEST_DIR"
echo "Downloading Sentry SDK v${SENTRY_SDK_VERSION}…"
curl -sLf "$URL" -o "$DEST_FILE"

# Quick sanity check: file should start with the Sentry banner comment.
if ! head -c 200 "$DEST_FILE" | grep -q "@sentry/browser"; then
    echo "❌ Downloaded file doesn't look like the Sentry SDK — aborting."
    rm -f "$DEST_FILE"
    exit 1
fi

printf '✓ Vendored %s (%s)\n' "$DEST_FILE" "$(wc -c < "$DEST_FILE" | awk '{printf "%.1fKB", $1/1024}')"
printf '  Version : %s\n' "$SENTRY_SDK_VERSION"
printf '\n'
printf 'Remember to commit the vendored bundle and update the comment\n'
printf 'in crates/intrada-web/index.html if the version changed.\n'
printf '\n'
printf 'No SRI integrity attribute is needed — the bundle is served from\n'
printf 'the same origin as index.html, so the same TLS termination + same\n'
printf 'deploy pipeline already gates trust. Adding integrity would just\n'
printf 'double-bookkeep without adding security.\n'
