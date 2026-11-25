#!/usr/bin/env bash
# Self-test for scripts/ios-snapshot-filter.sh (#2988): the filter shapes that
# match a snapshot test pass, and the ones `-only-testing:` would silently run
# nothing for fail before the two-minute build, saying why.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/ios-snapshot-filter.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cat >"$tmp/FocusPlayerSnapshotTests.swift" <<'SWIFT'
final class FocusPlayerSnapshotTests: SnapshotTestCase {
  func testPlaying() {}
  func testPlayingOnIPad() {}
  // func testRetired() {}
  private func orientationBand() {}
}
SWIFT
cat >"$tmp/SnapshotTestCase.swift" <<'SWIFT'
class SnapshotTestCase: XCTestCase {
  func assertSnapshot() {}
}
SWIFT
cat >"$tmp/ActorSnapshotTests.swift" <<'SWIFT'
@MainActor final class ActorSnapshotTests: SnapshotTestCase {
  @MainActor func testOnMain() async {}
}
SWIFT
cat >"$tmp/LibrarySnapshotTests.swift" <<'SWIFT'
final class LibrarySnapshotTests: XCTestCase {
  func testEmpty() throws {}
}
SWIFT

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

run() {
  local status=0
  bash "$script" "$1" "$tmp" 2>"$tmp/err" || status=$?
  echo "$status"
}

said() {
  [ "$(head -1 "$tmp/err")" = "✗ no test matched $1; use Class or Class/testMethod: exact names, no target prefix, no partial method names" ] &&
    echo said || echo silent
}

expect "a bare class passes" 0 "$(run FocusPlayerSnapshotTests)"
expect "a class and its exact method pass" 0 "$(run FocusPlayerSnapshotTests/testPlaying)"
expect "a method declared with throws passes" 0 "$(run LibrarySnapshotTests/testEmpty)"

expect "a target-prefixed class fails" 1 "$(run IntradaTests/FocusPlayerSnapshotTests)"
expect "a target-prefixed class says why" said "$(said IntradaTests/FocusPlayerSnapshotTests)"
expect "a target-prefixed class names the class it meant" \
  "  close: FocusPlayerSnapshotTests" "$(sed -n 2p "$tmp/err")"
expect "a partial method name fails" 1 "$(run FocusPlayerSnapshotTests/testPlay)"
expect "a partial method name says why" said "$(said FocusPlayerSnapshotTests/testPlay)"
expect "an unknown class fails" 1 "$(run PracticeSnapshotTests)"
expect "a partial class name fails" 1 "$(run FocusPlayer)"
expect "a method on the wrong class fails" 1 "$(run LibrarySnapshotTests/testPlaying)"
expect "an empty filter fails" 1 "$(run '')"
expect "a class after an attribute passes" 0 "$(run ActorSnapshotTests)"
expect "a method after an attribute passes" 0 "$(run ActorSnapshotTests/testOnMain)"

expect "a helper method not named test fails" 1 "$(run FocusPlayerSnapshotTests/orientationBand)"
expect "a commented-out test method fails" 1 "$(run FocusPlayerSnapshotTests/testRetired)"
expect "a class with no test methods fails" 1 "$(run SnapshotTestCase)"
expect "a trailing slash fails" 1 "$(run FocusPlayerSnapshotTests/)"
expect "a leading slash fails" 1 "$(run /FocusPlayerSnapshotTests)"
expect "a target, class and method fails" 1 "$(run IntradaTests/FocusPlayerSnapshotTests/testPlaying)"

echo "ios-snapshot-filter-test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
