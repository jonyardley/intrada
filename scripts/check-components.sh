#!/usr/bin/env bash
# Every component in the catalogue index has a page, every page is in the
# index, and each page names a real file, symbol and snapshot on both apps
# (#2620). An Android gap passes only with `not_on_android:` and a reason.
#
# The page proves the component exists and has pictures, not that its words
# are still true; that stays with review.
#
# The paths are overridable so the self-test can point them at fixtures
# (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

root="${COMPONENTS_ROOT:-.}"
catalogue="$root/design/components"
index="$catalogue/README.md"
ios_snaps="$root/ios/IntradaTests/__Snapshots__"
android_snaps="$root/android/app/src/test/snapshots"

# Components listed in the index whose page is still to be written. This list
# only shrinks: a name here that has a page fails the check.
pending="${COMPONENTS_PENDING-TagChip TypeBadge ScoreRing BottomSheet GlobalBanner FormErrorBanner PlaceholderContent ScreenScaffold SectionHeader HairlineDivider SegmentedProgress SectionTitle FieldCard FieldLabel}"

if [ ! -f "$index" ]; then
  echo "✗ check-components: no $index to read" >&2
  exit 2
fi

problems=()

# The first cell of each table row, link or not: `| [Card](Card/README.md) |` or `| TypeBadge |`.
# Each row prints its name and, when linked, the link's target.
rows=$(awk -F'|' '/^\|/ && !/^\|[ :|-]*$/ {
  cell = $2; gsub(/^ +| +$/, "", cell); target = ""
  if (cell ~ /^\[/) {
    target = cell; sub(/^[^(]*\(/, "", target); sub(/\).*$/, "", target)
    sub(/^\[/, "", cell); sub(/\].*$/, "", cell)
  }
  if (cell != "" && cell != "Component") print cell " " target
}' "$index")
names=$(printf '%s\n' "$rows" | cut -d' ' -f1)

is_pending() { case " $pending " in *" $1 "*) return 0 ;; *) return 1 ;; esac; }
in_index() { printf '%s\n' "$names" | grep -qx "$1"; }

# A key from the page's leading `---` block, or nothing.
key() {
  awk -v k="$1" 'NR == 1 && $0 != "---" { exit } NR > 1 && $0 == "---" { exit }
    NR > 1 { if (index($0, k ":") == 1) { sub("^" k ":[ ]*", ""); print; exit } }' "$2"
}

# A Swift or Kotlin declaration of the name, generics and receivers allowed:
# `struct Pills<Option`, `func cardSurface(`, `fun <T> Pills(`, `fun Modifier.cardShadow(`.
declares() {
  printf '(struct|class|enum|protocol|func|fun|val|let|var)[[:space:]]+(<[^>]*>[[:space:]]+)?([A-Za-z]+\\.)?%s([^A-Za-z0-9_]|$)' "$1"
}

list() { printf '%s\n' "$1" | tr ',' '\n' | sed 's/^ *//; s/ *$//' | grep -v '^$' || true; }

check_side() {
  local page="$1" side="$2" snaps="$3" symbols files shots symbol file found shot
  symbols=$(key "${side}_symbol" "$page")
  files=$(key "${side}_file" "$page")
  shots=$(key "${side}_snapshots" "$page")
  if [ -z "$(list "$symbols")" ] || [ -z "$(list "$files")" ] || [ -z "$(list "$shots")" ]; then
    problems+=("$page: needs ${side}_symbol, ${side}_file and ${side}_snapshots")
    return
  fi
  while IFS= read -r file; do
    case "$file" in "$side"/*) ;; *) problems+=("$page: ${side}_file $file is not under $side/") ;; esac
    [ -f "$root/$file" ] || problems+=("$page: ${side}_file $file does not exist")
  done < <(list "$files")
  while IFS= read -r symbol; do
    found=0
    while IFS= read -r file; do
      if [ -f "$root/$file" ] && grep -qE "$(declares "$symbol")" "$root/$file"; then found=1; fi
    done < <(list "$files")
    [ "$found" = 1 ] || problems+=("$page: ${side}_symbol $symbol is not in $(list "$files" | paste -sd' ' -)")
  done < <(list "$symbols")
  while IFS= read -r shot; do
    [ -f "$snaps/$shot" ] || problems+=("$page: ${side} snapshot $shot does not exist")
  done < <(list "$shots")
}

for name in $pending; do
  in_index "$name" || problems+=("$name is on the pending list but not in the index")
done

while IFS=' ' read -r name target; do
  [ -n "$name" ] || continue
  page="$catalogue/$name/README.md"
  if [ -n "$target" ] && [ "$target" != "$name/README.md" ]; then
    problems+=("the index links $name to $target, not $name/README.md")
  fi
  if [ -f "$page" ]; then
    if is_pending "$name"; then
      problems+=("$name has a page now: take it off the pending list in scripts/check-components.sh")
    fi
  elif ! is_pending "$name"; then
    problems+=("$name is in the index with no page at design/components/$name/README.md")
  fi
done <<<"$rows"

for page in "$catalogue"/*/README.md; do
  [ -f "$page" ] || continue
  name=$(basename "$(dirname "$page")")
  in_index "$name" || problems+=("$page is not in the index")
  [ "$(key name "$page")" = "$name" ] || problems+=("$page: name: should be $name")
  check_side "$page" ios "$ios_snaps"
  gap=$(key not_on_android "$page")
  if [ -n "$gap" ]; then
    if grep -q '^android_' "$page"; then
      problems+=("$page: has android_ keys beside not_on_android")
    fi
  elif grep -q '^not_on_android:' "$page"; then
    problems+=("$page: not_on_android needs a reason")
  else
    check_side "$page" android "$android_snaps"
  fi
done

if [ "${#problems[@]}" -gt 0 ]; then
  echo "✗ The component catalogue and the apps disagree (#2620):" >&2
  printf '    %s\n' "${problems[@]}" >&2
  echo "  Template: design/components/_template.md" >&2
  exit 1
fi
