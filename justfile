set dotenv-load

[doc("Show the available recipes")]
default:
    @just --list

[doc("Type-check the workspace without codegen, the fastest compile check")]
check-fast:
    cargo check --workspace

# Run all tests: nextest, same as CI's `test` job.
# Local green must mean CI green: keep these flags in lockstep with ci.yml.
# (CI adds --profile ci for junit output only; assertions are identical.)
# Doc tests dropped (#1198): they compiled three crates to run zero tests —
# revisit if doc tests ever exist.
[doc("Run the Rust tests with nextest, as CI's test job does")]
test:
    cargo nextest run --workspace

[doc("Run clippy with warnings as errors, as CI's clippy job does")]
lint:
    cargo clippy --workspace --all-targets -- -D warnings

[doc("Format the Rust code")]
fmt:
    cargo fmt --all

[doc("Check Rust formatting, as CI's fmt job does")]
fmt-check:
    cargo fmt --all -- --check

# MSRV floor check: same as CI's `msrv` job. Reads the floor from Cargo.toml
# rather than pinning it again here, installs that toolchain if it is absent
# (CI does this via dtolnay/rust-toolchain), then checks against it.
# Local green must mean CI green: keep this command in lockstep with ci.yml.
[doc("Check the workspace against the minimum supported Rust version, as CI does")]
msrv:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(grep -oE 'package.rust-version = "[^"]+"' Cargo.toml | cut -d'"' -f2)
    rustup toolchain install "$version" --profile minimal
    cargo +"$version" check --workspace --all-targets

# Coverage report via nextest: same as CI's `coverage` job (minus the
# Codecov upload, which needs a CI-only token).
# Local green must mean CI green: keep these flags in lockstep with ci.yml.
[doc("Write a Rust coverage report to codecov.json, as CI's coverage job does")]
coverage:
    cargo llvm-cov nextest --workspace --codecov --output-path codecov.json

