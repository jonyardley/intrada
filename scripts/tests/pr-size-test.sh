#!/usr/bin/env bash
# Self-test for scripts/check-pr-size.sh (#2877): the limit, the left-out
# paths, and which descriptions count as a Size section.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/check-pr-size.sh"
fail=0

expect() {
  local label="$1" want="$2" files="$3" body="$4" code=0
  printf '%b' "$files" | PR_BODY="$body" bash "$script" >/dev/null 2>&1 || code=$?
  if [ "$code" = "$want" ]; then echo "ok: $label"; else echo "FAIL: $label (exit $code)"; fail=1; fi
}

big='900\tcrates/intrada-core/src/lib.rs\n'
expect "under the limit passes" 0 '800\tsrc/a.rs\n' ""
expect "over the limit with no description fails" 1 "$big" ""
expect "left-out paths do not count" 0 '700\tsrc/a.rs\n5000\tCargo.lock\n900\tandroid/app/gradle.lockfile\n300\tios/IntradaTests/__Snapshots__/T/a.txt\n300\tandroid/app/src/test/snapshots/a.png\n400\tcrates/intrada-store/tests/fixtures/a.sql\n900\tdesign/sketch-kit/components/kit.js\n' ""
expect "a Size section with text passes" 0 "$big" $'## What this fixes\n\nx\n\n## Size\n\nOne generated table.\n'
expect "a CRLF Size section with text passes" 0 "$big" $'## Size\r\n\r\nOne generated table.\r\n'
expect "an empty Size section fails" 1 "$big" $'## Size\n\n## Checklist\n\n- [ ] x\n'
expect "a comment-only Size section fails" 1 "$big" $'## Size\n\n<!-- why it is big -->\n'
expect "a Size heading inside other text fails" 1 "$big" $'## Sizes and shapes\n\ntext\n'

exit "$fail"
