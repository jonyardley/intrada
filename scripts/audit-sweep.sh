#!/usr/bin/env bash
# The sweep layer of the whole-app audit (docs/audit.md, #2092): counts that
# need no judgement, written to docs/audit-metrics/<date>.json and compared
# with the newest earlier file. A probe whose tool or directory is missing
# records null, so a missing input never reads as a clean result.
#
# Usage: scripts/audit-sweep.sh [--date YYYY-MM-DD] [--out DIR]
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

stamp="$(date +%F)"
out_dir="docs/audit-metrics"
while [ $# -gt 0 ]; do
  case "$1" in
    --date) stamp="$2"; shift 2 ;;
    --out) out_dir="$2"; shift 2 ;;
    *) echo "usage: audit-sweep.sh [--date YYYY-MM-DD] [--out DIR]" >&2; exit 2 ;;
  esac
done

core="crates/intrada-core/src"
app="ios/Intrada"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
metrics="$tmp/metrics.tsv"
: >"$metrics"

put() {
  case "$2" in
    null | [0-9] | [1-9][0-9]*) printf '%s\t%s\n' "$1" "$2" >>"$metrics" ;;
    *) echo "audit-sweep: $1 read '$2', not a count" >&2; exit 1 ;;
  esac
}

list() {
  jq -R -s 'split("\n") | map(select(length > 0))' >"$tmp/list-$1.json"
}

line_count() {
  if [ -d "$1" ]; then
    find "$1" -type f -name "$2" -not -path '*/build/*' -exec cat {} + | wc -l | tr -d ' '
  else
    echo null
  fi
}

largest() {
  if [ -d "$1" ]; then
    find "$1" -type f -name "$2" -not -path '*/build/*' -not -name 'tests.rs' -not -path '*/tests/*' -exec wc -l {} + |
      grep -v ' total$' | sort -rn | head -5 | awk '{print $1 " " $2}'
  fi
}

matches() {
  local pattern="$1" dir="$2" glob="$3" exclude="${4:-build}"
  [ -d "$dir" ] || { echo null; return; }
  local found status=0
  found="$(grep -rEo --include="$glob" --exclude-dir=build --exclude-dir="$exclude" -- "$pattern" "$dir")" || status=$?
  if [ "$status" -gt 1 ]; then
    echo "grep failed ($status)"
  elif [ -z "$found" ]; then
    echo 0
  else
    printf '%s\n' "$found" | wc -l | tr -d ' '
  fi
}

previous=""
if [ -d "$out_dir" ]; then
  previous="$(find "$out_dir" -maxdepth 1 -name '*.json' | LC_ALL=C sort |
    awk -v cur="$out_dir/$stamp.json" '$0 < cur' | tail -1)"
fi
range=(--since="30 days ago" HEAD)
if [ -n "$previous" ]; then
  last_commit="$(jq -r '.commit' "$previous")"
  if git rev-parse --verify -q "$last_commit^{commit}" >/dev/null; then
    range=("$last_commit..HEAD")
  else
    range=(--since="$(jq -r '.date' "$previous") 00:00" HEAD)
  fi
fi

# ── Simplicity ──
put simplicity.rust_core_lines "$(line_count "$core" '*.rs')"
put simplicity.swift_app_lines "$(line_count "$app" '*.swift')"
largest "$core" '*.rs' | list hotspots_rust
largest "$app" '*.swift' | list hotspots_swift
put simplicity.rust_largest_file_lines "$(jq -r '.[0] // "0" | split(" ")[0]' "$tmp/list-hotspots_rust.json")"
put simplicity.swift_largest_file_lines "$(jq -r '.[0] // "0" | split(" ")[0]' "$tmp/list-hotspots_swift.json")"
put simplicity.allow_dead_code "$(matches 'allow\(dead_code\)' crates '*.rs')"
todo=null
if [ -d crates ] && [ -d "$app" ]; then
  todo="$({ grep -rEn --include='*.rs' --include='*.swift' --exclude-dir=build 'TODO|FIXME|HACK' crates "$app" || true; } |
    { grep -vE '\(#[0-9]+\)' || true; } | wc -l | tr -d ' ')"
