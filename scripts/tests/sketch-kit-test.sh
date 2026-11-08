#!/usr/bin/env bash
# Self-test for scripts/sketch-kit.py (#2700): each Theme.swift construct it
# reads lands in tokens.json with the right value, a line it cannot read fails
# rather than dropping a token, tokens.css defines what the kit reads (#2772),
# a stale kit fails --check, and a typed colour, font size, type style or token
# the kit does not have fails.
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
  var maxPoints: CGFloat {
    switch self {
    case .inline: 18
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
expect "icon size, points only" "15px" "$(token icon-inline)"
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
css="$tmp/kit/components/tokens.css"
expect "css alias reads the token" 1 "$(grep -cx '  --accent: var(--ink);' "$css")"
expect "css marker case alias" 1 "$(grep -cx '  --marker-butter: var(--marker);' "$css")"
expect "css plain value" 1 "$(grep -cx '  --spacing-card: 16px;' "$css")"
expect "css shadow" 1 "$(grep -cx '  --shadow-card: 0 1px 1px rgba(42, 39, 37, 0.05);' "$css")"
expect "css sans family" 1 "$(grep -cx '  --font-sans: "Hanken Grotesk", system-ui, sans-serif;' "$css")"
expect "css type class" 1 "$(grep -cx '.pageTitle { font-family: var(--font-sans); font-size: 30px; font-weight: 600; }' "$css")"
expect "css mono type class" 1 "$(grep -cx '.figure { font-family: var(--font-mono); font-size: 15px; font-weight: 400; }' "$css")"
expect "css font face per face" 3 "$(grep -c '^@font-face' "$css")"
expect "css font from the published fonts folder" 1 "$(grep -c 'url("../fonts/DMMono-Regular.ttf")' "$css")"
expect "fresh kit passes --check" 0 "$(run --check)"
expect "alias left inside a css value fails" \
  'tokens.css: cannot turn `{ink} 50%` for glow into CSS' \
  "$(python3 -c 'import importlib.util, sys
spec = importlib.util.spec_from_file_location("kit", sys.argv[1])
kit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(kit)
try:
    kit.tokens_css({"type": {"fonts": [], "families": {}, "groups": []},
                    "color": {"tokens": [{"name": "glow", "value": "{ink} 50%"}]}})
    print("no error")
except kit.KitError as e:
    print(e)' "$script")"

cp "$css" "$tmp/css.good"
echo '.extra { color: inherit; }' >>"$css"
expect "hand-edited tokens.css fails --check" 1 "$(run --check)"
expect "names the stale tokens.css" 1 "$(grep -c 'stale, run `just sketch-kit`: components/tokens.css$' "$tmp/err")"
rm "$css"
expect "missing tokens.css fails --check" 1 "$(run --check)"
expect "names the missing tokens.css" 1 "$(grep -c 'stale, run `just sketch-kit`: components/tokens.css$' "$tmp/err")"
cp "$tmp/css.good" "$css"

cp "$theme" "$tmp/Theme.good"
sed -i.bak 's/0xF7F4EF/0xF7F4F0/' "$theme"
expect "changed Theme.swift fails --check" 1 "$(run --check)"
expect "names the stale files" 1 "$(grep -c 'stale, run `just sketch-kit`: tokens.json, components/tokens.css$' "$tmp/err")"
cp "$tmp/Theme.good" "$theme"

sed -i.bak 's/static let ink = Color(hex: 0x2A2725)/static let ink = Color(hex: 0x2A2725)\n  static let glow = ink.mix(paperTop)/' "$theme"
expect "unreadable colour fails" 1 "$(run)"
expect "names the line" 1 "$(grep -c 'cannot read `ink.mix(paperTop)`' "$tmp/err")"
cp "$tmp/Theme.good" "$theme"

sed -i.bak 's/  static let card: CGFloat = 16/  static let card: CGFloat = 16\n  static let page: CGFloat = 20/' "$theme"
expect "token with no usage note fails" 1 "$(run)"
expect "names the token" 1 "$(grep -c 'no usage note for: spacing-page' "$tmp/err")"
cp "$tmp/Theme.good" "$theme"

sed -i.bak 's/static let pageTitleSize: CGFloat = 30/static let pageTitleSize: CGFloat = 30\n  static let odd = Font.title/' "$theme"
expect "unreadable font line fails" 1 "$(run)"
expect "names the font line" 1 "$(grep -c 'IntradaFont: cannot read `static let odd = Font.title`' "$tmp/err")"
cp "$tmp/Theme.good" "$theme"

sed -i.bak 's/  static let card: CGFloat = 16/  static let card: CGFloat = 16\n  static let big: CGFloat = card * 2/' "$theme"
expect "unreadable spacing line fails" 1 "$(run)"
expect "names the spacing line" 1 "$(grep -c 'IntradaSpacing: cannot read `static let big: CGFloat = card \* 2`' "$tmp/err")"
cp "$tmp/Theme.good" "$theme"

sed -i.bak 's/  static let none = IntradaShadow/  static let flat = IntradaShadow(color: IntradaColor.shadow, radius: 1)\n  static let none = IntradaShadow/' "$theme"
expect "unreadable shadow line fails" 1 "$(run)"
expect "names the shadow line" 1 "$(grep -c 'IntradaShadow: cannot read `static let flat' "$tmp/err")"
cp "$tmp/Theme.good" "$theme"

sed -i.bak 's/case .inline: 15$/case .inline: 15 * 2/' "$theme"
expect "worked icon size fails" 1 "$(run)"
expect "names the icon size line" 1 "$(grep -c 'IntradaIconSize: cannot read `case .inline: 15 \* 2`' "$tmp/err")"
cp "$tmp/Theme.good" "$theme"

sed -i.bak 's/case .inline: 15$/case .inline: 15\n    case .huge: IntradaSpacing.card/' "$theme"
expect "icon size from a token fails" 1 "$(run)"
expect "names the token icon size" 1 "$(grep -c 'IntradaIconSize: cannot read `case .huge: IntradaSpacing.card`' "$tmp/err")"
cp "$tmp/Theme.good" "$theme"

cp "$tmp/kit/token-notes.json" "$tmp/notes.good"
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); d["tokens"]["spacing-gone"]="u"; json.dump(d, open(sys.argv[1], "w"))' "$tmp/kit/token-notes.json"
expect "note for a gone token fails" 1 "$(run)"
expect "names the gone token" 1 "$(grep -c 'a note for a token Theme.swift no longer has: spacing-gone' "$tmp/err")"
cp "$tmp/notes.good" "$tmp/kit/token-notes.json"

sed 's/#FF000000/#FFFF0000/' "$tmp/drawable/ic_plus.xml" >"$tmp/drawable/ic_red.xml"
expect "red drawable fails" 1 "$(run)"
expect "names the colour" 1 "$(grep -c 'icon colour #FF0000 is not ink or white' "$tmp/err")"
expect "failed run keeps the icons" "plus.svg star.svg" "$(ls "$tmp/kit/icons" | tr '\n' ' ' | sed 's/ $//')"
rm "$tmp/drawable/ic_red.xml"

expect "restored kit generates" 0 "$(run)"

cp "$tmp/drawable/ic_plus.xml" "$tmp/plus.good"
sed 's/M12,5L12,19/M12,4L12,20/' "$tmp/plus.good" >"$tmp/drawable/ic_plus.xml"
expect "changed drawable fails --check" 1 "$(run --check)"
expect "names the changed icon" 1 "$(grep -c 'stale, run `just sketch-kit`: components/bundle.js, icons/plus.svg$' "$tmp/err")"
cp "$tmp/plus.good" "$tmp/drawable/ic_plus.xml"

mv "$tmp/drawable/ic_star.xml" "$tmp/star.good"
expect "deleted drawable fails --check" 1 "$(run --check)"
expect "names the deleted icon" 1 "$(grep -c 'stale, run `just sketch-kit`: components/bundle.js, icons/star.svg$' "$tmp/err")"
mv "$tmp/star.good" "$tmp/drawable/ic_star.xml"

cp "$tmp/kit/components/kit.js" "$tmp/kit.good"
echo 'var later = 1;' >>"$tmp/kit/components/kit.js"
expect "kit.js left unregenerated fails --check" 1 "$(run --check)"
expect "names the stale bundle" 1 "$(grep -c 'stale, run `just sketch-kit`: components/bundle.js$' "$tmp/err")"
cp "$tmp/kit.good" "$tmp/kit/components/kit.js"

echo 'h("b", { className: type("nope") });' >>"$tmp/kit/components/kit.js"
expect "unknown type style fails" 1 "$(run --check)"
expect "names the type style" 1 "$(grep -c 'components/kit.js:2: no type style nope' "$tmp/err")"
cp "$tmp/kit.good" "$tmp/kit/components/kit.js"

expect "kit passes --check again" 0 "$(run --check)"
cp "$tmp/kit/components/bundle.css" "$tmp/bundle.good"
echo '.ik-x { color: #2A2725; }' >>"$tmp/kit/components/bundle.css"
expect "typed colour fails" 1 "$(run --check)"
expect "names the typed colour" 1 "$(grep -c 'bundle.css:2: a typed colour' "$tmp/err")"
cp "$tmp/bundle.good" "$tmp/kit/components/bundle.css"
echo '.ik-x { font-size: 12px; }' >>"$tmp/kit/components/bundle.css"
expect "typed font size fails" 1 "$(run --check)"
expect "names the font size" 1 "$(grep -c 'bundle.css:2: a typed font size' "$tmp/err")"
cp "$tmp/bundle.good" "$tmp/kit/components/bundle.css"
echo '.ik-x { color: var(--brand); }' >>"$tmp/kit/components/bundle.css"
expect "unknown token fails" 1 "$(run --check)"
expect "names the token" 1 "$(grep -c -- '--brand is not in tokens.json' "$tmp/err")"
cp "$tmp/bundle.good" "$tmp/kit/components/bundle.css"
echo '{"symbols": {"plus": "plus", "timer": "tab_practice"}}' >"$tmp/kit/icons.json"
expect "icon mapping to a missing svg fails" 1 "$(run --check)"
expect "names the mapping" 1 "$(grep -c 'timer maps to tab_practice' "$tmp/err")"

echo "sketch-kit-test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
