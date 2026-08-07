set dotenv-load

# Default: show available commands
default:
    @just --list

# Kills any stale processes first so port conflicts don't serve old builds.
# Start the API dev server
dev:
    #!/usr/bin/env bash
    set -e
    pkill -f "intrada-api" 2>/dev/null || true
    sleep 0.3
    cargo run -p intrada-api

# Start only the API server (alias for `dev`)
dev-api: dev

# Type-check only (no codegen) — fastest feedback for "does it compile?"
check-fast:
    cargo check --workspace

# Run all tests: nextest, same as CI's `test` job.
# Local green must mean CI green: keep these flags in lockstep with ci.yml.
# (CI adds --profile ci for junit output only; assertions are identical.)
# Doc tests dropped (#1198): they compiled three crates to run zero tests —
# revisit if doc tests ever exist.
test:
    cargo nextest run --workspace

# Clippy with -D warnings: same targets as CI's `clippy` job.
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Format check only (what CI's fmt job runs)
fmt-check:
    cargo fmt --all -- --check

# Spell check + unused deps (what CI's Security & hygiene job runs).
# Both tools come from mise.toml (`mise install`) or brew.
hygiene:
    typos
    cargo-shear

# Check everything (fmt → clippy → test → hygiene, cheapest first). Mirrors
# the iOS test-tier green-stamp (#1200): skips on a clean, already-green HEAD
# (#1204). Delete `target/.check-stamp` to force a re-run.
check:
    #!/usr/bin/env bash
    set -euo pipefail
    stamp=target/.check-stamp
    sha="$(git rev-parse HEAD)"
    if [ -z "$(git status --porcelain)" ] && [ -f "$stamp" ] && [ "$(cat "$stamp")" = "$sha" ]; then
        echo "✓ HEAD $sha already green — skipping. Delete $stamp to force a re-run."
        exit 0
    fi
    just fmt-check lint test hygiene
    # Stamp only the exact tree we tested: a green run over uncommitted edits,
    # or one HEAD moved under, says nothing about $sha (#1204).
    if [ -z "$(git status --porcelain)" ] && [ "$(git rev-parse HEAD)" = "$sha" ]; then
        mkdir -p target
        echo "$sha" > "$stamp"
    fi

# Alias for check — catches errors before the 3-min CI roundtrip
pre-push: check

# Full gate: Rust (fmt/clippy/test) + the native iOS unit/snapshot tier.
# Slower — builds the iOS app — so run it before pushing changes under `ios/`.
# Plain `just check` stays Rust-only for fast Rust-only iterations. Runs the
# fast `ios-test` tier only; `ship` and CI additionally gate on
# `ios-test-full` (XCUITests too) before merge — see #1198.
check-all: check ios-test

# Seed development data (API must be running)
seed:
    bash scripts/seed-dev-data.sh

# ─────────────────────────────────────────────
# Worktrees — warm-start bootstrap (#1205)
# ─────────────────────────────────────────────

# New worktree branched from fresh origin/main, seeded from the main
# checkout's warm caches (target/, ios/build/{spm,dd}, ios/generated) via
# APFS clonefile (`cp -Rc`: copy-on-write, near-instant, no duplicated disk
# until files diverge). Cuts the first `just check` / `just ios-test` in a
# fresh worktree from ~5-10 min cold to close to what the main checkout pays
# warm. Refuses a name that sanitises to the same simulator name as an
# existing worktree (the foo/foo.1 collision documented in ios-testing.md).
[group('Worktrees')]
worktree-new name:
    #!/usr/bin/env bash
    set -euo pipefail
    # Directory and sim-name slug both derive from the raw name (matching
    # _ios-test-sim-name's basename-of-checkout basis), so a slash or shell
    # metacharacter would either nest the worktree under a subdirectory the
    # collision scan below can't see, or break out of the surrounding quotes.
    if ! printf '%s' "{{name}}" | grep -qE '^[A-Za-z0-9][A-Za-z0-9_-]*$'; then
        echo "✗ '{{name}}' must be alphanumeric plus '_'/'-' (no slashes) — .claude/worktrees/ stays flat." >&2
        exit 1
    fi

    main_root="$(git rev-parse --path-format=absolute --git-common-dir | xargs dirname)"
    # Always alongside the main checkout's own worktrees, even when this
    # recipe is invoked from inside another worktree — otherwise it nests
    # the new worktree under the current one instead of beside it.
    target="$main_root/.claude/worktrees/{{name}}"

    slug="$(printf '%s' "{{name}}" | tr -c 'A-Za-z0-9_-' '-' | sed 's/-*$//')"
    for dir in "$main_root"/.claude/worktrees/*/; do
        [ -d "$dir" ] || continue
        other="$(basename "$dir" | tr -c 'A-Za-z0-9_-' '-' | sed 's/-*$//')"
        if [ "$other" = "$slug" ]; then
            echo "✗ $(basename "$dir") already sanitises to sim name '$slug' — pick a different name (see docs/ios-testing.md § worktrees)." >&2
            exit 1
        fi
    done

    echo "→ fetching origin and creating worktree at $target…"
    git fetch origin
    git worktree add "$target" -b "{{name}}" origin/main

    echo "→ seeding warm caches from $main_root…"
    seeded=()
    for rel in target ios/build/spm ios/build/dd; do
        src="$main_root/$rel"
        dst="$target/$rel"
        if [ -d "$src" ]; then
            mkdir -p "$(dirname "$dst")"
            if cp -Rc "$src" "$dst" 2>/dev/null; then
                seeded+=("$rel")
            else
                echo "  ⚠ $rel: clonefile copy failed (non-APFS volume?) — worktree still created, will build cold" >&2
                rm -rf "$dst"
            fi
        fi
    done
    # A new branch must earn its own green (#1204) — strip the check-stamp
    # that came along with the target/ clone rather than exclude it up front.
    rm -f "$target/target/.check-stamp"

    # ios/generated only seeds when this worktree's own core source hash
    # matches the stamp being copied — same rule _ios-sync enforces, so a
    # binding for a different core revision never gets treated as fresh.
    stamp="$main_root/ios/generated/.gen-stamp"
    if [ -f "$stamp" ] && [ -d "$main_root/ios/generated" ]; then
        current="$(cd "$target" && just _ios-src-hash)"
        if [ "$(cat "$stamp")" = "$current" ]; then
            mkdir -p "$target/ios"
            if cp -Rc "$main_root/ios/generated" "$target/ios/generated" 2>/dev/null; then
                seeded+=("ios/generated")
            else
                echo "  ⚠ ios/generated: clonefile copy failed (non-APFS volume?) — will build cold" >&2
                rm -rf "$target/ios/generated"
            fi
        else
            echo "  ios/generated skipped — main checkout's bindings don't match this branch's core source"
        fi
    fi

    echo
    echo "✓ worktree ready: $target"
    if [ "${#seeded[@]}" -gt 0 ]; then
        echo "  seeded (warm): ${seeded[*]}"
    else
        echo "  nothing to seed — main checkout has no warm caches yet"
    fi
    stale=()
    for rel in target ios/build/spm ios/build/dd ios/generated; do
        found=0
        for s in "${seeded[@]:-}"; do [ "$s" = "$rel" ] && found=1; done
        [ "$found" = 1 ] || stale+=("$rel")
    done
    [ "${#stale[@]}" -eq 0 ] || echo "  will rebuild cold: ${stale[*]}"
    echo "  cd $target && just check"

# Companion to worktree-new: cleans the worktree's throwaway sim (if any),
# then removes the worktree via git. Run from any checkout; leaves the
# branch itself intact (delete separately once merged).
[group('Worktrees')]
worktree-rm name:
    #!/usr/bin/env bash
    set -euo pipefail
    main_root="$(git rev-parse --path-format=absolute --git-common-dir | xargs dirname)"
    target="$main_root/.claude/worktrees/{{name}}"
    if [ -d "$target" ]; then
        (cd "$target" && just ios-test-sim-clean) || true
    fi
    git worktree remove "$target"
    echo "✓ removed worktree $target"

# ─────────────────────────────────────────────
# Diagnostics & cleanup
# ─────────────────────────────────────────────

# Helps diagnose "Address already in use" errors when a previous dev session
# didn't shut down cleanly. Pair with `dev` / `dev-api`, which already pkill
# stale processes — use this when those scripts can't reach the holder (e.g.
# a foreign process holding the port).
# Show what's listening on the dev ports we use (API).
ports:
    #!/usr/bin/env bash
    for PORT in 3001; do
        echo "Port $PORT:"
        lsof -nP -iTCP:$PORT -sTCP:LISTEN 2>/dev/null || echo "  (free)"
        echo
    done


# ─────────────────────────────────────────────
# iOS — native SwiftUI app (on the Crux core)
# ─────────────────────────────────────────────
# Daily loop: `just ios` (Xcode) or `just ios-run` (headless). Both regenerate
# the Swift bindings ONLY when the core changed, so they stay in sync without
# slowing pure-Swift edits. ios/generated is a build precondition (gitignored,
# regenerated) — never hand-edit it; fix the Rust type and regenerate.
# `--use-cache` (#1202) skips the project rewrite when project.yml is
# unchanged; verified it also invalidates on a changed SENTRY_DSN_NATIVE, so
# it's safe on every call site here.

# Open the app in Xcode (regenerates bindings first if the core changed).
[group('iOS')]
ios: _ios-sync
    cd ios && xcodegen generate --use-cache
    xed ios/Intrada.xcodeproj

# Build + launch on a simulator and screenshot (regen if the core changed).
[group('iOS')]
ios-run: _ios-sync
    cd ios && xcodegen generate --use-cache
    bash scripts/ios-run-sim.sh

# Stream the app's logs from the booted simulator, filtered to our subsystem —
# drops the UIKit/keyboard/gesture noise so first-party signal is visible.
# `report(_:)` (Core/Logging.swift) logs swallowed FFI errors here (#846 class).
[group('iOS')]
ios-logs:
    xcrun simctl spawn booted log stream --predicate 'subsystem == "com.intrada.native"'

# Force a full regenerate of both Swift packages + refresh the change-stamp.
[group('iOS')]
ios-gen: ios-typegen (ios-package "debug")
    @mkdir -p ios/generated
    @just _ios-src-hash > ios/generated/.gen-stamp
    @echo "✓ bindings regenerated"

# Prep an on-device PERFORMANCE build: regenerate the Crux core optimized for
# release, then in Xcode select your device and Profile (⌘I). `ios`/`ios-gen`
# build the core in debug (cargo-swift's default) — 10–100× slower in hot paths,
# so misleading for perf work; this rebuilds it with `--release`. ⌘I builds the
# Swift app Release, signs with project.yml's team, and opens Instruments.
# Clears the gen-stamp so the next plain `just ios` rebuilds the debug core
# (never link a release core into a routine debug run).
[group('iOS')]
ios-release: ios-typegen (ios-package "release")
    cd ios && xcodegen generate --use-cache
    rm -f ios/generated/.gen-stamp
    @echo "✓ release core ready — opening Xcode. Select your device, then Product → Profile (⌘I). Next 'just ios' rebuilds the debug core."
    xed ios/Intrada.xcodeproj

# Build a signed Release .ipa and upload it to TestFlight (internal testing).
# Mirrors the release-testflight.yml CI lane for local debugging. Needs Ruby >=3
# (system Ruby 2.6 is too old — use rbenv) + the ASC_*/MATCH_* env set, and a
# one-time `fastlane match appstore` bootstrap. See specs/ios-testflight-cicd.md.
[group('iOS')]
testflight: ios-typegen (ios-package "release")
    cd ios && xcodegen generate --use-cache
    rm -f ios/generated/.gen-stamp
    bundle exec fastlane ios beta

# Losslessly shrink snapshot references — drops Xcode's redundant all-opaque
# alpha channel (keeps pixels + sRGB), ~75% smaller. Run after (re)recording
# snapshots, before committing. CI's Snapshot Hygiene job enforces this.
[group('iOS')]
ios-snapshots-optimize:
    find ios/IntradaTests/__Snapshots__ -name '*.png' -exec oxipng -o max --quiet {} +
    @echo "✓ snapshots optimized — review the git diff and commit"

# Re-record snapshot references for the tests matching `filter`, then optimise
# and re-check in one pass.
#
# Recording is delete-then-run-twice by nature: swift-snapshot-testing writes a
# missing reference and *fails* that run, so the second run is what proves the
# new image. Doing that by hand over the whole suite is the slow part of any UI
# change — five rounds of it is most of what made #1256 Phase B feel long.
#
# `filter` is anything `-only-testing:` accepts, minus the target prefix:
#   just ios-snapshots-record ScreenSnapshotTests/testComposeSheetFirstUse
#   just ios-snapshots-record ScreenSnapshotTests   # the whole class
[group('iOS')]
ios-snapshots-record filter: _ios-sync
    #!/usr/bin/env bash
    set -euo pipefail
    snaps=ios/IntradaTests/__Snapshots__
    # A method filter names one reference; a class filter names all of its own.
    # Newline-delimited rather than an array: macOS ships bash 3.2, no mapfile.
    method="$(basename "{{filter}}")"
    if [ "$method" != "{{filter}}" ]; then
        refs="$(find "$snaps" -name "$method.*.png")"
    else
        refs="$(find "$snaps/$method" -name '*.png' 2>/dev/null || true)"
    fi
    [ -z "$refs" ] || printf '%s\n' "$refs" | tr '\n' '\0' | xargs -0 rm -v
    just _ios-build-for-testing
    # First run writes the references and fails by design; the second is the
    # one whose result means anything.
    just _ios-test-without-building "IntradaTests/{{filter}}" 0 || true
    just _ios-test-without-building "IntradaTests/{{filter}}" 0
    # Only what was just written: `oxipng -o max` over the whole suite costs
    # more than the test run it follows.
    [ -z "$refs" ] || printf '%s\n' "$refs" | tr '\n' '\0' | xargs -0 oxipng -o max --quiet
    just ios-snapshots-check

# Orphan + size-ceiling check on snapshot references (same as CI).
[group('iOS')]
ios-snapshots-check:
    bash scripts/check-snapshots.sh

# ios/generated is excluded from both fmt recipes: generated bindings, never
# hand-edited, so never formatted. Toolchain-bundled swift-format, default config.
# Format the hand-written Swift trees in place (fixes what ios-fmt-check flags).
[group('iOS')]
ios-fmt:
    swift format --in-place --recursive --parallel ios/Intrada ios/IntradaTests ios/IntradaUITests

# Swift formatting check, lint mode (same as CI). Run before pushing ios/** changes.
[group('iOS')]
ios-fmt-check:
    swift format lint --strict --recursive --parallel ios/Intrada ios/IntradaTests ios/IntradaUITests

# Fast tier: IntradaTests only (unit + snapshot) on the pinned iPhone 16 /
# iOS 26.5 sim. Seconds once built — catches wire breaks, codecs, upgrade
# paths. XCUITests are NOT run here: 204 unit/snapshot tests are the local
# signal that matters, the 18 XCUITests add ~2 min plus flake and caught
# nothing locally in the #1194 session (#1198). `ship` and CI additionally
# run `ios-test-full` before merge, so nothing merges without the UI tier.
# Regenerates bindings first if the core changed. The device pin must match
# the recorded snapshot references (renderer-specific).
[group('iOS')]
ios-test: _ios-sync (_ios-test-run "fast")

# Full gate: IntradaTests + IntradaUITests. What `ship` and CI run before
# merge — see #1198.
[group('iOS')]
ios-test-full: _ios-sync (_ios-test-run "full")

# Compile-only Release build (no signing, no tests) — catches `#if DEBUG`-only
# code referenced from a file that itself compiles in Release, which passes
# every Debug-only PR gate and then fails `just testflight` / the release lane
# nobody runs per-PR (#1177). Separate derivedDataPath from the Debug test
# build so it can't disturb the products `_ios-build-for-testing` uploads.
[group('iOS')]
ios-build-release: _ios-sync
    #!/usr/bin/env bash
    set -euo pipefail
    cd ios
    xcodegen generate --use-cache
    xcodebuild build -project Intrada.xcodeproj -scheme Intrada -sdk iphonesimulator \
        -configuration Release -destination "generic/platform=iOS Simulator" \
        -derivedDataPath build/dd-release -clonedSourcePackagesDirPath build/spm -quiet \
        COMPILER_INDEX_STORE_ENABLE=NO CODE_SIGNING_ALLOWED=NO

# Shared build+test body for both tiers. Splits `build-for-testing` from
# `test-without-building` (#1198) so a flake retry or test-only change reruns
# in seconds instead of rebuilding the whole app. Skips the run entirely when
# HEAD is clean and already stamped green at this tier (or better) — kills
# re-verifying an unchanged tree, e.g. re-running a gate a test-runner subagent
# already ran green (#1192). Delete `ios/build/.ios-test-stamp` to force.
[private]
_ios-test-run tier:
    #!/usr/bin/env bash
    set -euo pipefail
    stamp=ios/build/.ios-test-stamp
    sha="$(git rev-parse HEAD)"
    if [ -z "$(git status --porcelain)" ] && [ -f "$stamp" ]; then
        read -r stamped_sha stamped_tier < "$stamp" || true
        if [ "${stamped_sha:-}" = "$sha" ] && { [ "${stamped_tier:-}" = "full" ] || [ "${stamped_tier:-}" = "{{tier}}" ]; }; then
            echo "✓ HEAD $sha already green at tier '$stamped_tier' on a clean tree — skipping. Delete $stamp to force a re-run."
            exit 0
        fi
    fi
    just _ios-test-guard
    just _ios-build-for-testing
    if [ "{{tier}}" = "fast" ]; then
        just _ios-test-without-building IntradaTests 0
    else
        # Relaunch-in-new-process, not in-process: the flake this recovers
        # from kills the runner process (#1203). Fast tier (unit/snapshot,
        # deterministic) stays strict. Full tier runs both targets in one
        # `xcodebuild` call locally, so retry applies to both; CI's fanned-out
        # jobs (#1207) call `_ios-test-without-building` once per target and
        # scope retry to the UI job only.
        just _ios-test-without-building "" 1
    fi
    # Same exact-tree guard as `check` above (#1204).
    if [ -z "$(git status --porcelain)" ] && [ "$(git rev-parse HEAD)" = "$sha" ]; then
        printf '%s %s\n' "$sha" "{{tier}}" > "$stamp"
    fi

# Regenerate the Xcode project and build the test products (app + .xctest
# bundles) for THIS worktree's pinned iPhone 16 / iOS 26.5 sim, without
# running anything. Shared by `_ios-test-run` (local) and CI's
# `native-ios-build` job (#1207) — the xcodebuild invocation lives in exactly
# one place so CI and local dev can't drift apart.
[private]
_ios-build-for-testing:
    #!/usr/bin/env bash
    set -euo pipefail
    cd ios
    xcodegen generate --use-cache
    name="$(just _ios-test-sim-name)"
    udid="$(just _ios-test-sim-udid)"
    [ -n "$udid" ] || udid=$(xcrun simctl create "$name" "iPhone 16" "iOS26.5")
    xcodebuild build-for-testing -project Intrada.xcodeproj -scheme Intrada -sdk iphonesimulator \
        -destination "id=$udid" -derivedDataPath build/dd \
        -clonedSourcePackagesDirPath build/spm -quiet \
        COMPILER_INDEX_STORE_ENABLE=NO CODE_SIGNING_ALLOWED=NO

# Run already-built tests against THIS worktree's sim, without rebuilding.
# `only` is an xcodebuild target name (IntradaTests / IntradaUITests) or ""
# to run everything the built products contain; `retry` is "1" to add the
# relaunch-on-crash flags (#1203), else "0". Shared by `_ios-test-run` (local,
# both targets in one call) and CI's fanned-out `native-ios-test-unit` /
# `native-ios-test-ui` jobs (#1207), which call this once per target so a
# UI-suite crash can't take the unit suite down with it.
[private]
_ios-test-without-building only retry:
    #!/usr/bin/env bash
    set -euo pipefail
    cd ios
    name="$(just _ios-test-sim-name)"
    udid="$(just _ios-test-sim-udid)"
    [ -n "$udid" ] || udid=$(xcrun simctl create "$name" "iPhone 16" "iOS26.5")
    onlyflag=""
    [ -z "{{only}}" ] || onlyflag="-only-testing:{{only}}"
    retryflags=""
    [ "{{retry}}" != "1" ] || retryflags="-retry-tests-on-failure -test-iterations 2 -test-repetition-relaunch-enabled YES"
    xcodebuild test-without-building -project Intrada.xcodeproj -scheme Intrada -sdk iphonesimulator \
        -destination "id=$udid" -derivedDataPath build/dd \
        -clonedSourcePackagesDirPath build/spm -quiet \
        $onlyflag $retryflags \
        COMPILER_INDEX_STORE_ENABLE=NO CODE_SIGNING_ALLOWED=NO

# Refuse to start while another xcodebuild/XCTestAgent is already running
# against THIS checkout — two overlapping full-suite runs in one checkout
# share a simulator and crash each other's XCUITests (#1192). `xcodebuild` is
# invoked with a relative `-derivedDataPath` (from `cd ios`), so it never
# appears in the process's own command line — every checkout's argv is
# identical text. Match on each candidate process's cwd via `lsof` instead;
# parallel worktrees (distinct cwds, and already on distinct sims) are
# unaffected. Warns rather than blocking on uncommitted `crates/` changes: a
# concurrent core edit by another writer can red this gate with a compile
# error unrelated to the diff under test.
[private]
_ios-test-guard:
    #!/usr/bin/env bash
    set -euo pipefail
    here="$(pwd)/ios"
    for pid in $(pgrep -f 'xcodebuild|XCTestAgent' 2>/dev/null || true); do
        # `|| true`: the pid can exit between pgrep and lsof (a routine race,
        # not exotic) — lsof then fails, and under pipefail that failure
        # propagates through the assignment and aborts the whole script.
        cwd="$(lsof -a -p "$pid" -d cwd -Fn 2>/dev/null | sed -n 's/^n//p')" || true
        if [ "$cwd" = "$here" ]; then
            echo "✗ another xcodebuild/XCTestAgent (pid $pid) is already running in this checkout ($here)." >&2
            echo "  Concurrent full-suite runs in one checkout are never intentional (#1192) — wait for it to finish." >&2
            echo "  Check with: pgrep -fl 'xcodebuild|XCTestAgent'" >&2
            exit 1
        fi
    done
    if [ -n "$(git status --porcelain -- crates/ 2>/dev/null)" ]; then
        echo "⚠ uncommitted changes under crates/ — if another writer is mid-edit to intrada-core, a build failure below may be theirs, not this diff's (#1192)." >&2
    fi

# Per-worktree sim name (basename of the checkout, sanitised to simctl-safe
# chars) so parallel worktrees don't share one device. The device model is
# irrelevant to snapshot output — swift-snapshot-testing pins `.iPhone13`; only
# the iOS 26.5 runtime affects the pixels — so any distinct device is safe.
[private]
_ios-test-sim-name:
    @printf 'intrada-test-26-5-%s\n' "$(basename "$(git rev-parse --show-toplevel)" | tr -c 'A-Za-z0-9_-' '-' | sed 's/-*$//')"

# UDID of THIS worktree's snapshot sim, or empty if it doesn't exist yet.
[private]
_ios-test-sim-udid:
    @xcrun simctl list devices --json | python3 -c "import json,sys; d=json.load(sys.stdin)['devices']; print(next((x['udid'] for v in d.values() for x in v if x['name']=='$(just _ios-test-sim-name)'), ''))"

# Delete THIS worktree's snapshot sim (created by `ios-test`). Only ever removes
# the device named for the current worktree — never another worktree's or the
# main checkout's, and never a global reset (see the shared-simulator rule).
[group('iOS')]
ios-test-sim-clean:
    #!/usr/bin/env bash
    set -euo pipefail
    name="$(just _ios-test-sim-name)"
    udid="$(just _ios-test-sim-udid)"
    if [ -n "$udid" ]; then
        # `simctl delete` refuses a booted device, and `ios-test` leaves its sim
        # booted — shut it down first (ignore "already shutdown").
        xcrun simctl shutdown "$udid" 2>/dev/null || true
        xcrun simctl delete "$udid" && echo "✓ deleted $name ($udid)"
    else
        echo "✓ no sim named $name — nothing to clean"
    fi

# Facet typegen → ios/generated/SharedTypes (Event/Effect/ViewModel + bincode).
[group('iOS')]
ios-typegen:
    # Pre-clean so a renamed/removed core type can't leave an orphan Swift file
    # (crux's swift typegen overwrites but never deletes) — keeps typegen in sync.
    rm -rf ios/generated/SharedTypes
    RUST_LOG=info cargo run -p intrada-ffi --bin codegen --features codegen -- --output-dir ios/generated

# cargo-swift → ios/generated/IntradaCoreFFI (CoreFFI + RustFramework.xcframework).
[group('iOS')]
ios-package profile="debug":
    #!/usr/bin/env bash
    set -euo pipefail
    cd crates/intrada-ffi
    if [ "{{profile}}" = "release" ]; then rel="--release"; else rel=""; fi
    cargo swift package --name IntradaCoreFFI --platforms ios --lib-type static --features uniffi $rel --accept-all
    rm -rf ../../ios/generated/IntradaCoreFFI
    mkdir -p ../../ios/generated
    mv IntradaCoreFFI ../../ios/generated/IntradaCoreFFI
    # Requires cargo-swift 0.9.0 (`cargo install cargo-swift --version =0.9.0`):
    # its bundled uniffi-bindgen matches our uniffi=0.29.4 crate's runtime
    # contract; newer cargo-swift crashes the app with a contract mismatch.
    # cargo-swift nests the modulemap+header one level too deep; the
    # xcframework Info.plist declares HeadersPath=Headers, so canImport fails
    # and the FFI types vanish. Move them up (crux counter example's 0.9 fix).
    xcf=../../ios/generated/IntradaCoreFFI/RustFramework.xcframework
    moved=0
    for slice in "$xcf"/*/; do
        hd="$slice/headers"
        if [ -d "$hd/RustFramework" ]; then
            mv "$hd/RustFramework/"* "$hd/"; rmdir "$hd/RustFramework"; moved=1
        fi
    done
    [ "$moved" = 1 ] || echo "⚠️  cargo-swift header layout changed — verify canImport(intrada_ffiFFI)"
    echo "✓ ios/generated/IntradaCoreFFI"

# Regenerate bindings only if intrada-core / intrada-ffi changed since last gen.
[private]
_ios-sync:
    #!/usr/bin/env bash
    set -euo pipefail
    stamp=ios/generated/.gen-stamp
    current=$(just _ios-src-hash)
    if [ ! -d ios/generated/IntradaCoreFFI ] || [ ! -d ios/generated/SharedTypes ] || [ "$(cat "$stamp" 2>/dev/null)" != "$current" ]; then
        echo "↻ core changed (or no bindings) — regenerating…"
        just ios-gen
    else
        echo "✓ bindings up to date"
    fi

[private]
_ios-src-hash:
    @find crates/intrada-core/src crates/intrada-ffi/src crates/intrada-core/Cargo.toml crates/intrada-ffi/Cargo.toml -type f -exec shasum {} \; | shasum | cut -d' ' -f1
