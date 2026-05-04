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
# iOS — Tauri/Leptos shell
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

