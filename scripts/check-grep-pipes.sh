#!/usr/bin/env bash
# Under pipefail, a pipe into grep -q reads a match as a miss: grep exits on
# its first match and the writer still writing dies of a closed pipe (#2847).
# A here-string has no writer to kill, so grep -q pattern <<<"$text" is the
# shape. Every .sh under scripts/ is read, sourced helpers included, since a
# file sourced from a pipefail recipe runs under it too.
#
# The root is overridable so the self-test can point it at fixtures
# (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

root="${GREP_PIPES_ROOT:-scripts}"
if [ ! -d "$root" ]; then
  echo "✗ check-grep-pipes: no $root to read" >&2
  exit 2
fi

hits=$(find "$root" -name '*.sh' -type f -print0 | sort -z | xargs -0 perl -ne '
  BEGIN { $buf = "" }
  chomp;
  unless ($buf eq "" && /^\s*#/) {
    $at = $. if $buf eq "";
    if (!eof && (s/\\\s*$// || /\|\s*$/)) {
      $buf .= "$_ ";
    } else {
      $buf .= $_;
      print "$ARGV:$at: $buf\n"
        if $buf =~ /(?<!\|)\|(?!\|)&?\s*(?:command\s+|!\s*)?[ef]?grep(?:\s+-\S+)*?\s+(?:-[A-Za-z]*q[A-Za-z]*|--quiet|--silent)(?=\s|;|$)/;
      $buf = "";
    }
  }
  close ARGV if eof
')
if [ -n "$hits" ]; then
  echo "✗ a pipe into grep -q, which under pipefail can read a match as a miss (#2847):" >&2
  printf '%s\n' "$hits" | sed 's/^/    /' >&2
  echo "  Feed grep a here-string: grep -q pattern <<<\"\$text\", or <<<\"\$(command)\"." >&2
  exit 1
fi