# Spell check + unused deps + workflow lint + markdown links + the Sentry
# release name contract (what CI's Security & hygiene job runs), plus the
# self-test proving pr-visuals.sh still classifies a modified, an added and a
# deleted snapshot reference correctly. The three tools come from mise.toml
# (`mise install`) or brew, cargo-deny and gitleaks from cargo or brew; the
# scripts and self-tests need nothing installed.
# `actionlint` falls back to mise when it is absent from PATH: bare needs
# mise's shims, and a plain shell without `mise activate` would fail the whole
# gate with 127, which reads as broken rather than as a tool that is not
# installed. The link check is diff-scoped, so it reads committed content only.
# The checks below are independent (each self-test sandboxes its own mktemp -d), so they run concurrently.
# A check whose tool is absent prints a line starting "skipped:" and passes;
# those lines are echoed on a green run so a skip never reads as a pass.
[doc("Run the hygiene checks in parallel: spelling, unused deps, workflow lint, links, comment density, dashes, snapshots, cargo-deny, Gitleaks, the script self-tests and the release-name, app-version, faint-ink and haptics checks")]
hygiene:
    #!/usr/bin/env bash
    set -uo pipefail
    actionlint_cmd="actionlint"
    command -v actionlint >/dev/null 2>&1 || actionlint_cmd="mise x -- actionlint"
    checks=(
        "typos:typos"
        "cargo-shear:cargo-shear"
        "actionlint:$actionlint_cmd"
        "check-links:bash scripts/check-links.sh"
        "check-release-name:bash scripts/check-release-name.sh"
        "check-app-version:bash scripts/check-app-version.sh"
        "check-faint-ink:bash scripts/check-faint-ink.sh"
        "check-haptics:bash scripts/check-haptics.sh"
        "comment-density:bash scripts/check-comment-density.sh"
        "dash-check:bash scripts/check-dashes.sh"
        "snapshot-hygiene:bash scripts/check-snapshots.sh"
        "cargo-deny:just deny"
        "gitleaks:just gitleaks"
        "hygiene-checks-test:bash scripts/tests/hygiene-checks-test.sh"
        "pr-visuals-test:bash scripts/tests/pr-visuals-test.sh"
        "status-release-test:bash scripts/tests/status-release-test.sh"
        "status-epics-test:bash scripts/tests/status-epics-test.sh"
        "epic-add-test:bash scripts/tests/epic-add-test.sh"
        "claim-issue-test:bash scripts/tests/claim-issue-test.sh"
        "pr-open-test:bash scripts/tests/pr-open-test.sh"
        "session-claims-test:bash scripts/tests/session-claims-test.sh"
        "handover-test:bash scripts/tests/handover-test.sh"
        "ios-sim-lock-test:bash scripts/tests/ios-sim-lock-test.sh"
        "cmux-gate-test:bash scripts/tests/cmux-gate-test.sh"
        "audit-sweep-test:bash scripts/tests/audit-sweep-test.sh"
    )
    tmpdir=$(mktemp -d) || exit 1
    trap 'rm -rf "$tmpdir"' EXIT
    names=()
    pids=()
    for entry in "${checks[@]}"; do
        name="${entry%%:*}"
        cmd="${entry#*:}"
        names+=("$name")
        bash -c "$cmd" >"$tmpdir/$name.log" 2>&1 &
        pids+=($!)
    done
    fail=0
    for i in "${!pids[@]}"; do
        if ! wait "${pids[$i]}"; then
            fail=1
            echo "✗ ${names[$i]} failed:" >&2
            sed 's/^/    /' "$tmpdir/${names[$i]}.log" >&2
        fi
    done
    if [ "$fail" -eq 0 ]; then
        cat "$tmpdir"/*.log | grep '^skipped:' || true
        echo "✓ hygiene (${#checks[@]} checks, parallel)"
    fi
    exit "$fail"

# Offline, the advisory fetch fails; the retry reads the cached database.
[doc("Check dependencies with cargo-deny, as CI does; skips when it is not installed")]
deny:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v cargo-deny >/dev/null 2>&1; then
        echo "skipped: cargo-deny is not installed (cargo install cargo-deny), CI still runs it"
        exit 0
    fi
    cargo deny --log-level warn --all-features check \
        || cargo deny --offline --log-level warn --all-features check

# Scans only the commits this branch adds, as CI's Gitleaks step does on a PR;
# a scan of the whole history reports historical findings CI never sees.
[doc("Scan this branch's commits for leaked secrets with Gitleaks; skips when it is not installed")]
gitleaks:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v gitleaks >/dev/null 2>&1; then
        echo "skipped: gitleaks is not installed (brew install gitleaks), CI still runs it"
        exit 0
    fi
    if ! git rev-parse -q --verify origin/main >/dev/null; then
        echo "skipped: no origin/main ref to scan from, CI still runs Gitleaks"
        exit 0
    fi
    gitleaks git --no-banner --redact --log-level warn --log-opts="--no-merges --first-parent origin/main..HEAD"

[doc("The audit's mechanical counts, written to docs/audit-metrics and compared with the last sweep (docs/audit.md)")]
audit-sweep *args:
    bash scripts/audit-sweep.sh {{args}}

[doc("Show what is in flight: open PRs, claimed issues, epics and recent merges")]
status:
    ./scripts/generate-status.sh

[doc("Move an issue on the project board, e.g. just project-status 1565 \"In progress\"")]
project-status issue status:
    ./scripts/project-status.sh {{issue}} "{{status}}"

# Before-and-after markdown for every snapshot reference this branch changed,
# ready to paste under "What it looks like" in a PR body (#1631).
[doc("Print before-and-after markdown for the snapshot references this branch changed")]
pr-visuals:
    ./scripts/pr-visuals.sh

# Claim an issue before building it, and refuse if someone already has: the
# in-flight label is set, the newest "Claimed" comment names another branch,
# or an open PR already references it (#1702). Two or more merged PRs already
# referencing the issue refuses a third fix without a named decision:
# just claim 1650 "the approach is wrong because X" (#1890).
[doc("Claim an issue before building it, refusing if someone already has")]
claim number decision="":
    bash scripts/claim-issue.sh "{{number}}" "{{decision}}"

# Print the opener for the next session, so finishing a unit and clearing
# costs a paste rather than a retelling (#1986). The issue number defaults to
# the one in the title of an open PR on this branch.
[doc("Print the opener for the next session")]
handover number="":
    bash scripts/handover.sh "{{number}}"

# Put issues under an epic as GitHub sub-issues, in the working order given:
# just epic-add 1967 1934 1935 (#1968). Refuses an issue another epic holds.
[doc("Put issues under an epic as sub-issues, in working order")]
[positional-arguments]
epic-add parent +children:
    bash scripts/epic-add.sh "$@"

[doc("Move issues under an epic from whichever epic holds them now")]
[positional-arguments]
epic-move parent +children:
    bash scripts/epic-add.sh --move "$@"

# Wrap `gh pr create`, refusing when an issue number in the title has no
# claim naming this branch (#1702): `just pr-open "Title (#42)" "Body text"`.
# just re-splits a variadic parameter on whitespace, which breaks on a title
# containing "(#42)", so title and body are named and quoted; any extra flags
# (e.g. --draft) must be simple, space-free tokens.
[doc("Open a PR, refusing when an issue in the title has no claim for this branch")]
pr-open title body *flags:
    bash scripts/pr-open.sh --title {{quote(title)}} --body {{quote(body)}} {{flags}}

# Check everything (fmt, then lint+test+hygiene overlapped). Mirrors the iOS
# test-tier green-stamp (#1200): skips on a clean, already-green HEAD (#1204).
# Delete `target/.check-stamp` to force a re-run.
[doc("Run the Rust gate: fmt, lint, test and hygiene; skips an unchanged green HEAD")]
check:
    #!/usr/bin/env bash
    set -euo pipefail
    stamp=target/.check-stamp
    sha="$(git rev-parse HEAD)"
    if [ -z "$(git status --porcelain)" ] && [ -f "$stamp" ] && [ "$(cat "$stamp")" = "$sha" ]; then
        echo "✓ HEAD $sha already green — skipping. Delete $stamp to force a re-run."
        exit 0
    fi
    source scripts/lib/cmux-gate.sh
    cmux_gate_start "just check"
    hygiene_log=""
    trap 'status=$?; rm -f ${hygiene_log:+"$hygiene_log"}; cmux_gate_finish "$status"' EXIT
    cmux_gate_step 0.1 "fmt"
    just fmt-check
    # hygiene needs nothing lint/test produce, so it runs alongside them
    # instead of tailing them. Captured rather than left to print live: it
    # often finishes first, and clippy/test output would otherwise bury it.
    hygiene_log=$(mktemp)
    just hygiene >"$hygiene_log" 2>&1 &
    hygiene_pid=$!
    lint_status=0
    cmux_gate_step 0.3 "lint"
    just lint || lint_status=$?
    # Skip test on a lint failure, same fail-fast as the old sequential order.
    test_status=0
    if [ "$lint_status" -eq 0 ]; then
        cmux_gate_step 0.6 "test"
        just test || test_status=$?
    fi
    hygiene_status=0
    cmux_gate_step 0.9 "hygiene"
    wait "$hygiene_pid" || hygiene_status=$?
    cat "$hygiene_log"
    if [ "$lint_status" -ne 0 ]; then exit "$lint_status"; fi
    if [ "$test_status" -ne 0 ]; then exit "$test_status"; fi
    if [ "$hygiene_status" -ne 0 ]; then exit "$hygiene_status"; fi
    # Stamp only the exact tree we tested: a green run over uncommitted edits,
    # or one HEAD moved under, says nothing about $sha (#1204).
    if [ -z "$(git status --porcelain)" ] && [ "$(git rev-parse HEAD)" = "$sha" ]; then
        mkdir -p target
        echo "$sha" > "$stamp"
    fi

[doc("Alias for check")]
pre-push: check

# Rust (fmt/clippy/test) + the native iOS unit/snapshot tier.
# Slower — builds the iOS app — so run it before pushing changes under `ios/`.
# Plain `just check` stays Rust-only for fast Rust-only iterations. Runs the
# fast `ios-test` tier only; CI runs the XCUITests on every PR (#1198, #2114).
[doc("Run check plus the fast iOS tier (unit and snapshot tests, no XCUITests)")]
check-all: check ios-test

# ─────────────────────────────────────────────
# Worktrees — warm-start bootstrap (#1205)
# ─────────────────────────────────────────────

# New worktree branched from fresh origin/main, a thin wrapper for
# worktrunk's `wt switch --create`. Worktrunk's user config sets the path
# (`../intrada-worktrees/<name>`), reflinks the gitignored warm caches
# (target/, ios/build/, ios/generated) from the main checkout, and opens a
# cmux workspace there. This keeps only what is intrada's own: the name rule,
# the simulator-name collision check (the foo/foo.1 collision documented in
# ios-testing.md) and #1204's own-green rule. ios/generated needs no hash
# check here: `_ios-sync` regenerates whenever its stamp does not match.
[doc("Create a worktree from fresh origin/main with worktrunk (wt switch -c)")]
[group('Worktrees')]
worktree-new name:
    #!/usr/bin/env bash
    set -euo pipefail
    # Directory and sim-name slug both derive from the raw name (matching
    # _ios-test-sim-name's basename-of-checkout basis), so a slash or shell
    # metacharacter would either nest the worktree under a subdirectory the
    # collision scan below can't see, or break out of the surrounding quotes.
    if ! printf '%s' "{{name}}" | grep -qE '^[A-Za-z0-9][A-Za-z0-9_-]*$'; then
        echo "✗ '{{name}}' must be alphanumeric plus '_'/'-' (no slashes) — worktree root stays flat." >&2
        exit 1
    fi
    command -v wt >/dev/null || { echo "✗ worktrunk (wt) is not installed: brew install worktrunk" >&2; exit 1; }

    main_root="$(git rev-parse --path-format=absolute --git-common-dir | xargs dirname)"
    worktree_root="$(dirname "$main_root")/intrada-worktrees"
    slug="$(printf '%s' "{{name}}" | tr -c 'A-Za-z0-9_-' '-' | sed 's/-*$//')"
    for dir in "$worktree_root"/*/; do
        [ -d "$dir" ] || continue
        other="$(basename "$dir" | tr -c 'A-Za-z0-9_-' '-' | sed 's/-*$//')"
        if [ "$other" = "$slug" ]; then
            echo "✗ $(basename "$dir") already sanitises to sim name '$slug' — pick a different name (see docs/ios-testing.md § worktrees)." >&2
            exit 1
        fi
    done

    git fetch origin
    wt -C "$main_root" switch --create "{{name}}" --base origin/main

    target="$(git worktree list --porcelain | awk -v b="branch refs/heads/{{name}}" '/^worktree /{p=substr($0, 10)} $0 == b {print p}')"
    # A new branch must earn its own green (#1204): the reflinked target/
    # brings the main checkout's check-stamp with it.
    [ -n "$target" ] && rm -f "$target/target/.check-stamp"
    echo "  cd $target && just check"

