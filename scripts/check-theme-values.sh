#!/usr/bin/env bash
# The code is the design kit (#2698), so a screen built from a sketch must take
# its colours and spacing from the theme: a typed value keeps every other check
# green while the app drifts apart (#2699). Each shell's theme file is the one
# place a raw colour or spacing number may appear; zero, clear, transparent and
# Compose's Color.Unspecified are not values.
#
# A size (a frame, a stroke, a height on anything but a Spacer) is not spacing
# and stays out of scope. A value wrapped over lines, or after a `//` in a
# string, slips through, and a string that reads like code (`Text("spacing: 6
# notes")`) is caught: deliberate, not tracked.
#
# What the allowlist holds, and how it shrinks: the header of
# scripts/theme-values-allowlist.txt.
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

# Each allowlist line: path, matched text, count and its issue, tab separated.
malformed=$(awk -F'\t' '!/^#/ && NF && !(NF == 4 && $3 ~ /^[1-9][0-9]*$/ && $4 ~ /^#[0-9]+$/)' "$allowlist")
if [ -n "$malformed" ]; then
  echo "✗ check-theme-values: $allowlist lines need a path, the text, a count and an issue, tab separated:" >&2
  printf '%s\n' "$malformed" | sed 's/^/    /' >&2
  exit 2
fi

# One line per raw value: path, the matched text, line number and the line.
hits=$(find "$root" -name '*.swift' \
  ! -path "$root/DesignSystem/Theme.swift" -print0 |
  cat - <(find "$android_root" -name '*.kt' ! -path '*/ui/Theme.kt' -print0) |
  xargs -0 perl -ne '
    s{//.*}{};
    my $line = $_; chomp $line; $line =~ s/^\s+//;
    my @found;
    my $num = qr/(\d+(?:\.\d+)?)/;
    my $args = qr/\((?:[^()]|\([^()]*\))*\)/;
    my $hue = qr/(?:red|orange|yellow|green|mint|teal|cyan|blue|indigo|purple|pink|brown|white|gray|grey|black|primary|secondary|tertiary|quaternary|accentColor)\b/;
    if ($ARGV =~ /\.swift$/) {
      push @found, $1 while /(\b(?:UI)?Color\(\s*(?:\.sRGB|\.displayP3|red:|hex:|white:|hue:)|\.init\(\s*(?:red|hex|white|hue):)/g;
      push @found, $1 while /(\bColor\.$hue|\b(?:foregroundStyle|foregroundColor|tint|fill|stroke|background)\(\s*\.$hue)/g;
      push @found, $1 while /(\.shadow\(\s*color:\s*\.$hue|\bcolors:\s*\[(?:[^\]]*[,\s])?\.$hue)/g;
      while (/(\.padding\((?:\s*(?:\.\w+|\[[^\]]*\])\s*,)?\s*$num\s*\))/g) { push @found, $1 if $2 != 0 }
      while (/(\b(?:spacing|minLength):\s*$num)/g) { push @found, $1 if $2 != 0 }
      while (/((?:\bEdgeInsets|\.padding\(\s*\.init)\([^()]*\))/g) {
        my $call = $1;
        push @found, $call if grep { $_ != 0 } $call =~ /\b(?:top|leading|bottom|trailing):\s*$num/g;
      }
    } else {
      push @found, $1 while /(\bColor\(\s*(?:\d|red\s*=)|\bColor\.(?!Transparent\b|TRANSPARENT\b|Unspecified\b)[A-Z]\w*|\bColor\.hs[lv]\()/g;
      while (/(\b(?:padding|PaddingValues|spacedBy)$args)/g) {
        my $call = $1;
        push @found, $call if grep { $_ != 0 } $call =~ /$num\.dp/g;
      }
      while (/(\bSpacer\(\s*(?:modifier\s*=\s*)?Modifier\.(?:height|width)\(\s*$num\.dp\s*\))/g) { push @found, $1 if $2 != 0 }
    }
    for (@found) { (my $s = $_) =~ s/\s+/ /g; print "$ARGV\t$s\t$.\t$line\n" }
    close ARGV if eof
  ')

# Compare the live count of each path and text with its allowed count.
report=$(awk -F'\t' '
  FILENAME == ARGV[1] { if (!/^#/ && NF) allowed[$1 "\t" $2] = $3; next }
  NF { key = $1 "\t" $2; seen[key]++; where[key] = where[key] "over\t    " $1 ":" $3 ": " $4 "\n" }
  END {
    for (key in seen) {
      n = (key in allowed) ? allowed[key] : 0
      if (seen[key] > n) printf "%s", where[key]
    }
    for (key in allowed) {
      if (seen[key] < allowed[key]) {
        split(key, k, "\t")
        printf "under\t    %s  %s: allowed %d, found %d\n", k[1], k[2], allowed[key], seen[key]
      }
    }
  }' "$allowlist" <(printf '%s\n' "$hits"))
over=$(printf '%s\n' "$report" | grep '^over' | cut -f2- | sort || true)
under=$(printf '%s\n' "$report" | grep '^under' | cut -f2- | sort || true)

status=0
if [ -n "$over" ]; then
  echo "✗ a raw colour or spacing number outside the theme (#2699):" >&2
  printf '%s\n' "$over" >&2
  echo "  Use an IntradaColor or IntradaSpacing token; a value the theme lacks becomes a token in both theme files." >&2
  status=1
fi
if [ -n "$under" ]; then
  echo "✗ $allowlist allows more than the screens now use; lower the count, or remove the entry at 0:" >&2
  printf '%s\n' "$under" >&2
  status=1
fi
exit $status
