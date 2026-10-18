#!/usr/bin/env bash
# The code is the design kit (#2698), so a screen built from a sketch must take
# its colours and spacing from the theme: a typed value keeps every other check
# green while the app drifts apart (#2699). Each shell's theme file is the one
# place a raw colour or spacing number may appear; zero is not a value.
#
# What counts as raw:
#   Swift   Color(red:, Color(hex:, Color(white:, Color(.sRGB, UIColor(red:,
#           .init(red:; .padding(N) and .padding(.edge, N); spacing: N
#   Kotlin  Color(0x, Color(red =; N.dp inside padding(, PaddingValues(,
#           spacedBy( and Spacer(Modifier.height( or .width(
# A size (a frame, a stroke, a height off a Spacer) is not spacing and stays
# literal. A value wrapped over lines, or after a `//` in a string, slips
# through: deliberate, not tracked.
#
# The allowlist froze the uses found on 2026-10-18 and only shrinks: one entry
# covers every match of that text in that file, and an entry that matches
# nothing fails until it is removed.
#
# The roots and the list are overridable so the self-test can point them at
# fixtures (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail
export LC_ALL=C

cd "$(git rev-parse --show-toplevel)"

root="${THEME_VALUES_ROOT:-ios/Intrada}"
android_root="${THEME_VALUES_ANDROID_ROOT:-android/app/src/main/kotlin}"
allowlist="${THEME_VALUES_ALLOWLIST:-scripts/theme-values-allowlist.txt}"

for path in "$root" "$android_root" "$allowlist"; do
  if [ ! -e "$path" ]; then
    echo "✗ check-theme-values: no $path to read" >&2
    exit 2
  fi
done

# One line per raw value: path, the matched text, line number and the line.
hits=$(find "$root" -name '*.swift' \
  ! -path "$root/DesignSystem/Theme.swift" -print0 |
  cat - <(find "$android_root" -name '*.kt' ! -path '*/ui/Theme.kt' -print0) |
  xargs -0 perl -ne '
    s{//.*}{};
    my $line = $_; chomp $line; $line =~ s/^\s+//;
    my @found;
    my $num = qr/(\d+(?:\.\d+)?)/;
    if ($ARGV =~ /\.swift$/) {
      push @found, $1 while /(\b(?:UI)?Color\(\s*(?:\.sRGB|\.displayP3|red:|hex:|white:|hue:)|\.init\(\s*(?:red|hex|white|hue):)/g;
      while (/(\.padding\((?:\s*(?:\.\w+|\[[^\]]*\])\s*,)?\s*$num\s*\))/g) { push @found, $1 if $2 != 0 }
      while (/(\bspacing:\s*$num)/g) { push @found, $1 if $2 != 0 }
    } else {
      push @found, $1 while /(\bColor\(\s*(?:0x|red\s*=))/g;
      while (/(\b(?:padding|PaddingValues|spacedBy)\([^()]*\))/g) {
        my $call = $1;
        push @found, $call if grep { $_ != 0 } $call =~ /$num\.dp/g;
      }
      while (/(\bSpacer\(\s*(?:modifier\s*=\s*)?Modifier\.(?:height|width)\(\s*$num\.dp\s*\))/g) { push @found, $1 if $2 != 0 }
    }
    for (@found) { (my $s = $_) =~ s/\s+/ /g; print "$ARGV\t$s\t$.\t$line\n" }
    close ARGV if eof
  ')

allowed=$(sed -e 's/#.*//' -e 's/[[:space:]]*$//' "$allowlist" | grep -v '^$' | sort -u || true)
keys=$(printf '%s\n' "$hits" | awk -F'\t' 'NF { print $1 " " $2 }' | sort -u)

new=$(awk -F'\t' 'NR == FNR { ok[$0] = 1; next }
  NF && !(($1 " " $2) in ok) { print $1 ":" $3 ": " $4 }' \
  <(printf '%s\n' "$allowed") <(printf '%s\n' "$hits"))
stale=$(comm -23 <(printf '%s\n' "$allowed") <(printf '%s\n' "$keys") | grep -v '^$' || true)

status=0
if [ -n "$new" ]; then
  echo "✗ a raw colour or spacing number outside the theme (#2699):" >&2
  printf '%s\n' "$new" | sed 's/^/    /' >&2
  echo "  Use an IntradaColor or IntradaSpacing token; a missing token goes into both theme files first." >&2
  status=1
fi
if [ -n "$stale" ]; then
  echo "✗ $allowlist lists entries it no longer needs; remove them:" >&2
  printf '%s\n' "$stale" | sed 's/^/    /' >&2
  status=1
fi
exit $status