# Companion to worktree-new: cleans the worktree's throwaway sim (if any),
# then removes the worktree via git. Run from any checkout; leaves the
# branch itself intact (delete separately once merged).
[doc("Remove a worktree and its snapshot simulator, keeping the branch")]
[group('Worktrees')]
worktree-rm name:
    #!/usr/bin/env bash
    set -euo pipefail
    main_root="$(git rev-parse --path-format=absolute --git-common-dir | xargs dirname)"
    worktree_root="${INTRADA_WORKTREE_ROOT:-$(dirname "$main_root")/intrada-worktrees}"
    target="$worktree_root/{{name}}"
    if [ -d "$target" ]; then
        (cd "$target" && just ios-test-sim-clean) || true
    fi
    git worktree remove "$target"
    echo "✓ removed worktree $target"

# Every worktree of this repo with its branch, uncommitted file count and the
# Claude Code session holding its lease. A worktree sitting at main with no
# commits is not evidence that it is free: uncommitted work looks identical
# from the outside, and only the lease says whether a session is inside.
[doc("List this repo's worktrees with branch, uncommitted files and lease holder")]
[group('Worktrees')]
worktrees:
    #!/usr/bin/env bash
    set -euo pipefail
    lease="$HOME/.claude/hooks/worktree-lease.sh"
    if [ -x "$lease" ]; then
        "$lease" list "$PWD"
    else
        git worktree list
        echo "  (no worktree-lease.sh on this machine, so no session holders shown)"
    fi

