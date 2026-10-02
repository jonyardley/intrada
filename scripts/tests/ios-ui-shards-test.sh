#!/usr/bin/env bash
# Self-test for scripts/ios-ui-shards.py (#2269): how the CI gate deals UI test
# classes across its simulators, using the class durations from the one CI
# result bundle measured on 2026-10-02, and what a run records for the next.
# A stub `xcrun` serves the result bundle's JSON from a fixture file.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/ios-ui-shards.py"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir "$tmp/bin"
cat >"$tmp/bin/xcrun" <<'EOF'
#!/usr/bin/env bash
cat "$FIXTURE"
EOF
chmod +x "$tmp/bin/xcrun"
export PATH="$tmp/bin:$PATH"

pass=0
fail=0
expect() {
  local name="$1" want="$2" got="$3"
  if [ "$want" = "$got" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf '✗ %s\n  want: %s\n  got:  %s\n' "$name" "$want" "$got"
  fi
}

# plan SHARDS SECONDS_FILE CLASS... -> shards joined by " | ", flags stripped
plan() {
  local shards="$1" seconds="$2"
  shift 2
  printf '%s\n' "$@" | python3 "$script" plan "$shards" "$seconds" \
    | sed 's/-only-testing:IntradaUITests\///g' | paste -sd'|' - | sed 's/|/ | /g'
}

measured="$tmp/measured.tsv"
printf 'IntradaUITests/%s\t%s\n' \
  SessionBuilderUITests 96.8 SessionRecoveryUITests 88.2 ProfileUITests 55.0 \
  PracticeSuggestionUITests 41.3 ClickBarUITests 41.1 VariationPickerUITests 33.6 \
  CreateFaultUITests 31.2 LibraryAddExerciseUITests 28.0 VariationManagementUITests 25.4 \
  LibraryDetailAddExerciseUITests 21.3 SessionHistoryUITests 19.1 \
  NativeBackNavigationUITests 9.0 >"$measured"
all=()
while IFS=$'\t' read -r name _; do all+=("$name"); done <"$measured"

expect "the two longest classes start on different simulators, the rest fill the gaps" \
  "SessionBuilderUITests | SessionRecoveryUITests | ProfileUITests LibraryDetailAddExerciseUITests | PracticeSuggestionUITests VariationManagementUITests NativeBackNavigationUITests | ClickBarUITests LibraryAddExerciseUITests | VariationPickerUITests CreateFaultUITests SessionHistoryUITests" \
  "$(plan 6 "$measured" "${all[@]}")"

expect "a class with no measurement weighs the median, not the front of the queue" \
  "SessionBuilderUITests | NewUITests ProfileUITests" \
  "$(plan 2 "$measured" IntradaUITests/SessionBuilderUITests IntradaUITests/ProfileUITests IntradaUITests/NewUITests)"

expect "with nothing measured, every class weighs the same and deals in name order" \
  "AlphaUITests GammaUITests | BetaUITests" \
  "$(plan 2 "$tmp/missing.tsv" IntradaUITests/GammaUITests IntradaUITests/AlphaUITests IntradaUITests/BetaUITests)"

expect "more simulators than classes leaves the spare ones empty" \
  "AlphaUITests |  | " \
  "$(plan 3 "$tmp/missing.tsv" IntradaUITests/AlphaUITests)"

expect "a class listed twice runs once" \
  "AlphaUITests | " \
  "$(plan 2 "$tmp/missing.tsv" IntradaUITests/AlphaUITests IntradaUITests/AlphaUITests)"

printf 'IntradaUITests/AlphaUITests\tnot a number\nIntradaUITests/BetaUITests\t50\n' >"$tmp/garbled.tsv"
expect "an unreadable duration counts as unmeasured" \
  "AlphaUITests | BetaUITests" \
  "$(plan 2 "$tmp/garbled.tsv" IntradaUITests/AlphaUITests IntradaUITests/BetaUITests)"

cat >"$tmp/list.json" <<'EOF'
{"errors": [], "values": [{"disabledTests": [{"identifier": "IntradaUITests/OffUITests/testOff()"}],
  "enabledTests": [{"identifier": "IntradaUITests/ProfileUITests/testEdit()"},
    {"identifier": "IntradaUITests/ProfileUITests/testDefaults()"},
    {"identifier": "IntradaUITests/ClickBarUITests/testBar()"}, {"identifier": "bare"}]}]}
EOF
expect "each class with an enabled test, once, in listing order; disabled and malformed ones dropped" \
  "IntradaUITests/ProfileUITests IntradaUITests/ClickBarUITests" \
  "$(python3 "$script" classes "$tmp/list.json" | paste -sd' ' -)"
expect "the count is every enabled test, for the run to account for" \
  "4" "$(python3 "$script" count "$tmp/list.json")"

url=test://com.apple.xcode/Intrada/IntradaUITests
cat >"$tmp/tests.json" <<EOF
{"testNodes": [{"nodeType": "UI test bundle", "name": "IntradaUITests", "children": [
  {"nodeType": "Test Suite", "name": "AlphaUITests", "children": [
    {"nodeType": "Test Case", "nodeIdentifierURL": "$url/AlphaUITests/testOne", "durationInSeconds": 10.25},
    {"nodeType": "Test Case", "nodeIdentifierURL": "$url/AlphaUITests/testTwo", "durationInSeconds": 5}]},
  {"nodeType": "Test Suite", "name": "GammaUITests", "children": [
    {"nodeType": "Test Case", "nodeIdentifierURL": "$url/GammaUITests/testThree"}]}]}]}
EOF
printf 'IntradaUITests/AlphaUITests\t99.0\nIntradaUITests/BetaUITests\t7.0\n' >"$tmp/recorded.tsv"
FIXTURE="$tmp/tests.json" python3 "$script" record bundle "$tmp/recorded.tsv"
expect "a run replaces the classes it measured, keeps the rest, and skips a case with no duration" \
  "IntradaUITests/AlphaUITests	15.2 IntradaUITests/BetaUITests	7.0" \
  "$(paste -sd' ' "$tmp/recorded.tsv")"

FIXTURE="$tmp/tests.json" python3 "$script" record bundle "$tmp/fresh.tsv"
expect "the first run writes the file" "IntradaUITests/AlphaUITests	15.2" "$(cat "$tmp/fresh.tsv")"

echo "ios-ui-shards-test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
