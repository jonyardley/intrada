#!/usr/bin/env bash
# Every haptic goes through Haptic in Store+Feedback.swift (#2013), so a change
# to how or when they fire is made once. A success haptic built by hand can
# also fire before the core confirms, which #1937 closed. SwiftUI's
# sensoryFeedback modifier is allowed for .selection only: a selection tick
# follows no send, so it has nothing to wait for.
#
# Kotlin follows the same rule through ui/Haptic.kt (#2488): no feedback type
# is named and no view haptic is performed anywhere else.
#
# The root is overridable so the self-test can point it at fixtures
# (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

root="${HAPTICS_ROOT:-ios/Intrada}"
android_root="${ANDROID_HAPTICS_ROOT:-android/app/src/main/kotlin/com/intrada/android}"

if [ ! -d "$root" ]; then
  echo "✗ check-haptics: no $root to read" >&2
  exit 2
fi

hits=$(find "$root" -name '*.swift' ! -path "$root/Core/Store+Feedback.swift" -print0 |
  xargs -0 perl -ne 's{//.*}{}; print "$ARGV:$.: $_" if /FeedbackGenerator\b/ || /sensoryFeedback\s*\((?!\s*\.selection\b)/; close ARGV if eof')

if [ -n "$hits" ]; then
  echo "✗ a haptic built outside Store+Feedback.swift (#2013):" >&2
  printf '%s\n' "$hits" | sed 's/^/    /' >&2
  echo "  Use store.send(_:onSuccess:) after a send, or Haptic.<kind>.play() otherwise." >&2
  exit 1
fi

if [ ! -d "$android_root" ]; then
  echo "✗ check-haptics: no $android_root to read" >&2
  exit 2
fi
kotlin_hits=$(find "$android_root" -name '*.kt' ! -path "$android_root/ui/Haptic.kt" -print0 |
  xargs -0 perl -ne 's{//.*}{}; print "$ARGV:$.: $_" if /\bperformHapticFeedback\b/ || /\bHapticFeedbackType\b/ || /\bHapticFeedbackConstants\b/; close ARGV if eof')
if [ -n "$kotlin_hits" ]; then
  echo "✗ a haptic built outside ui/Haptic.kt (#2488):" >&2
  printf '%s\n' "$kotlin_hits" | sed 's/^/    /' >&2
  echo "  Use LocalHapticFeedback.current.play(Haptic.<kind>)." >&2
  exit 1
fi
