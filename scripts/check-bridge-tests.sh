#!/usr/bin/env bash
# A Rust round trip cannot see Rust and Swift disagreeing about the bincode
# wire, so an edit can look saved and not be (#846). Every type that derives
# Facet for the bridge, and every ViewModel field, must be named in a LiveBridge
# test (#2101); a field only after a dot, so an event case of the same name
# does not count. Naming is not asserting: this catches a forgotten test, not a
# weak one. The allowlist froze the gaps found on 2026-09-30 and only shrinks: an
# entry that is now tested, or no longer exists, fails until it is removed.
#
# A Kotlin round trip through LiveBridge counts the way a Swift one does: it is
# a second decoder of the same wire, and the check asks only that some shell
# test names the type (#2265).
#
# The roots are overridable so the self-test can point them at fixtures
# (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

core="${BRIDGE_CORE_ROOT:-crates}"
model="${BRIDGE_VIEWMODEL:-crates/intrada-core/src/model.rs}"
tests="${BRIDGE_TESTS_ROOT:-ios/IntradaTests}"
android_tests="${BRIDGE_ANDROID_TESTS_ROOT:-android/app/src/test/kotlin}"
allowlist="${BRIDGE_ALLOWLIST:-scripts/bridge-tests-allowlist.txt}"

for path in "$core" "$model" "$tests" "$android_tests" "$allowlist"; do
  if [ ! -e "$path" ]; then
    echo "✗ check-bridge-tests: no $path to read" >&2
    exit 2
  fi
done

types=$(find "$core" -name '*.rs' -print0 | xargs -0 perl -ne '
  if ($attr ne "" || /^\s*#\[/) {
    $attr .= $_;
    if (($attr =~ tr/[//) <= ($attr =~ tr/]//)) {
      $want = 1 if $attr =~ /facet::Facet\b|#\[effect\(facet_typegen\)\]/;
      $attr = "";
    }
  } elsif ($want && !/^\s*(\/\/|$)/) {
    print /^\s*pub\s+(?:struct|enum)\s+(\w+)/ ? "$1\n" : "!$ARGV:$.\n";
    $want = 0;
  }
  if (eof) { close ARGV; $want = 0; $attr = "" }
' | sort -u)

drift=$(grep '^!' <<<"$types" || true)
if [ -n "$drift" ]; then
  echo "✗ check-bridge-tests: a Facet derive not followed by a pub struct or enum; teach the parser the shape:" >&2
  printf '%s\n' "${drift//!/    }" >&2
  exit 2
fi

fields=$(perl -ne '
  $in = 1 if /^pub struct ViewModel\b/;
  next unless $in;
  if (/^\}/) { last }
  if (/^\s*pub\s+(?:r#)?(\w+)\s*:/) { ($f = $1) =~ s/_(\w)/\u$1/g; print "ViewModel.$f\n" }
' "$model" | sort -u)

if [ -z "$types" ] || [ -z "$fields" ]; then
  echo "✗ check-bridge-tests: found no bridge types or no ViewModel fields; the parser has drifted" >&2
  exit 2
fi

swift_files=$(find "$tests" \( -name '*BridgeTests.swift' -o -name 'RowsBridge.swift' \) | sort)
kotlin_files=$(find "$android_tests" -name '*RoundTripTest.kt' | sort)
if [ -z "$swift_files" ] || [ -z "$kotlin_files" ]; then
  echo "✗ check-bridge-tests: no LiveBridge tests under $tests or under $android_tests" >&2
  exit 2
fi
bridge_files=$(printf '%s\n%s\n' "$swift_files" "$kotlin_files")

named=$(printf '%s\n' "$bridge_files" | tr '\n' '\0' |
  xargs -0 perl -ne 's{"(?:[^"\\]|\\.)*"}{""}g; s{//.*}{}; print "$_\n" for /\b([A-Za-z_]\w*)\b/g' | sort -u)
members=$(printf '%s\n' "$bridge_files" | tr '\n' '\0' |
  xargs -0 perl -ne 's{//.*}{}; print "$_\n" for /\.([a-z]\w*)\b(?!\s*\()/g' | sort -u)
allowed=$(sed -e 's/#.*//' -e 's/[[:space:]]*$//' "$allowlist" | grep -v '^$' | sort -u || true)

untested=()
stale=()
while IFS= read -r entry; do
  if [ "$entry" = "${entry#ViewModel.}" ]; then seen=$named; else seen=$members; fi
  if grep -qxF "${entry#ViewModel.}" <<<"$seen"; then
    grep -qxF "$entry" <<<"$allowed" && stale+=("$entry (now tested)")
  elif ! grep -qxF "$entry" <<<"$allowed"; then
    untested+=("$entry")
  fi
done < <(printf '%s\n%s\n' "$types" "$fields")

while IFS= read -r entry; do
  [ -z "$entry" ] && continue
  grep -qxF "$entry" <<<"$types
$fields" || stale+=("$entry (no longer on the bridge)")
done <<<"$allowed"

status=0
if [ ${#untested[@]} -gt 0 ]; then
  echo "✗ on the bridge with no LiveBridge test naming it (#846, #2101):" >&2
  printf '    %s\n' "${untested[@]}" >&2
  echo "  Drive it through RowsBridge in a *BridgeTests.swift file, or LiveBridge in a *RoundTripTest.kt file, and assert on what comes back." >&2
  status=1
fi
if [ ${#stale[@]} -gt 0 ]; then
  echo "✗ $allowlist lists entries it no longer needs; remove them:" >&2
  printf '    %s\n' "${stale[@]}" >&2
  status=1
fi
exit $status