# ─────────────────────────────────────────────
# Diagnostics & cleanup
# ─────────────────────────────────────────────

# Claude Code usage from this machine's transcripts, at API prices. Any
# all-digit argument is the day count (default 7); anything else (e.g.
# --quality, --brief) passes straight through to usage-report.py, in either
# order: `just usage 14 --quality` and `just usage --quality 14` both work.
[doc("Report Claude Code usage from this machine's transcripts at API prices")]
usage *args:
    #!/usr/bin/env bash
    set -euo pipefail
    report="$HOME/.claude/hooks/usage-report.py"
    days=7
    flags=()
    for a in {{args}}; do
        case "$a" in
            ''|*[!0-9]*) flags+=("$a") ;;
            *) days="$a" ;;
        esac
    done
    if [ -f "$report" ]; then
        # bash 3.2 (macOS's default) treats "${flags[@]}" as unbound under
        # `set -u` when the array is empty; this expansion is the workaround.
        python3 "$report" --days "$days" ${flags[@]+"${flags[@]}"}
    else
        echo "no usage-report.py in ~/.claude/hooks on this machine, so nothing to read"
    fi

# Neither language server resolved before this existed: `rust-analyzer` on PATH
# is a rustup shim that fails unless the component is installed for the pinned
# toolchain, and `sourcekit-lsp` never activated because its root markers live
# under `ios/`, not at the repo root where a session starts. `buildServer.json`
# fixes the second and is machine-local, so it is gitignored.
#
# `xcode-build-server config` alone is not enough, and its failure is worse
# than silence: it points the index at Xcode's *default* DerivedData, which
# this repo never writes to, so sourcekit-lsp reports `No such module` for
# UIKit and the generated packages. Those are fabricated errors on code that
# compiles, and with `lsp.diagnosticsOnEdit` on, an agent is handed them after
# every Swift edit. Parsing a real indexing build's log instead records
# per-file flags in `.compile` and an `indexStorePath` in `buildServer.json`,
# which is what makes diagnostics honest and jump-to-definition work across
# files. Builds into its own `dd-lsp` so it never disturbs the test products.
#
# `references` still finds nothing: a sourcekit-lsp limitation here, not a
# missing path, so finding callers of a Swift symbol stays a grep job.
# One-time per machine and per worktree that wants Swift.
[doc("Set up rust-analyzer and Swift language server diagnostics, once per machine and worktree")]
lsp-setup: _ios-sync
    #!/usr/bin/env bash
    set -euo pipefail
    rustup component add rust-analyzer
    cd ios
    xcodegen generate
    name="$(just _ios-test-sim-name)"
    udid="$(just _ios-test-sim-udid)"
    [ -n "$udid" ] || udid=$(xcrun simctl create "$name" "iPhone 16" "iOS26.5")
    cd .. && xcode-build-server config -project ios/Intrada.xcodeproj -scheme Intrada
    cd ios
    rm -rf build/dd-lsp
    # No `-quiet`: `parse` reads the compile commands out of the build log.
    xcodebuild build-for-testing -project Intrada.xcodeproj -scheme Intrada -sdk iphonesimulator \
        -destination "id=$udid" -derivedDataPath build/dd-lsp \
        -clonedSourcePackagesDirPath build/spm \
        COMPILER_INDEX_STORE_ENABLE=YES CODE_SIGNING_ALLOWED=NO \
        | (cd .. && xcode-build-server parse)
    echo "✓ rust-analyzer installed; Swift diagnostics, hover and cross-file definition wired"

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

