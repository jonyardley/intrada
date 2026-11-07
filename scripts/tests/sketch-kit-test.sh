#!/usr/bin/env bash
# Self-test for scripts/sketch-kit.py (#2700): each Theme.swift construct it
# reads lands in tokens.json with the right value, a line it cannot read fails
# rather than dropping a token, a stale kit fails --check, and a typed colour or
# unknown token in the kit fails.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/sketch-kit.py"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

pass=0
fail=0
expect() {
  local name="$1" want="$2" got="$3"
  if [ "$want" = "$got" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf '✗ %s\n  want: %s\n  got:  %s\n' "$name" "$want" "$got"
    [ ! -s "$tmp/err" ] || sed 's/^/  stderr: /' "$tmp/err"
  fi
}

theme="$tmp/Theme.swift"
cat >"$theme" <<'EOF'
enum IntradaColor {
  /// A doc line the parser skips.
  static let paperTop = Color(hex: 0xF7F4EF)
  static let cardFill = Color(hex: 0xFFFFFF)
  static let ink = Color(hex: 0x2A2725)
  static let accent = ink
  static let marker = Color(hex: 0xFFE9A3)
  static func marker(_ colour: HighlighterColour) -> Color {
    switch colour {
    case .butter: marker
    case .coral: Color(hex: 0xFFB4A2)
    }
  }
  static let shadow = ink.opacity(0.05)
  static let sheetScrim = Color.black.opacity(0.2)
}
enum IntradaFont {
  static let pageTitleSize: CGFloat = 30
  static let pageTitle = Font.custom(Hanken.semibold, size: pageTitleSize, relativeTo: .largeTitle)
  static let secondary = Font.custom(Hanken.regular, size: 15, relativeTo: .subheadline)
    .monospacedDigit()
  static let figure = Font.custom(Mono.regular, size: 15, relativeTo: .subheadline)
  static func timer(_ size: CGFloat = 56) -> Font {
    .custom(Hanken.semibold, size: size, relativeTo: .largeTitle)
  }
  static let chart = Font.system(.footnote, design: .monospaced)
  enum Hanken {
    static let regular = "HankenGrotesk-Regular"
    static let semibold = "HankenGrotesk-SemiBold"
  }
  enum Mono {
    static let regular = "DMMono-Regular"
  }
}
enum IntradaSpacing {
  static let card: CGFloat = 16
}
enum IntradaRadius {
  static let card: CGFloat = 3
}
enum IntradaGlyph {
  static let bar: CGFloat = 36
}
enum IntradaIconSize {
  case inline
  var points: CGFloat {
    switch self {
    case .inline: 15
    }
  }
}
enum IntradaOpacity {
  static let wash: Double = 0.12
}
struct IntradaShadow {
  let color: Color
  static let none = IntradaShadow(color: .clear, radius: 0, y: 0)
  static let card = IntradaShadow(color: IntradaColor.shadow, radius: 1, y: 1)
  static let hero = IntradaShadow(color: .black.opacity(0.18), radius: 20, y: 10)
  static let appIcon = IntradaShadow(
    color: IntradaColor.ink.opacity(IntradaOpacity.wash), radius: 10, y: 8)
}
EOF

mkdir -p "$tmp/drawable" "$tmp/kit/components"
cat >"$tmp/drawable/ic_plus.xml" <<'EOF'
<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="24dp" android:height="24dp"
    android:viewportWidth="24" android:viewportHeight="24">
    <path android:pathData="M12,5L12,19M5,12L19,12"
        android:strokeColor="#FF000000" android:strokeWidth="2" android:strokeLineCap="round" />
</vector>
EOF
cp "$root/android/app/src/main/res/drawable/ic_star.xml" "$tmp/drawable/"
echo '<vector xmlns:android="http://schemas.android.com/apk/res/android"><group/></vector>' \
  >"$tmp/drawable/ic_launcher_foreground.xml"

notes='{"tokens": {}, "styles": {"pageTitle": "t", "secondary": "s", "figure": "f"}, "families": {}}'
for n in paperTop cardFill ink accent marker marker-butter marker-coral shadow sheetScrim \
  spacing-card radius-card glyph-bar icon-inline opacity-wash shadow-card shadow-hero shadow-appIcon; do
  notes=$(python3 -c 'import json,sys; d=json.loads(sys.argv[1]); d["tokens"][sys.argv[2]]="u"; print(json.dumps(d))' "$notes" "$n")
done
echo "$notes" >"$tmp/kit/token-notes.json"
echo '{"symbols": {"plus": "plus", "music.note": null}}' >"$tmp/kit/icons.json"
echo 'window.Intrada = { Card: function () {} };' >"$tmp/kit/components/kit.js"
echo '.ik-card { background: var(--cardFill); color: var(--ik-own); }' >"$tmp/kit/components/bundle.css"
for c in ScreenScaffold Card LibraryItemCard MarkerButton TabBar; do
  mkdir -p "$tmp/kit/components/$c"
  echo '<div class="pageTitle" style="background: var(--paperTop)"></div>' >"$tmp/kit/components/$c/preview.html"
done

run() {
  local status=0
  python3 "$script" --theme "$theme" --drawables "$tmp/drawable" --kit "$tmp/kit" "$@" \
    >"$tmp/out" 2>"$tmp/err" || status=$?
  echo "$status"
}
token() {
  python3 -c 'import json,sys
d=json.load(open(sys.argv[1]))
for fam in d.values():
    if isinstance(fam, dict):
        for t in fam.get("tokens", []):
            if t["name"] == sys.argv[2]: print(t["value"])' "$tmp/kit/tokens.json" "$1"
}

expect "generates" 0 "$(run)"
expect "hex colour" "#f7f4ef" "$(token paperTop)"
expect "alias" "{ink}" "$(token accent)"
expect "marker case alias" "{marker}" "$(token marker-butter)"
expect "marker case hex" "#ffb4a2" "$(token marker-coral)"
expect "faded token" "rgba(42, 39, 37, 0.05)" "$(token shadow)"
expect "faded black" "rgba(0, 0, 0, 0.2)" "$(token sheetScrim)"
expect "spacing" "16px" "$(token spacing-card)"
expect "icon size" "15px" "$(token icon-inline)"
expect "opacity" "0.12" "$(token opacity-wash)"
expect "shadow through a token" "0 1px 1px rgba(42, 39, 37, 0.05)" "$(token shadow-card)"
expect "two-line shadow with an opacity token" "0 8px 10px rgba(42, 39, 37, 0.12)" "$(token shadow-appIcon)"
expect "clear shadow left out" "" "$(token shadow-none)"
expect "type styles" "pageTitle:30px:600:sans secondary:15px:400:sans figure:15px:400:mono" \
  "$(python3 -c 'import json,sys
d=json.load(open(sys.argv[1]))
print(" ".join(":".join(str(s[k]) for k in ("name", "fontSize", "fontWeight", "family")) for s in d["type"]["groups"][0]["styles"]))' "$tmp/kit/tokens.json")"
expect "icons, launcher skipped" "plus.svg star.svg" "$(ls "$tmp/kit/icons" | tr '\n' ' ' | sed 's/ $//')"
expect "icon in ink" 1 "$(grep -c 'stroke="#2a2725"' "$tmp/kit/icons/plus.svg")"
expect "bundle icon follows the text colour" 1 "$(grep -c 'stroke=\\"currentColor' "$tmp/kit/components/bundle.js")"
expect "fresh kit passes --check" 0 "$(run --check)"

cp "$theme" "$tmp/Theme.good"
sed -i.bak 's/0xF7F4EF/0xF7F4F0/' "$theme"
expect "changed Theme.swift fails --check" 1 "$(run --check)"
expect "names the stale file" 1 "$(grep -c 'stale, run `just sketch-kit`: tokens.json' "$tmp/err")"
cp "$tmp/Theme.good" "$theme"

sed -i.bak 's/static let ink = Color(hex: 0x2A2725)/static let ink = Color(hex: 0x2A2725)\n  static let glow = ink.mix(paperTop)/' "$theme"
expect "unreadable colour fails" 1 "$(run)"
expect "names the line" 1 "$(grep -c 'cannot read `ink.mix(paperTop)`' "$tmp/err")"
cp "$tmp/Theme.good" "$theme"

sed -i.bak 's/  static let card: CGFloat = 16/  static let card: CGFloat = 16\n  static let page: CGFloat = 20/' "$theme"
expect "token with no usage note fails" 1 "$(run)"
expect "names the token" 1 "$(grep -c 'no usage note for: spacing-page' "$tmp/err")"
cp "$tmp/Theme.good" "$theme"

expect "restored kit generates" 0 "$(run)"
cp "$tmp/kit/components/bundle.css" "$tmp/bundle.good"
echo '.ik-x { color: #2A2725; }' >>"$tmp/kit/components/bundle.css"
expect "typed colour fails" 1 "$(run --check)"
cp "$tmp/bundle.good" "$tmp/kit/components/bundle.css"
echo '.ik-x { color: var(--brand); }' >>"$tmp/kit/components/bundle.css"
expect "unknown token fails" 1 "$(run --check)"
expect "names the token" 1 "$(grep -c -- '--brand is not in tokens.json' "$tmp/err")"
cp "$tmp/bundle.good" "$tmp/kit/components/bundle.css"
echo '{"symbols": {"plus": "plus", "timer": "tab_practice"}}' >"$tmp/kit/icons.json"
expect "icon mapping to a missing svg fails" 1 "$(run --check)"

echo "sketch-kit-test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
