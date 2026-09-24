#!/usr/bin/env bash
# Self-test for scripts/audit-sweep.sh. Builds a throwaway repo with one known
# instance of each probe, runs the sweep twice with a change between, and
# asserts the counts, the exclusions and the comparison table.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/audit-sweep.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

repo="$tmp/repo"
mkdir -p "$repo/crates/intrada-core/src/domain" "$repo/ios/Intrada/DesignSystem" \
  "$repo/ios/IntradaTests/__Snapshots__" "$repo/specs"
cd "$repo"
git init -q -b main
git config user.email test@example.com
git config user.name test
git config commit.gpgsign false

cat >crates/intrada-core/src/lib.rs <<'EOF'
pub const BLOB_VERSION: u32 = 7;
#[allow(dead_code)]
fn live() { let x = Some(1).unwrap(); }
#[cfg(test)]
pub fn helper() {}
fn after_a_gated_item() { Some(2).unwrap(); }
#[cfg(test)]
mod tests {
    #[test]
    fn a() { Some(1).unwrap(); }
    #[test]
    fn b() {}
}
EOF
seq 1 50 | sed 's/^/\/\/ line /' >crates/intrada-core/src/domain/tests.rs
cat >ios/Intrada/View.swift <<'EOF'
let a = Color(red: 1, green: 0, blue: 0)
let b = Text("x").padding(8).font(.system(size: 12))
let c = Button("Go") {}.accessibilityLabel("Go")
let d = try! load()
let e = UserDefaults.standard
let h = FileProtectionType.complete
migrator.registerMigration("v1") {}
let f = URL(string: "https://example.com/path")
// TODO: nothing tracks this
// TODO(#12) this one is tracked
EOF
echo 'let theme = Color(red: 0, green: 0, blue: 0).padding(4)' >ios/Intrada/DesignSystem/Theme.swift
printf '@Test func a() {}\nfunc testB() {}\n' >ios/IntradaTests/T.swift
printf 'png' >ios/IntradaTests/__Snapshots__/x.png
printf '[[package]]\nname = "a"\n\n[[package]]\nname = "b"\n' >Cargo.lock
printf 'See `ios/Intrada/View.swift`, `docs/missing.md` and `ios/generated/`.\n' >CLAUDE.md
echo 'ios/generated/' >.gitignore
touch specs/a.md specs/README.md
git add -A
git commit -q -m base

pass=0
fail=0

check() {
  local haystack="$1" desc="$2" needle="$3"
  if printf '%s' "$haystack" | grep -qF -- "$needle"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "FAIL: $desc (expected to find: $needle)" >&2
  fi
}

check_exact() {
  local actual="$1" desc="$2" expected="$3"
  if [ "$actual" = "$expected" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "FAIL: $desc (expected exactly: $expected, got: $actual)" >&2
  fi
}

first="$(bash "$script" --date 2026-01-01 --out "$tmp/metrics")"
check "$first" "a first run says there is nothing to compare with" "No earlier sweep to compare with."
json="$tmp/metrics/2026-01-01.json"
metric() { jq -r --arg k "$1" '.metrics[$k] | tostring' "$json"; }

check_exact "$(jq -r '.metrics | keys | length' "$json")" "every probe writes its metric" 28
check_exact "$(metric principles.rust_unwrap_outside_tests)" "unwraps in a gated test module are not counted, unwraps below a gated item are" 2
check_exact "$(metric simplicity.rust_core_lines)" "core lines include test files" 63
check_exact "$(metric simplicity.swift_app_lines)" "app lines include DesignSystem" 11
check_exact "$(metric simplicity.rust_largest_file_lines)" "the largest core file is not a test file" 13
check_exact "$(metric simplicity.swift_largest_file_lines)" "the largest app file" 10
check_exact "$(metric simplicity.allow_dead_code)" "allow(dead_code) counts" 1
check_exact "$(metric accessibility.swift_accessibility_modifiers)" "an accessibility label counts" 1
check_exact "$(metric accessibility.swift_buttons)" "a button counts" 1
check_exact "$(metric tests.snapshot_images)" "a snapshot reference counts" 1
check_exact "$(metric data.grdb_migrations)" "a migration counts" 1
check_exact "$(metric security.userdefaults_refs)" "a UserDefaults use counts" 1
check_exact "$(metric security.file_protection_refs)" "a file protection use counts" 1
check_exact "$(metric dependencies.cargo_lock_packages)" "each locked package counts" 2
check_exact "$(metric tests.rust_tests)" "each #[test] counts" 2
check_exact "$(metric data.blob_version)" "the blob version is read from the constant" 7
check_exact "$(metric tests.swift_tests)" "Swift Testing and XCTest both count" 2
check_exact "$(metric consistency.swift_raw_colours)" "a raw colour in DesignSystem is not counted" 1
check_exact "$(metric consistency.swift_numeric_spacing)" "numeric padding in DesignSystem is not counted" 1
check_exact "$(metric accessibility.swift_fixed_font_sizes)" "a fixed font size counts" 1
check_exact "$(metric principles.swift_force_try_or_cast)" "try! counts" 1
check_exact "$(metric simplicity.todo_without_issue)" "a TODO naming an issue is not counted" 1
check_exact "$(metric security.network_hosts)" "one host is found" 1
check_exact "$(metric security.privacy_manifests)" "no privacy manifest reads as zero" 0
check_exact "$(metric dependencies.cargo_deny_advisories)" "no manifest records null, not a clean zero" null
check_exact "$(metric drift.specs)" "the specs README is not a spec" 1
check_exact "$(metric drift.dead_paths)" "a missing path counts and an ignored one does not" 1
check_exact "$(jq -r '.lists.dead_paths[0]' "$json")" "the dead path names its document" "CLAUDE.md: docs/missing.md"
check "$(jq -r '.lists.hotspots_rust | join(",")' "$json")" "the hotspot list names the source file" "crates/intrada-core/src/lib.rs"
check "$(jq -r '.lists.churn | join(",")' "$json")" "the churn list names a changed file" "crates/intrada-core/src/lib.rs"
check_exact "$(jq -r '.lists.hotspots_rust | map(select(contains("tests.rs"))) | length' "$json")" "test files are not hotspots" 0

echo 'let g = Button("Stop") {}' >>ios/Intrada/View.swift
git commit -qam more
second="$(bash "$script" --date 2026-02-01 --out "$tmp/metrics")"
check "$second" "a second run compares with the first" "Against $tmp/metrics/2026-01-01.json:"
check "$second" "a changed metric shows before and now" "| accessibility.swift_buttons | 1 | 2 |"
check_exact "$(jq -r '.metrics["activity.commits_since_last"]' "$tmp/metrics/2026-02-01.json")" \
  "commits are counted from the previous sweep's commit, not its date" 1
if printf '%s' "$second" | grep -qF "| tests.rust_tests |"; then
  fail=$((fail + 1))
  echo "FAIL: an unchanged metric is left out of the table" >&2
else
  pass=$((pass + 1))
fi

git mv ios/Intrada ios/App
git commit -qm moved
third="$(bash "$script" --date 2026-03-01 --out "$tmp/metrics")"
json="$tmp/metrics/2026-03-01.json"
check_exact "$(metric simplicity.swift_app_lines)" "a missing app directory records null, not zero" null
check_exact "$(metric consistency.swift_raw_colours)" "a probe on a missing directory records null" null
check_exact "$(metric security.network_hosts)" "no hosts are claimed for a missing directory" null
check "$third" "a probe that stopped reading shows as skipped, not as an improvement" "| accessibility.swift_buttons | 2 | skipped |"

echo "audit-sweep-test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
