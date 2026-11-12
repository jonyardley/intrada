#!/usr/bin/env bash
# Self-test for scripts/crew-park.sh (#2819): plans and diffs shaped like real
# ones, each exit path, and a stand-in rules file to prove the list is read at
# run time rather than copied into the script.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/crew-park.sh"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
fails=0
n=0

expect() {
  local want=$1 name=$2 needle=$3 out rc=0
  shift 3
  n=$((n + 1))
  out=$("$@" 2>&1) || rc=$?
  if [ "$rc" -ne "$want" ] || [[ "$out" != *"$needle"* ]]; then
    echo "✗ $name: wanted exit $want with '$needle', got (exit $rc): $out" >&2
    fails=$((fails + 1))
  fi
}
park() { bash "$script" "$work/$1"; }
plan() { printf '%s\n' "$2" >"$work/$1"; }

plan ffi-dir '**Files**: `crates/intrada-ffi/`, ios/Intrada/Views/LibraryView.swift'
plan ffi-bare '**Files**: crates/intrada-ffi, as #2803 lists it.'
plan ffi-file '**Files**: crates/intrada-ffi/src/lib.rs.'
plan session '**Files**: crates/intrada-core/src/domain/session/active.rs'
plan generated '**Files**: ios/generated/SharedTypes.swift; android/generated/uniffi.kt'
plan store './crates/intrada-store/src/migrations.rs'
plan bridge-name '**Files**: CoreBridge.kt and the views that read it'
plan store-swift '**Files**: ios/Intrada/Core/SharedItemStore.swift'
plan plain '**Files**: `docs/roadmap.md`, ios/Intrada/Views/LibraryView.swift, crates/intrada-core/src/app.rs'
plan near-miss '**Files**: crates/intrada-ffi-tools/src/main.rs, ios/generatedish/x.swift, crates/intrada-storefront'
plan empty ''
plan abbrev-ffi '**Files**: intrada-ffi/src/lib.rs'
plan abbrev-session '**Files**: `domain/session/active.rs`'
plan crate-prose 'Decisions: the intrada-store migrations stay as they are.'
plan abbrev-store-swift '**Files**: Core/SharedItemStore.swift'
plan wider-glob '**Files**: every file under crates/**'
plan abbrev-miss '**Files**: session/active.rs, src/lib.rs, intrada-core/src/app.rs, ios/Intrada/Views/*.swift, **Tests**: none'
printf 'diff --git a/docs/x.md b/docs/x.md\n--- a/docs/x.md\n+++ b/docs/x.md\n@@ -1 +1 @@\n+A plan naming crates/intrada-ffi/ exits 3.\n' >"$work/diff-mentions"
printf 'diff --git a/crates/intrada-ffi/src/lib.rs b/crates/intrada-ffi/src/lib.rs\nindex 1..2 100644\n--- a/crates/intrada-ffi/src/lib.rs\n+++ b/crates/intrada-ffi/src/lib.rs\n' >"$work/diff-ffi"
printf 'diff --git a/x b/crates/intrada-store/src/lib.rs\nnew file mode 100644\n--- /dev/null\n+++ b/crates/intrada-store/src/lib.rs\n' >"$work/diff-new-store"

expect 3 ffi-dir "crates/intrada-ffi/ matches crates/intrada-ffi/**" park ffi-dir
expect 3 ffi-bare "crates/intrada-ffi matches crates/intrada-ffi/**" park ffi-bare
expect 3 ffi-file "crates/intrada-ffi/src/lib.rs matches" park ffi-file
expect 3 session "matches crates/intrada-core/src/domain/session/**" park session
expect 3 generated-ios "ios/generated/SharedTypes.swift matches ios/generated/**" park generated
expect 3 generated-android "android/generated/uniffi.kt matches android/generated/**" park generated
expect 3 store "crates/intrada-store/src/migrations.rs matches" park store
expect 3 bridge-name "CoreBridge.kt matches android/app/src/main/kotlin/com/intrada/android/core/CoreBridge.kt" park bridge-name
expect 3 store-swift "ios/Intrada/Core/SharedItemStore.swift matches" park store-swift
expect 0 plain "" park plain
expect 3 abbrev-ffi "intrada-ffi/src/lib.rs matches crates/intrada-ffi/**" park abbrev-ffi
expect 3 abbrev-session "domain/session/active.rs matches crates/intrada-core/src/domain/session/**" park abbrev-session
expect 3 crate-prose "intrada-store matches crates/intrada-store/**" park crate-prose
expect 3 abbrev-store-swift "Core/SharedItemStore.swift matches" park abbrev-store-swift
expect 3 wider-glob "crates/** matches crates/intrada-ffi/**" park wider-glob
expect 0 abbrev-miss "" park abbrev-miss
expect 0 near-miss "" park near-miss
expect 0 empty "" park empty
expect 0 diff-mentions "" park diff-mentions
expect 3 diff-ffi "crates/intrada-ffi/src/lib.rs matches" park diff-ffi
expect 3 diff-new-store "crates/intrada-store/src/lib.rs matches" park diff-new-store

n=$((n + 1))
if [ "$(park generated 2>&1 | grep -c '^park: ')" -ne 2 ]; then
  echo "✗ every match printed: wanted 2 lines for two sensitive paths" >&2
  fails=$((fails + 1))
fi

printf -- '---\npaths:\n  - "docs/**"\n---\n\n# Stand-in\n' >"$work/rules.md"
expect 3 rules-read-at-run-time "docs/roadmap.md matches docs/**" env CREW_PARK_RULES="$work/rules.md" bash "$script" "$work/plain"
expect 0 rules-replaced "" env CREW_PARK_RULES="$work/rules.md" bash "$script" "$work/ffi-dir"
printf '# No frontmatter\npaths:\n  - "docs/**"\n' >"$work/no-front.md"
expect 2 rules-no-frontmatter "no paths: list" env CREW_PARK_RULES="$work/no-front.md" bash "$script" "$work/plain"
printf -- '---\ndescription: x\n---\npaths:\n  - "docs/**"\n' >"$work/after-front.md"
expect 2 rules-paths-after-frontmatter "no paths: list" env CREW_PARK_RULES="$work/after-front.md" bash "$script" "$work/plain"
printf -- '---\npaths:\n  - "docs/**"\nname: x\n  - "crates/**"\n---\n' >"$work/two-keys.md"
expect 0 rules-list-ends-at-next-key "" env CREW_PARK_RULES="$work/two-keys.md" bash "$script" "$work/ffi-file"
expect 2 rules-missing "no paths: list" env CREW_PARK_RULES="$work/nowhere.md" bash "$script" "$work/plain"
expect 2 file-missing "cannot read" bash "$script" "$work/nowhere"
expect 2 no-argument "usage" bash "$script"

if [ "$fails" -ne 0 ]; then
  echo "✗ crew-park: $fails of $n cases wrong" >&2
  exit 1
fi
echo "✓ crew-park: $n cases, parking each sensitive path and passing the rest"
