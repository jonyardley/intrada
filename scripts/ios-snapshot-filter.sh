#!/usr/bin/env bash
# Checks an `ios-snapshots-record` filter against the test sources before the
# recipe deletes references or builds (#2988): `-only-testing:` matching
# nothing builds for two minutes, runs nothing and fails without saying why.
#
# Usage: ios-snapshot-filter.sh <filter> [tests-dir]
# The directory override is for scripts/tests/ios-snapshot-filter-test.sh.
set -euo pipefail

filter="${1:-}"
dir="${2:-ios/IntradaTests}"

reject() {
  echo "✗ no test matched $filter; use Class or Class/testMethod: exact names, no target prefix, no partial method names" >&2
  local hint="${filter##*/}" near=""
  case "$hint" in "" | *[!A-Za-z0-9_]*) exit 1 ;; esac
  near="$(grep -rhoE "(class|func) [A-Za-z0-9_]*${hint}[A-Za-z0-9_]*" --include='*.swift' "$dir" 2>/dev/null |
    awk '{print $2}' | sort -u | head -5 | paste -sd' ' - || true)"
  [ -z "$near" ] || echo "  close: $near" >&2
  exit 1
}

case "$filter" in
  "" | */*/* | /* | */) reject ;;
esac
class="${filter%%/*}"
method=""
[ "$class" = "$filter" ] || method="${filter#*/}"
case "$class$method" in *[!A-Za-z0-9_]*) reject ;; esac

# Methods can sit in an extension in another file, so every file naming the
# class counts.
files="$(grep -rlE "^[[:space:]]*(final |public |open |internal )*(class|extension) ${class}([^A-Za-z0-9_]|$)" \
  --include='*.swift' "$dir" 2>/dev/null || true)"
declared="$(grep -lE "^[[:space:]]*(final |public |open |internal )*class ${class}([^A-Za-z0-9_]|$)" \
  $files /dev/null 2>/dev/null || true)"
[ -n "$declared" ] || reject
[ -n "$method" ] || exit 0
if ! grep -lE "func ${method}\(" $files /dev/null >/dev/null 2>&1; then
  reject
fi
