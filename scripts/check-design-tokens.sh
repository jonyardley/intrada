#!/usr/bin/env bash
# Theme.swift is the token source and the design system page only shows it
# (#2603), so every Swift colour must appear on the page. A colour counts as
# shown when its hex is anywhere on the page: the marker pastels and hero
# gradients live in swatches, not in :root. A :root property named after the
# token (--butter is marker, --track is masteryTrack) must carry the same hex.
#
# Hex only: a token moved onto a colour the page already shows slips through.
#
# The paths are overridable so the self-test can point them at fixtures
# (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

theme="${DESIGN_TOKENS_THEME:-ios/Intrada/DesignSystem/Theme.swift}"
page="${DESIGN_TOKENS_PAGE:-design/intrada-design-system.dc.html}"

for file in "$theme" "$page"; do
  if [ ! -f "$file" ]; then
    echo "✗ check-design-tokens: no $file to read" >&2
    exit 2
  fi
done

misses=$(perl -e '
  my ($theme, $page) = @ARGV;
  my %alias = (butter => "marker", track => "masteryTrack");

  open my $p, "<", $page or die "$page: $!";
  my $html = do { local $/; <$p> };
  my %on_page = map { uc($_) => 1 } $html =~ /#([0-9A-Fa-f]{6})\b/g;
  my %root;
  if ($html =~ /:root\s*\{(.*?)\}/s) {
    my $block = $1;
    while ($block =~ /--([\w-]+)\s*:\s*#([0-9A-Fa-f]{6})\b/g) {
      $root{ $alias{$1} // $1 } = uc $2;
    }
  }

  open my $t, "<", $theme or die "$theme: $!";
  my $func = "";
  while (<$t>) {
    $func = $1 if /static func (\w+)\(/;
    my ($name, $hex);
    if (/static let (\w+) = Color\(hex: 0x([0-9A-Fa-f]{6})\)/) {
      ($name, $hex) = ($1, uc $2);
    } elsif (/case \.(\w+): Color\(hex: 0x([0-9A-Fa-f]{6})\)/) {
      ($name, $hex) = ("$func(.$1)", uc $2);
    } else {
      next;
    }
    if (exists $root{$name} && $root{$name} ne $hex) {
      print "$name #$hex (the page has #$root{$name})\n";
    } elsif (!$on_page{$hex}) {
      print "$name #$hex\n";
    }
  }
' "$theme" "$page")

if [ -n "$misses" ]; then
  echo "✗ Theme.swift colours missing from the design system page (#2603):" >&2
  printf '%s\n' "$misses" | sed 's/^/    /' >&2
  echo "  Show each on $page: a :root property and a table row, or a swatch." >&2
  exit 1
fi