fi
put simplicity.todo_without_issue "$todo"

# ── Principles ──
# #[cfg(test)] also gates single test-only items above production code, so a
# file is left only at a gated `mod`; test-only helpers are counted, which
# errs towards noise rather than a false zero.
unwraps=null
if [ -d "$core" ]; then
  unwraps="$(find "$core" -type f -name '*.rs' -not -name 'tests.rs' -not -path '*/tests/*' \
    -exec awk 'FNR == 1 { armed = 0 }
      armed && /^[[:space:]]*(pub(\([a-z]+\))? )?mod / { nextfile }
      { armed = /#\[cfg\(test\)\]/; n += gsub(/\.unwrap\(\)/, "") }
      END { print n + 0 }' {} + |
    awk '{ s += $1 } END { print s + 0 }')"
fi
put principles.rust_unwrap_outside_tests "$unwraps"
put principles.swift_force_try_or_cast "$(matches 'try!|as! ' "$app" '*.swift')"

# ── Consistency (Theme.swift owns every token, so DesignSystem is excluded) ──
put consistency.swift_raw_colours "$(matches 'Color\((red|hex|white):|UIColor\(red:|#[0-9A-Fa-f]{6}' "$app" '*.swift' DesignSystem)"
put consistency.swift_numeric_spacing "$(matches '\.padding\((\.[a-zA-Z]+, )?[0-9]|spacing: [0-9]|cornerRadius: [0-9]' "$app" '*.swift' DesignSystem)"

# ── Accessibility ──
put accessibility.swift_fixed_font_sizes "$(matches '\.system\(size:' "$app" '*.swift' DesignSystem)"
put accessibility.swift_accessibility_modifiers "$(matches '\.accessibility(Label|Hint|Value|Hidden|AddTraits|Element|Action)' "$app" '*.swift')"
put accessibility.swift_buttons "$(matches 'Button\(' "$app" '*.swift')"

# ── Tests that bite ──
put tests.rust_tests "$(matches '#\[test\]' crates '*.rs')"
swift_tests=null
for d in ios/IntradaTests ios/IntradaUITests; do
  n="$(matches '@Test|func test[A-Z_]' "$d" '*.swift')"
  case "$n" in
    null) ;;
    [0-9]*) swift_tests=$(( ${swift_tests/null/0} + n )) ;;
    *) put tests.swift_tests "$n" ;;
  esac
done
put tests.swift_tests "$swift_tests"
snapshots=null
[ -d ios ] && snapshots="$(find ios -path '*__Snapshots__*' -name '*.png' -not -path '*/build/*' | wc -l | tr -d ' ')"
put tests.snapshot_images "$snapshots"

# ── Data integrity ──
put data.grdb_migrations "$(matches 'registerMigration' "$app" '*.swift')"
blob="$({ grep -rhE 'const BLOB_VERSION: u32 = [0-9]+' "$core" 2>/dev/null || true; } | grep -oE '[0-9]+;' | tr -d ';' | head -1)"
put data.blob_version "${blob:-null}"

# ── Security and privacy ──
manifests=null
[ -d "$app" ] && manifests="$(find "$app" -name 'PrivacyInfo.xcprivacy' | wc -l | tr -d ' ')"
put security.privacy_manifests "$manifests"
put security.file_protection_refs "$(matches 'FileProtection|fileProtection' "$app" '*.swift')"
put security.userdefaults_refs "$(matches 'UserDefaults' "$app" '*.swift')"
{ [ -d "$app" ] && grep -rhoE --include='*.swift' --exclude-dir=build 'https?://[A-Za-z0-9.-]+' "$app" || true; } |
  LC_ALL=C sort -u | list network_hosts
hosts=null
[ -d "$app" ] && hosts="$(jq length "$tmp/list-network_hosts.json")"
put security.network_hosts "$hosts"
leaks=null
if command -v gitleaks >/dev/null 2>&1 &&
  gitleaks git --no-banner --redact --exit-code 0 --report-format json --report-path "$tmp/leaks.json" . >/dev/null 2>&1; then
  leaks="$(jq length "$tmp/leaks.json")"