[doc("Open the app in Xcode, regenerating bindings if the core changed")]
[group('iOS')]
ios: _ios-sync
    cd ios && xcodegen generate --use-cache
    xed ios/Intrada.xcodeproj

[doc("Build, launch on a simulator and screenshot, regenerating bindings if the core changed")]
[group('iOS')]
ios-run: _ios-sync
    cd ios && xcodegen generate --use-cache
    bash scripts/ios-run-sim.sh

# Stream the app's logs from the booted simulator, filtered to our subsystem —
# drops the UIKit/keyboard/gesture noise so first-party signal is visible.
# `report(_:)` (Core/Logging.swift) logs swallowed FFI errors here (#846 class).
[doc("Stream the app's own logs from the booted simulator")]
[group('iOS')]
ios-logs:
    xcrun simctl spawn booted log stream --predicate 'subsystem == "com.intrada.native"'

[doc("Force a full regenerate of both Swift packages and refresh the change stamp")]
[group('iOS')]
ios-gen: ios-typegen (ios-package "debug")
    @mkdir -p ios/generated
    @just _ios-src-hash > ios/generated/.gen-stamp
    @echo "✓ bindings regenerated"

# Build a signed Release .ipa and upload it to TestFlight (internal testing).
# Mirrors the release-testflight.yml CI lane for local debugging. Needs Ruby >=3
# (system Ruby 2.6 is too old — use rbenv) + the ASC_*/MATCH_* env set, and a
# one-time `fastlane match appstore` bootstrap. See specs/ios-testflight-cicd.md.
[doc("Build a signed Release app and upload it to TestFlight")]
[group('iOS')]
testflight: ios-typegen (ios-package "release")
    cd ios && xcodegen generate --use-cache
    rm -f ios/generated/.gen-stamp
    bundle exec fastlane ios beta

# Losslessly shrink snapshot references — drops Xcode's redundant all-opaque
# alpha channel (keeps pixels + sRGB), ~75% smaller. Run after (re)recording
# snapshots, before committing. CI's Snapshot Hygiene job enforces this.
[doc("Losslessly shrink snapshot references before committing")]
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
#   just ios-snapshots-record SessionSummarySnapshotTests/testReflectionSheet
#   just ios-snapshots-record SessionSummarySnapshotTests   # the whole class
[doc("Re-record the snapshot references matching a test filter, then optimise and check")]
[group('iOS')]
ios-snapshots-record filter: _ios-sync
    #!/usr/bin/env bash
    set -euo pipefail
    snaps=ios/IntradaTests/__Snapshots__
    # A method filter names one reference; a class filter names all of its own.
    # Newline-delimited rather than an array: macOS ships bash 3.2, no mapfile.
    method="$(basename "{{filter}}")"
    matching() {
        if [ "$method" != "{{filter}}" ]; then
            find "$snaps" -name "$method.*.png"
        else
            find "$snaps/$method" -name '*.png' 2>/dev/null || true
        fi
    }
    refs="$(matching)"
    [ -z "$refs" ] || printf '%s\n' "$refs" | tr '\n' '\0' | xargs -0 rm -v
    source scripts/ios-sim-lock.sh
    ios_sim_lock_acquire_for_run
    _ios_snapshots_record_cleanup() {
        ios_sim_lock_release_with_idle_shutdown "$(just _ios-test-sim-udid 2>/dev/null || true)"
    }
    trap _ios_snapshots_record_cleanup EXIT
    just _ios-build-for-testing
    # First run writes the references and fails by design; the second is the
    # one whose result means anything.
    just _ios-test-without-building "-only-testing:IntradaTests/{{filter}}" 0 || true
    just _ios-test-without-building "-only-testing:IntradaTests/{{filter}}" 0
    # Re-find rather than reusing `$refs`: a first record has nothing to delete,
    # so optimising that list would skip the reference it just wrote and leave
    # `ios-snapshots-check` failing on an un-optimised PNG.
    written="$(matching)"
    [ -z "$written" ] || printf '%s\n' "$written" | tr '\n' '\0' | xargs -0 oxipng -o max --quiet
    just ios-snapshots-check

