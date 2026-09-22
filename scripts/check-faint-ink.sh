#!/usr/bin/env bash
# inkFaint is 2.45:1 on paper, below the AA floor for text, so it is kept to
# the section eyebrow (#1941): 67 uses had drifted onto metadata and body text,
# and a pixel snapshot cannot see contrast. Text takes inkSecondary and glyphs
# inkFaintIcon; the Eyebrow view in SectionHeader.swift is the one reader.
#
# A `//` inside a string literal is read as a comment, so a use after one on
# the same line slips through: deliberate, not tracked.
#
# The root is overridable so the self-test can point it at fixtures
# (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

root="${FAINT_INK_ROOT:-ios/Intrada}"

if [ ! -d "$root" ]; then
  echo "✗ check-faint-ink: no $root to read" >&2
  exit 2
fi

hits=$(find "$root" -name '*.swift' \
  ! -path "$root/DesignSystem/Theme.swift" \
  ! -path "$root/Views/Components/SectionHeader.swift" -print0 |
  xargs -0 perl -ne 's{//.*}{}; print "$ARGV:$.: $_" if /\binkFaint(?![A-Za-z])/; close ARGV if eof')

if [ -n "$hits" ]; then
  echo "✗ inkFaint on something other than an eyebrow (#1941):" >&2
  printf '%s' "$hits" | sed 's/^/    /' >&2
  echo "  Text takes inkSecondary, a glyph inkFaintIcon, a section label Eyebrow." >&2
  exit 1
fi
