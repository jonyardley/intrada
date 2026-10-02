#!/usr/bin/env bash
# Snapshot hygiene guard (see .claude/rules/ios-quality.md).
#
#  1. Orphans: every __Snapshots__/<Class>/<method>.N.png must map to a
#     `func <method>` in ios/IntradaTests/<Class>.swift. A renamed or deleted
#     test leaves a dead PNG that bloats git history forever — fail so it's
#     pruned with the test.
#  2. Size ceiling: a reference over SNAPSHOT_MAX_BYTES is almost always an
#     un-optimized PNG (Xcode writes a redundant all-opaque alpha channel;
#     `just ios-snapshots-optimize` strips it losslessly, ~75% smaller). Fail so
#     it's optimized before it lands, keeping per-record history growth low.
#
# Android references under android/app/src/test/snapshots take the same two
# rules (#2241). Roborazzi writes each to the path its test names, so a
# reference is live when a test still names that path, and the ceiling is the
# default one with no larger buckets until an Android reference earns one.
#
# The roots are overridable so the self-test can point them at fixtures
# (scripts/tests/hygiene-checks-test.sh).
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

ROOT="${SNAPSHOT_IOS_ROOT:-ios/IntradaTests}"
ANDROID_ROOT="${SNAPSHOT_ANDROID_ROOT:-android/app/src/test}"
SNAP_DIR="$ROOT/__Snapshots__"
MAX_BYTES="${SNAPSHOT_MAX_BYTES:-200000}"
# References that stay large as lossless PNG even after `oxipng -o max -Z`:
# smooth gradients (Practice one-tap hero, Focus player radial, the Up next
# hero's card-level reference), dense-control screens (Session summary's
# per-item score-pill rows over the gold gradient toast), and one
# accessibility-size screen whose bill is glyphs rather than gradient (the
# session detail at .accessibility3 fills the frame with large text, and
# `oxipng` leaves it byte-identical). They get a higher bound. Keep this list
# TIGHT: add only a reference proven irreducible, with the reason. Cropping
# does not help the gradients: flat paper costs almost nothing and the gradient
# is the whole bill, which is why a component crop is on the list alongside the
# full screens.
LARGE_MAX_BYTES="${SNAPSHOT_LARGE_MAX_BYTES:-300000}"
is_large() {
  case "$1" in
    testPracticeScreen | testPracticeScreenPopulated | testPracticeScreenPriorities | \
      testPracticeScreenQuietDay | testPracticeScreenSuggestionDismissed | \
      testPracticeScreenSuggestionDismissedPriorities | testPracticeScreenGreeting | \
      testUpNextHeroNeverMarked | testUpNextHeroFilledPlanAccessibilitySize | \
      testUpNextHeroCoral | \
      testFocusPlayerWithReps | testFocusPlayerWithTarget | testFocusPlayerLongSession | \
      testFocusPlayerWithVariations | testFocusPlayerWithVariationsAccessibilitySize | \
      testPracticeSessionDetailAccessibilitySize | \
      testSessionSummaryCompleted | testSessionSummaryWithReflection | \
      testSessionSummaryWithVariations) return 0 ;;
    *) return 1 ;;
  esac
}

# One tier above `is_large`: the Up next hero (#1082) and its filled plan card
# (#57) are the largest unbroken gradients the app draws, each over half a
# full-screen reference, so they clear the 300k bound even fully optimised. Keep this bucket to references that are
# mostly one gradient; anything else belongs in `is_large` or under the default.
XL_MAX_BYTES="${SNAPSHOT_XL_MAX_BYTES:-420000}"
is_xl() {
  case "$1" in
    testPracticeScreenSuggestion | testPracticeScreenSuggestionPriorities | \
      testUpNextHeroFilledPlan) return 0 ;;
    *) return 1 ;;
  esac
}

fail=0
if [ -d "$SNAP_DIR" ]; then
  while IFS= read -r png; do
    cls=$(basename "$(dirname "$png")")
    method=$(basename "$png" | cut -d. -f1)
    swift="$ROOT/$cls.swift"
    if [ ! -f "$swift" ] || ! grep -qE "func[[:space:]]+$method[[:space:]]*\(" "$swift"; then
      echo "::error file=$png::orphan snapshot: no 'func $method' in $swift (delete the PNG or restore the test)"
      fail=1
    fi
    ceiling="$MAX_BYTES"
    is_large "$method" && ceiling="$LARGE_MAX_BYTES"
    is_xl "$method" && ceiling="$XL_MAX_BYTES"
    size=$(wc -c < "$png" | tr -d ' ')
    if [ "$size" -gt "$ceiling" ]; then
      echo "::error file=$png::$size bytes > $ceiling ceiling: run 'just ios-snapshots-optimize' (or raise SNAPSHOT_MAX_BYTES if genuinely large)"
      fail=1
    fi
  done < <(find "$SNAP_DIR" -name '*.png')
else
  echo "no $SNAP_DIR; no iOS references to check"
fi

if [ ! -d "$ANDROID_ROOT" ]; then
  echo "✗ check-snapshots: no $ANDROID_ROOT to read" >&2
  exit 2
fi
ANDROID_SNAP_DIR="$ANDROID_ROOT/snapshots"
if [ -d "$ANDROID_SNAP_DIR" ]; then
  named=$(find "$ANDROID_ROOT" -name '*.kt' -print0 |
    xargs -0 perl -0777 -ne 's{/\*.*?\*/}{}gs; s{//[^\n]*}{}g; print "$1\n" while m{"(?:[^"\n]*/)?(snapshots/[^"\n]+\.png)"}g' | sort -u)
  while IFS= read -r png; do
    ref="${png#"$ANDROID_ROOT"/}"
    if ! grep -qxF "$ref" <<<"$named"; then
      echo "::error file=$png::orphan snapshot: no test under $ANDROID_ROOT names $ref (delete the PNG or restore the test)"
      fail=1
    fi
    size=$(wc -c < "$png" | tr -d ' ')
    if [ "$size" -gt "$MAX_BYTES" ]; then
      echo "::error file=$png::$size bytes > $MAX_BYTES ceiling: run 'oxipng -o max' on it (or raise SNAPSHOT_MAX_BYTES if genuinely large)"
      fail=1
    fi
  done < <(find "$ANDROID_SNAP_DIR" -name '*.png')
else
  echo "no $ANDROID_SNAP_DIR; no Android references to check"
fi

if [ "$fail" -eq 0 ]; then
  echo "ok: all snapshot references map to a test and are within $MAX_BYTES bytes"
fi
exit "$fail"