[doc("Check snapshot references for orphans and size, as CI does")]
[group('iOS')]
ios-snapshots-check:
    bash scripts/check-snapshots.sh

# ios/generated is excluded from both fmt recipes: generated bindings, never
# hand-edited, so never formatted. Toolchain-bundled swift-format, default config.
[doc("Format the hand-written Swift in place")]
[group('iOS')]
ios-fmt:
    swift format --in-place --recursive --parallel ios/Intrada ios/IntradaTests ios/IntradaUITests

[doc("Check Swift formatting, as CI does; run before pushing ios changes")]
[group('iOS')]
ios-fmt-check:
    swift format lint --strict --recursive --parallel ios/Intrada ios/IntradaTests ios/IntradaUITests

# Fast tier: IntradaTests only (unit + snapshot) on the pinned iPhone 16 /
# iOS 26.5 sim. Seconds once built — catches wire breaks, codecs, upgrade
# paths. XCUITests are NOT run here: 204 unit/snapshot tests are the local
# signal that matters, the 18 XCUITests add ~2 min plus flake and caught
# nothing locally in the #1194 session (#1198). CI runs the UI tier on every
# PR, so nothing merges without it (#2114).
# Regenerates bindings first if the core changed. The device pin must match
# the recorded snapshot references (renderer-specific).
[doc("Run the fast iOS tier: unit and snapshot tests, no XCUITests")]
[group('iOS')]
ios-test: _ios-sync (_ios-test-run "fast")

# Full tier: IntradaTests + IntradaUITests, one test at a time. CI runs the UI
# tests on every PR, so `ship` does not run this (#2114).
[doc("Run the full iOS tier with XCUITests, for debugging a UI failure")]
[group('iOS')]
ios-test-full: _ios-sync (_ios-test-run "full")

# Rebuild and run one IntradaUITests class, for verifying a review fix without
# repeating the full UI tier (#1884); pair with `ios-test` (the fast tier).
# Takes the same machine-wide lock as `_ios-test-run` (#1622): this is a UI
# test entry point too, and can run concurrently with another worktree's
# full-tier run.
[doc("Rebuild and run one XCUITest class")]
[group('iOS')]
ios-test-ui-class class: _ios-sync
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/ios-sim-lock.sh
    ios_sim_lock_acquire_for_run
    _ios_test_ui_class_cleanup() {
        ios_sim_lock_release_with_idle_shutdown "$(just _ios-test-sim-udid 2>/dev/null || true)"
    }
    trap _ios_test_ui_class_cleanup EXIT
    just _ios-test-guard
    just _ios-build-for-testing
    just _ios-test-without-building "-only-testing:IntradaUITests/{{class}}" 0

# Compile-only Release build (no signing, no tests) — catches `#if DEBUG`-only
# code referenced from a file that itself compiles in Release, which passes
# every Debug-only PR gate and then fails `just testflight` / the release lane
# nobody runs per-PR (#1177). Separate derivedDataPath from the Debug test
# build so it can't disturb the products `_ios-build-for-testing` uploads.
[doc("Compile a Release build without signing or tests, to catch Debug-only code")]
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
    source scripts/lib/cmux-gate.sh
    cmux_gate_start "just ios-test ({{tier}})"
    trap 'cmux_gate_finish $?' EXIT
    cmux_gate_step 0.05 "waiting for the simulator"
    source scripts/ios-sim-lock.sh
    ios_sim_lock_acquire_for_run
    _ios_test_run_cleanup() {
        local status=$?
        ios_sim_lock_release_with_idle_shutdown "$(just _ios-test-sim-udid 2>/dev/null || true)"
        cmux_gate_finish "$status"
    }
    trap _ios_test_run_cleanup EXIT
    cmux_gate_step 0.15 "build"
    just _ios-test-guard
    just _ios-build-for-testing
    cmux_gate_step 0.5 "test"
    if [ "{{tier}}" = "fast" ]; then
        just _ios-test-without-building -only-testing:IntradaTests 0
    else
        # Relaunch-in-new-process, not in-process: the flake this recovers
        # from kills the runner process (#1203). Fast tier (unit/snapshot,
        # deterministic) stays strict. Full tier runs both targets in one
        # `xcodebuild` call locally, so retry applies to both; CI's fanned-out
        # jobs (#1207) call `_ios-test-without-building` once per slice (unit,
        # then the two UI slices) and scope retry to the UI slices only.
        # Sequential, on the settled source device: a just-booted clone loses
        # SpringBoard's install-placeholder race and refuses the runner as
        # Busy, and the `bootstatus` wait cannot reach a clone because
        # xcodebuild shuts the source device down before cloning (#1480).
        # Costs the UI tier 339s against 86s parallel (2026-09-07).
        just _ios-test-without-building "" 1 0
    fi
    # Same exact-tree guard as `check` above (#1204).
    if [ -z "$(git status --porcelain)" ] && [ "$(git rev-parse HEAD)" = "$sha" ]; then
        printf '%s %s\n' "$sha" "{{tier}}" > "$stamp"
    fi

