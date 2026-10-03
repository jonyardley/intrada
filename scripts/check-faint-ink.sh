#!/usr/bin/env bash
# inkFaint is 2.45:1 on paper, below the AA floor for text (#1941): 67 uses had
# drifted onto metadata and body text, and a pixel snapshot cannot see
# contrast. Text takes inkSecondary and glyphs inkFaintIcon. iOS dropped the
# token with the capitals section label (#1881); the check stays for Android.
#
# A `//` inside a string literal is read as a comment, so a use after one on
# the same line slips through: deliberate, not tracked.
#
# Kotlin too, since the Android theme carries the token (#2309): its theme is
# the one place the name may appear.
#
# The root is overridable so the self-test can point it at fixtures
# (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

root="${FAINT_INK_ROOT:-ios/Intrada}"
android_root="${FAINT_INK_ANDROID_ROOT:-android/app/src/main/kotlin}"

for dir in "$root" "$android_root"; do
  if [ ! -d "$dir" ]; then
    echo "✗ check-faint-ink: no $dir to read" >&2
    exit 2
  fi
done

hits=$(find "$root" -name '*.swift' \
  ! -path "$root/DesignSystem/Theme.swift" -print0 |
  cat - <(find "$android_root" -name '*.kt' ! -path '*/ui/Theme.kt' -print0) |
  xargs -0 perl -ne 's{//.*}{}; print "$ARGV:$.: $_" if /\binkFaint(?![A-Za-z])/; close ARGV if eof')

if [ -n "$hits" ]; then
  echo "✗ inkFaint on text or a glyph (#1941):" >&2
  printf '%s' "$hits" | sed 's/^/    /' >&2
  echo "  Text takes inkSecondary, a glyph inkFaintIcon." >&2
  exit 1
fi