fi
put security.gitleaks_history "$leaks"

# ── Dependencies ──
packages=null
[ -f Cargo.lock ] && packages="$(grep -c '^\[\[package\]\]' Cargo.lock || true)"
put dependencies.cargo_lock_packages "$packages"
advisories=null
if command -v cargo-deny >/dev/null 2>&1 && [ -f Cargo.lock ] && [ -f Cargo.toml ]; then
  cargo deny --log-level error check advisories >"$tmp/deny.txt" 2>&1 || true
  if grep -q 'advisories ok' "$tmp/deny.txt"; then
    advisories=0
  elif grep -q '^error\[' "$tmp/deny.txt"; then
    advisories="$(grep -c '^error\[' "$tmp/deny.txt")"
  fi
fi
put dependencies.cargo_deny_advisories "$advisories"

# ── Drift ──
: >"$tmp/dead.txt"
for doc in CLAUDE.md .claude/rules/*.md; do
  [ -f "$doc" ] || continue
  { grep -oE '`[^` ]+`' "$doc" || true; } | tr -d '`' |
    { grep -E '^(crates|ios|docs|specs|scripts|\.claude|\.github)/' || true; } |
    { grep -vE '[*<>{}$]' || true; } | sed -E 's/[:#].*$//' | LC_ALL=C sort -u |
    while IFS= read -r path; do
      [ -e "$path" ] || git check-ignore -q "$path" || echo "$doc: $path" >>"$tmp/dead.txt"
    done
done
list dead_paths <"$tmp/dead.txt"
put drift.dead_paths "$(jq length "$tmp/list-dead_paths.json")"
specs=null
[ -d specs ] && specs="$(find specs -maxdepth 1 -name '*.md' -not -name README.md | wc -l | tr -d ' ')"
put drift.specs "$specs"

# ── Activity since the previous sweep ──
put activity.commits_since_last "$(git rev-list --count "${range[@]}")"
{ git log "${range[@]}" --name-only --pretty=format: -- crates "$app" 2>/dev/null || true; } |
  { grep -v '^$' || true; } | sort | uniq -c | sort -rn | head -5 | awk '{print $1 " " $2}' | list churn

mkdir -p "$out_dir"
out="$out_dir/$stamp.json"
jq -n \
  --arg date "$stamp" \
  --arg commit "$(git rev-parse HEAD)" \
  --slurpfile rows <(jq -R -s 'split("\n") | map(select(length > 0) | split("\t") | {(.[0]): (.[1] | fromjson)}) | add' "$metrics") \
  --slurpfile hr "$tmp/list-hotspots_rust.json" \
  --slurpfile hs "$tmp/list-hotspots_swift.json" \
  --slurpfile ch "$tmp/list-churn.json" \
  --slurpfile nh "$tmp/list-network_hosts.json" \
  --slurpfile dp "$tmp/list-dead_paths.json" \
  '{date: $date, commit: $commit, metrics: $rows[0],
    lists: {hotspots_rust: $hr[0], hotspots_swift: $hs[0], churn: $ch[0], network_hosts: $nh[0], dead_paths: $dp[0]}}' \
  >"$out"

echo "Wrote $out"
if [ -z "$previous" ]; then
  echo "No earlier sweep to compare with."
  jq -r '.metrics | to_entries[] | "  \(.key): \(.value // "skipped")"' "$out"
  exit 0
fi
echo "Against $previous:"
echo
echo "| Metric | Before | Now |"
echo "|---|---|---|"
jq -r --slurpfile prev "$previous" '
  $prev[0].metrics as $was | .metrics | to_entries[]
  | select(.value != $was[.key])
  | .key as $k
  | "| \($k) | \(if $was | has($k) then ($was[$k] // "skipped") else "none" end) | \(.value // "skipped") |"' "$out"
echo
jq -r --slurpfile prev "$previous" '
  $prev[0].metrics as $was | [.metrics | to_entries[] | select(.value == $was[.key])] | length
  | "\(.) metrics unchanged."' "$out"