# Content fingerprint of the ios/ sources the test products are built from:
# tracked blob hashes, plus any uncommitted diff, plus untracked filenames.
# Content rather than mtimes, so a `git pull` or branch switch that restores
# identical bytes is not mistaken for a stale build, and CI's separate build and
# test jobs agree by construction (same commit, clean checkout) with no escape
# hatch. `ios/build`, `ios/generated` and the generated project are gitignored
# and so excluded: CI's test jobs deliberately run without the latter two, and
# `_ios-sync` owns binding freshness in every recipe that builds (#1530).
# Snapshot references are excluded too: a test reads them at run time rather
# than the build compiling them in, so counting them made `ios-snapshots-record`
# fail the guard on the very reference it had just written (#1546).
[private]
_ios-inputs-fingerprint:
    #!/usr/bin/env bash
    set -euo pipefail
    # `ls-tree` rejects an exclude pathspec, so the tracked side is listed as
    # blobs rather than one tree object.
    refs=':(exclude)ios/IntradaTests/__Snapshots__'
    {
        git ls-files -s -- ios "$refs"
        git diff HEAD -- ios "$refs"
        git ls-files -o --exclude-standard ios "$refs"
    } | shasum -a 256 | cut -d' ' -f1

# Regenerate the Xcode project and build the test products (app + .xctest
# bundles) for THIS worktree's pinned iPhone 16 / iOS 26.5 sim, without
# running anything. Shared by `_ios-test-run` (local) and CI's self-hosted
# gate (#1207): the xcodebuild invocation lives in exactly one place so CI and
# local dev can't drift apart.
# No `--use-cache` here: the cache key is project.yml, which a new Swift file
# doesn't change, so a newly added test file silently never joins the target
# and the suite goes green without ever running it (#1456).
[private]
_ios-build-for-testing:
    #!/usr/bin/env bash
    set -euo pipefail
    cd ios
    # A run file is named for its destination, so a pin bump would leave the
    # old one beside the new one and the single-file lookup in
    # `_ios-test-without-building` would have nothing to choose from.
    rm -f build/dd/Build/Products/*.xctestrun
    xcodegen generate
    name="$(just _ios-test-sim-name)"
    udid="$(just _ios-test-sim-udid)"
    [ -n "$udid" ] || udid=$(xcrun simctl create "$name" "iPhone 16" "iOS26.5")
    xcodebuild build-for-testing -project Intrada.xcodeproj -scheme Intrada -sdk iphonesimulator \
        -destination "id=$udid" -derivedDataPath build/dd \
        -clonedSourcePackagesDirPath build/spm -quiet \
        COMPILER_INDEX_STORE_ENABLE=NO CODE_SIGNING_ALLOWED=NO
    # Written last, so it only exists for a build that actually succeeded (#1530).
    just _ios-inputs-fingerprint > build/dd/Build/Products/ios-inputs.sha256

# Run already-built tests against THIS worktree's sim, without rebuilding.
# Driven by the `.xctestrun` the build wrote rather than `-project`/`-scheme`:
# it already names the bundles and their platform.
# `filters` is a space-separated list of xcodebuild `-only-testing:` /
# `-skip-testing:` flags, or "" to run everything the built products contain;
# `retry` is "1" to add the relaunch-on-crash flags (#1203), else "0";
# `parallel` is "1" to clone simulators and run test classes concurrently,
# else "0". Shared by `_ios-test-run` (local, everything in one call),
# `ios-test-ui-class` and `ios-snapshots-record`, and CI's self-hosted gate,
# which runs the unit and UI tiers as separate steps (#1207).
[private]
_ios-test-without-building filters retry parallel="0":
    #!/usr/bin/env bash
    set -euo pipefail
    cd ios
    name="$(just _ios-test-sim-name)"
    udid="$(just _ios-test-sim-udid)"
    [ -n "$udid" ] || udid=$(xcrun simctl create "$name" "iPhone 16" "iOS26.5")
    # Booted here, not by xcodebuild: handed a shut-down device it boots it and
    # installs the app while SpringBoard is still starting, SpringBoard never
    # sees that install finish, and every launch is refused as "Busy" (#1648).
    xcrun simctl bootstatus "$udid" -b
    # With the UI tier sequential the device outlives the run, so the app's
    # UserDefaults would too, and a leftover crash-recovery blob satisfies the
    # resume prompt on its own, hiding a broken save seam (#1480).
    xcrun simctl uninstall "$udid" com.intrada.native >/dev/null 2>&1 || true
    shopt -s nullglob
    runs=(build/dd/Build/Products/*.xctestrun)
    if [ "${#runs[@]}" -ne 1 ]; then
        echo "✗ expected one .xctestrun in ios/build/dd/Build/Products, found ${#runs[@]} — run 'just _ios-build-for-testing' first" >&2
        exit 1
    fi
    # Refuse a build that predates the current ios/ sources. Running the old
    # products means a line you deliberately broke is still intact in the
    # binary, so the test passes, which reads as "this test constrains
    # nothing" and argues for deleting a good test (#1530).
    stamp=build/dd/Build/Products/ios-inputs.sha256
    if [ "$(cat "$stamp" 2>/dev/null || true)" != "$(just _ios-inputs-fingerprint)" ]; then
        echo "✗ ios/ has changed since these test products were built — run 'just _ios-build-for-testing' first." >&2
        echo "  Running them anyway lets a deliberately broken line pass, which is how a good test gets deleted (#1530)." >&2
        exit 1
    fi
    # Cloned simulators are opt-in per caller, defaulting off: a clone's test
    # runner hits "Application failed preflight checks (Busy)" under memory
    # pressure, which reds the gate for no test reason. Six measured on the
    # self-hosted M4 over ten runs (#1824): UI step 175s median against 280s
    # at four, no preflight failures, so the self-hosted CI gate opts in at
    # six. The local full tier stays sequential since #1480. #1642 fixed the
    # rename test that used to silently skip its own field-clearing under
    # clone load and reddened main.
    flags=()
    if [ "{{parallel}}" = "1" ]; then
        flags+=(-parallel-testing-enabled YES -maximum-concurrent-test-simulator-destinations 6)
    else
        flags+=(-parallel-testing-enabled NO)
    fi
    if [ -n "{{filters}}" ]; then
        for f in {{filters}}; do flags+=("$f"); done
    fi
    [ "{{retry}}" != "1" ] || flags+=(-retry-tests-on-failure -test-iterations 2 -test-repetition-relaunch-enabled YES)
    status=0
    # `-collect-test-diagnostics never`: after a refused app launch xcodebuild
    # otherwise spends a fixed 600s gathering a sysdiagnose-style bundle nobody
    # reads, which is why a failed full tier took 12 to 21 minutes (#1480).
    xcodebuild test-without-building -xctestrun "${runs[0]}" \
        -destination "id=$udid" -derivedDataPath build/dd -quiet \
        -collect-test-diagnostics never \
        "${flags[@]}" || status=$?
    # `-quiet` prints nothing on success, so a passing run is indistinguishable
    # from one that never started, and a failing one never says how much of the
    # suite got to run. Report the counts the result bundle holds either way:
    # silence is not evidence, and reading it as "no tests ran" cost a session
    # two bogus issues (#1536, #1537).
    latest="$(ls -td build/dd/Logs/Test/*.xcresult 2>/dev/null | head -1 || true)"
    if [ -z "$latest" ]; then
        echo "✗ no .xcresult under ios/build/dd/Logs/Test: the run produced no result bundle." >&2
        [ "$status" -ne 0 ] || status=1
        exit "$status"
    fi
    # Zero passed is a failure even when xcodebuild is happy: that is the shape
    # an empty run would take.
    if ! xcrun xcresulttool get test-results summary --path "$latest" \
        | python3 -c 'import json,sys; s=json.load(sys.stdin); print("{} passed, {} failed, {} skipped".format(s["passedTests"], s["failedTests"], s["skippedTests"])); sys.exit(1 if s["passedTests"] == 0 else 0)'; then
        [ "$status" -ne 0 ] || status=1
    fi
    exit "$status"

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
[doc("Delete this worktree's snapshot simulator, and only that one")]
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

[doc("Generate the Swift types into ios/generated/SharedTypes with facet")]
[group('iOS')]
ios-typegen:
    # Pre-clean so a renamed/removed core type can't leave an orphan Swift file
    # (crux's swift typegen overwrites but never deletes) — keeps typegen in sync.
    rm -rf ios/generated/SharedTypes
    RUST_LOG=info cargo run -p intrada-ffi --bin codegen --features codegen -- --output-dir ios/generated

[doc("Build the Rust core into ios/generated/IntradaCoreFFI with cargo-swift")]
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
    @find crates/intrada-core/src crates/intrada-ffi/src crates/intrada-core/Cargo.toml crates/intrada-ffi/Cargo.toml Cargo.lock -type f -exec shasum {} \; | shasum | cut -d' ' -f1
