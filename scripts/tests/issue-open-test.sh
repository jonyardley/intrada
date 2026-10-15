#!/usr/bin/env bash
# Self-test for scripts/issue-open.sh (#2696): an issue opens only with a
# horizon label and a kind label, and the refusal names the fix.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/issue-open.sh"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/bin"

cat >"$work/bin/gh" <<'FAKE'
#!/usr/bin/env bash
echo "$*" >>"$CALLS"
echo "https://github.com/jonyardley/intrada/issues/1"
FAKE
chmod +x "$work/bin/gh"

export CALLS="$work/calls.log"
export PATH="$work/bin:$PATH"

pass=0
fail=0

check() {
  local name="$1" want_status="$2" want_text="$3"
  shift 3
  rm -f "$CALLS"
  set +e
  out="$("$script" --title "A title" --body "A body" "$@" 2>&1)"
  status=$?
  set -e
  # A refusal must not reach gh; a pass must.
  local opened=1 want_opened=0
  [ -f "$CALLS" ] || opened=0
  [ "$want_status" -eq 0 ] && want_opened=1
  if [ "$status" -eq "$want_status" ] && [ "$opened" -eq "$want_opened" ] &&
    printf '%s' "$out" | grep -qF -- "$want_text"; then
    echo "✓ $name"
    pass=$((pass + 1))
  else
    echo "✗ $name (status $status, opened $opened): $out"
    fail=$((fail + 1))
  fi
}

check "no labels is refused" 1 "no horizon label"
check "a kind without a horizon is refused" 1 "no horizon label" --label ux
check "a horizon without a kind is refused" 1 "no kind label" --label horizon:next
check "an unknown label is not a kind" 1 "no kind label" --label "horizon:now,in-flight"
check "comma separated labels open" 0 "issues/1" --label "horizon:next,ux"
check "repeated label flags open" 0 "issues/1" --label horizon:later --label android
check "an unknown horizon is refused" 1 "no horizon label" --label "horizon:soon,bug"
# A file named like a label must not let a wildcard match it.
touch "$work/horizon:now"
cd "$work"
check "a wildcard is not a horizon" 1 "no horizon label" --label "horizon:*,ux"

# gh matches label names exactly, so the forwarded list must lose its spaces.
rm -f "$CALLS"
"$script" --title "A title" --body "A body" --label "horizon:now, bug" --label " ios" >/dev/null 2>&1
if grep -qF -- "--label horizon:now,bug,ios" "$CALLS"; then
  echo "✓ labels reach gh as one list without spaces"
  pass=$((pass + 1))
else
  echo "✗ labels reach gh as one list without spaces: $(cat "$CALLS" 2>/dev/null)"
  fail=$((fail + 1))
fi

printf '\n%s passed, %s failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
