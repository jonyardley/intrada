#!/usr/bin/env bash
# Self-test for scripts/ios-failed-tests.py (#2269): which failures a retry may
# rerun, in the shapes real result bundles take, and the shapes where a rerun
# of the failed cases alone would forgive something. A stub `xcrun` serves the
# result bundle's JSON from a fixture directory.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/ios-failed-tests.py"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir "$tmp/bin"
cat >"$tmp/bin/xcrun" <<'EOF'
#!/usr/bin/env bash
cat "$FIXTURE/$4.json"
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

# fixture NAME FAILED_TESTS TREE_JSON
fixture() {
  mkdir -p "$tmp/$1"
  echo "{\"passedTests\": 20, \"failedTests\": $2, \"skippedTests\": 0}" >"$tmp/$1/summary.json"
  echo "{\"devices\": [], \"testPlanConfigurations\": [], \"testNodes\": [$3]}" >"$tmp/$1/tests.json"
}

run() {
  local status=0
  FIXTURE="$tmp/$1" python3 "$script" bundle >"$tmp/out" 2>"$tmp/err" || status=$?
  echo "$status $(tr '\n' ' ' <"$tmp/out")"
}

url=test://com.apple.xcode/Intrada
xctest_case='{"nodeType": "Test Case", "result": "Failed", "name": "testStart()",
  "nodeIdentifier": "PracticeUITests/testStart()",
  "nodeIdentifierURL": "'$url'/IntradaUITests/PracticeUITests/testStart",
  "children": [{"nodeType": "Repetition", "result": "Failed", "children": [
    {"nodeType": "Failure Message", "name": "XCTAssertTrue failed - player.options is on screen",
     "sourceLocation": {"lineNumber": 71}}]}]}'
swift_testing_case='{"nodeType": "Test Case", "result": "Failed", "name": "reads()",
  "nodeIdentifier": "PageReaderTests/reads()",
  "nodeIdentifierURL": "'$url'/IntradaTests/PageReaderTests/reads()",
  "children": [{"nodeType": "Failure Message", "name": "Expectation failed"}]}'
nested_case='{"nodeType": "Test Case", "result": "Failed", "name": "inner()",
  "nodeIdentifier": "Outer/Inner/inner()",
  "nodeIdentifierURL": "'$url'/IntradaTests/Outer/Inner/inner()", "children": []}'
passed_case='{"nodeType": "Test Case", "result": "Passed", "name": "testOther()",
  "nodeIdentifier": "PracticeUITests/testOther()",
  "nodeIdentifierURL": "'$url'/IntradaUITests/PracticeUITests/testOther", "children": []}'
suite() { echo '{"nodeType": "Test Suite", "result": "'"$1"'", "name": "'"$2"'", "children": ['"$3"']}'; }
bundle() { echo '{"nodeType": "Unit test bundle", "result": "Failed", "name": "'"$1"'", "children": ['"$2"']}'; }

fixture xctest 1 "$(bundle IntradaUITests "$(suite Failed PracticeUITests "$xctest_case, $passed_case")")"
expect "an XCTest failure reruns by its bare method name" \
  "0 -only-testing:IntradaUITests/PracticeUITests/testStart " "$(run xctest)"
expect "an XCTest failure prints its message from under the repetition" true \
  "$(grep -q 'player.options is on screen (line 71)' "$tmp/err" && echo true || echo false)"

fixture swift 1 "$(bundle IntradaTests "$(suite Failed PageReaderTests "$swift_testing_case")")"
expect "a Swift Testing failure reruns with its brackets" \
  "0 -only-testing:IntradaTests/PageReaderTests/reads() " "$(run swift)"

fixture nested 1 "$(bundle IntradaTests "$(suite Failed Outer "$(suite Failed Inner "$nested_case")")")"
expect "a nested suite keeps its target and every suite" \
  "0 -only-testing:IntradaTests/Outer/Inner/inner() " "$(run nested)"

fixture both 2 "$(bundle IntradaUITests "$(suite Failed PracticeUITests "$xctest_case"), $(suite Failed PageReaderTests "$swift_testing_case")")"
expect "two failures rerun both" \
  "0 -only-testing:IntradaUITests/PracticeUITests/testStart -only-testing:IntradaTests/PageReaderTests/reads() " \
  "$(run both)"

refused='{"nodeType": "Test Suite", "result": "Failed", "name": "ClickBarUITests", "children": [
  {"nodeType": "Failure Message", "name": "Application failed preflight checks (Busy)"}]}'
fixture refused 1 "$(bundle IntradaUITests "$(suite Failed PracticeUITests "$xctest_case"), $refused")"
expect "a class refused by its clone blocks the rerun of a flake beside it" "1 " "$(run refused)"

bundle_message='{"nodeType": "Failure Message", "name": "The test runner exited before starting"}'
fixture bundle_message 1 "$(bundle IntradaUITests "$(suite Failed PracticeUITests "$xctest_case"), $bundle_message")"
expect "a failure on the bundle itself blocks the rerun of a flake beside it" "1 " "$(run bundle_message)"

fixture silent_suite 1 "$(bundle IntradaUITests "$(suite Failed PracticeUITests "$xctest_case"), $(suite Failed ClickBarUITests "$passed_case")")"
expect "a failed class with no failed test in it blocks the rerun" "1 " "$(run silent_suite)"

fixture undercount 2 "$(bundle IntradaUITests "$(suite Failed PracticeUITests "$xctest_case")")"
expect "more failures reported than found blocks the rerun" "1 " "$(run undercount)"

short='{"nodeType": "Test Case", "result": "Failed", "name": "x()", "nodeIdentifier": "x()",
  "nodeIdentifierURL": "test://com.apple.xcode/Intrada/x", "children": []}'
fixture short 2 "$(bundle IntradaUITests "$(suite Failed PracticeUITests "$xctest_case, $short")")"
expect "a test with no runnable name blocks the whole rerun, not just itself" "1 " "$(run short)"

fixture green 0 "$(bundle IntradaUITests "$(suite Passed PracticeUITests "$passed_case")" | sed 's/"Failed"/"Passed"/')"
expect "a green bundle lists nothing to rerun" "0 " "$(run green)"

echo "ios-failed-tests-test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
