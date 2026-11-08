#!/usr/bin/env python3
# Builds the sketch kit's generated files, so the Claude Design system's tokens
# and icons are made from the code (#2700):
#
#   design/sketch-kit/tokens.json            from Theme.swift, with the usage notes
#                                            in design/sketch-kit/token-notes.json
#   design/sketch-kit/icons/*.svg            from Android's ic_*.xml vector drawables
#   design/sketch-kit/components/bundle.js   the icons plus components/kit.js
#
# --check regenerates into a scratch folder and fails when a committed file is
# stale, and fails when the kit's CSS, scripts or pages type a colour or name a
# token tokens.json does not have. A Theme.swift line it cannot read fails the
# run rather than dropping the token.

import argparse
import filecmp
import json
import os
import re
import shutil
import sys
import tempfile
import xml.etree.ElementTree as ET

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
KIT = "design/sketch-kit"
COMPONENTS = ["ScreenScaffold", "Card", "LibraryItemCard", "MarkerButton", "TabBar"]
FAMILIES = {"Hanken": ("Hanken Grotesk", "sans"), "Mono": ("DM Mono", "mono")}
WEIGHTS = {"Regular": 400, "Medium": 500, "SemiBold": 600, "Bold": 700}
ANDROID_NS = "{http://schemas.android.com/apk/res/android}"


class KitError(Exception):
    pass


def braced(text, start, name):
    """The text from just after an opening brace to its matching close."""
    depth, i = 1, start
    while depth:
        if i >= len(text):
            raise KitError(f"unclosed block {name}")
        depth += {"{": 1, "}": -1}.get(text[i], 0)
        i += 1
    return text[start : i - 1]


def blocks(text, header):
    """The body of each `enum|struct NAME {` block, matched by braces."""
    out = {}
    for m in re.finditer(r"^(?:enum|struct) (\w+)\b[^{\n]*\{", text, re.M):
        if m.group(1) in header:
            out[m.group(1)] = braced(text, m.end(), m.group(1))
    missing = [h for h in header if h not in out]
    if missing:
        raise KitError("Theme.swift has no " + ", ".join(missing))
    return out


def statements(body):
    """Top-level lines of a block with continuation lines folded in."""
    lines, buf, depth = [], "", 0
    for raw in body.splitlines():
        line = raw.split("//", 1)[0].strip() if not raw.strip().startswith("///") else ""
        if not line and not buf:
            continue
        if line.startswith(".") and not buf and lines:
            lines[-1] = f"{lines[-1]} {line}"
            continue
        buf = f"{buf} {line}".strip()
        depth += line.count("(") + line.count("[") - line.count(")") - line.count("]")
        if depth <= 0 and not buf.endswith("="):
            lines.append(buf)
            buf, depth = "", 0
    if buf:
        lines.append(buf)
    return lines


def rgb(hexcode):
    if not hexcode.startswith("#"):
        raise KitError(f"cannot fade {hexcode}, which is not a plain colour")
    h = hexcode.lstrip("#")
    return ", ".join(str(int(h[i : i + 2], 16)) for i in (0, 2, 4))


def parse_colours(body):
    tokens, hexes, current_func = [], {}, None

    def resolve(name):
        seen = set()
        while name in hexes and hexes[name].startswith("{"):
            if name in seen:
                raise KitError(f"colour alias loop at {name}")
            seen.add(name)
            name = hexes[name][1:-1]
        if name not in hexes:
            raise KitError(f"colour {name} is used before Theme.swift defines it")
        return hexes[name]

    def value(expr, where):
        expr = expr.strip()
        if m := re.fullmatch(r"Color\(hex: 0x([0-9A-Fa-f]{6})\)", expr):
            return "#" + m.group(1).lower()
        if m := re.fullmatch(r"(?:Color\.)?black\.opacity\(([\d.]+)\)", expr):
            return f"rgba(0, 0, 0, {m.group(1)})"
        if m := re.fullmatch(r"(\w+)\.opacity\(([\d.]+)\)", expr):
            return f"rgba({rgb(resolve(m.group(1)))}, {m.group(2)})"
        if re.fullmatch(r"\w+", expr):
            resolve(expr)
            return "{" + expr + "}"
        raise KitError(f"IntradaColor.{where}: cannot read `{expr}`")

    for line in statements(body):
        if m := re.match(r"static let (\w+) = (.+)$", line):
            name, val = m.group(1), value(m.group(2), m.group(1))
            hexes[name] = val
            tokens.append((name, val))
        elif m := re.match(r"static func (\w+)\(_ \w+: HighlighterColour\) -> Color \{", line):
            current_func = m.group(1)
        elif m := re.match(r"case \.(\w+): (.+)$", line):
            if not current_func:
                raise KitError(f"IntradaColor: a case outside a function: `{line}`")
            name = f"{current_func}-{m.group(1)}"
            val = value(m.group(2), name)
            hexes[name] = val
            tokens.append((name, val))
        elif line in ("}", "}}"):
            current_func = None
        elif line.startswith("switch "):
            continue
        else:
            raise KitError(f"IntradaColor: cannot read `{line}`")
    return tokens, resolve


def parse_numbers(body, enum, kind=r"CGFloat|Double"):
    out = []
    for line in statements(body):
        if m := re.fullmatch(rf"static let (\w+): (?:{kind}) = ([\d.]+)", line):
            out.append((m.group(1), m.group(2)))
        else:
            raise KitError(f"{enum}: cannot read `{line}`")
    return out


def parse_fonts(body):
    sizes, faces, styles = {}, {}, []
    nested = blocks(body.replace("  enum ", "enum "), ["Hanken", "Mono"])
    for fam, inner in nested.items():
        for line in statements(inner):
            m = re.fullmatch(r'static let (\w+) = "([\w-]+)"', line)
            if not m:
                raise KitError(f"IntradaFont.{fam}: cannot read `{line}`")
            faces[f"{fam}.{m.group(1)}"] = (fam, m.group(2))
    outer = re.sub(r"enum (Hanken|Mono) \{.*?\n\s*\}", "", body, flags=re.S)
    for line in statements(outer):
        if m := re.fullmatch(r"static let (\w+): CGFloat = ([\d.]+)", line):
            sizes[m.group(1)] = m.group(2)
        elif m := re.fullmatch(
            r"static let (\w+) = Font\.custom\((\w+\.\w+), size: (\w+|[\d.]+), relativeTo: \.\w+\)"
            r"(?:\s*\.monospacedDigit\(\))?",
            line,
        ):
            name, face, size = m.groups()
            if face not in faces:
                raise KitError(f"IntradaFont.{name}: no face {face}")
            size = sizes.get(size, size)
            if not re.fullmatch(r"[\d.]+", size):
                raise KitError(f"IntradaFont.{name}: cannot read size {size}")
            fam, ps = faces[face]
            weight = WEIGHTS[ps.split("-", 1)[1]]
            styles.append((name, size, weight, FAMILIES[fam][1]))
        elif re.fullmatch(r"static let \w+ = Font\.system\(.+\)", line):
            continue
        elif line.startswith("static func ") or line == "}":
            continue
        else:
            raise KitError(f"IntradaFont: cannot read `{line}`")
    return faces, styles


def parse_icon_sizes(body):
    m = re.search(r"var points: CGFloat \{", body)
    if not m:
        raise KitError("IntradaIconSize has no points")
    out = []
    for line in statements(braced(body, m.end(), "IntradaIconSize.points")):
        if line in ("switch self {", "}"):
            continue
        if c := re.fullmatch(r"case \.(\w+): ([\d.]+)", line):
            out.append(c.groups())
        else:
            raise KitError(f"IntradaIconSize: cannot read `{line}`")
    return out


def parse_shadows(body, resolve, opacities):
    out = []
    for line in statements(body):
        if re.fullmatch(r"let \w+: \w+", line):
            continue
        m = re.fullmatch(
            r"static let (\w+) = IntradaShadow\(\s*color: (.+?), radius: ([\d.]+), y: ([\d.]+)\)", line
        )
        if not m:
            raise KitError(f"IntradaShadow: cannot read `{line}`")
        name, colour, radius, y = m.groups()
        if colour == ".clear":
            continue
        colour = re.sub(r"IntradaOpacity\.(\w+)", lambda o: opacities[o.group(1)], colour)
        colour = colour.replace("IntradaColor.", "")
        if cm := re.fullmatch(r"\.?black\.opacity\(([\d.]+)\)", colour):
            css = f"rgba(0, 0, 0, {cm.group(1)})"
        elif cm := re.fullmatch(r"(\w+)\.opacity\(([\d.]+)\)", colour):
            css = f"rgba({rgb(resolve(cm.group(1)))}, {cm.group(2)})"
        elif re.fullmatch(r"\w+", colour):
            css = resolve(colour)
        else:
            raise KitError(f"IntradaShadow.{name}: cannot read colour `{colour}`")
        out.append((name, f"0 {y}px {radius}px {css}"))
    return out


def tokens_json(theme_path, notes_path):
    with open(theme_path, encoding="utf-8") as f:
        text = f.read()
    with open(notes_path, encoding="utf-8") as f:
        notes = json.load(f)
    enums = blocks(
        text,
        ["IntradaColor", "IntradaFont", "IntradaSpacing", "IntradaRadius", "IntradaGlyph",
         "IntradaIconSize", "IntradaOpacity", "IntradaShadow"],
    )
    colours, resolve = parse_colours(enums["IntradaColor"])
    faces, styles = parse_fonts(enums["IntradaFont"])
    opacities = dict(parse_numbers(enums["IntradaOpacity"], "IntradaOpacity", "Double"))

    families = {
        "spacing": [(f"spacing-{n}", f"{v}px") for n, v in parse_numbers(enums["IntradaSpacing"], "IntradaSpacing")],
        "radius": [(f"radius-{n}", f"{v}px") for n, v in parse_numbers(enums["IntradaRadius"], "IntradaRadius")],
        "shadow": [(f"shadow-{n}", v) for n, v in parse_shadows(enums["IntradaShadow"], resolve, opacities)],
        "opacity": [(f"opacity-{n}", v) for n, v in opacities.items()],
        "iconSize": [(f"icon-{n}", f"{v}px") for n, v in parse_icon_sizes(enums["IntradaIconSize"])],
        "glyph": [(f"glyph-{n}", f"{v}px") for n, v in parse_numbers(enums["IntradaGlyph"], "IntradaGlyph")],
    }

    token_notes, style_notes = notes.get("tokens", {}), notes.get("styles", {})
    names = [n for n, _ in colours] + [n for fam in families.values() for n, _ in fam]
    missing = [n for n in names if n not in token_notes]
    missing += [f"style {n}" for n, *_ in styles if n not in style_notes]
    stale = [n for n in token_notes if n not in names]
    stale += [f"style {n}" for n in style_notes if n not in {s[0] for s in styles}]
    stale += [f"family {n}" for n in notes.get("families", {}) if n not in families]
    if missing or stale:
        lines = []
        if missing:
            lines.append("no usage note for: " + ", ".join(missing))
        if stale:
            lines.append("a note for a token Theme.swift no longer has: " + ", ".join(stale))
        raise KitError(f"{os.path.relpath(notes_path, ROOT)}: " + "; ".join(lines))

    def entries(pairs):
        return [{"name": n, "value": v, "usage": token_notes[n]} for n, v in pairs]

    out = {
        "name": "Intrada",
        "version": 1,
        "meta": {
            "source": "github",
            "repo": "jonyardley/intrada",
            "generator": "just sketch-kit",
            "paths": {
                "tokens": ["ios/Intrada/DesignSystem/Theme.swift"],
                "fonts": ["ios/Intrada/Resources/Fonts/"],
                "docs": ["design/components/", "design/sketch-kit/", "docs/design-principles.md"],
            },
        },
        "color": {"themes": [{"id": "light", "name": "Paper"}], "tokens": entries(colours)},
        "type": {
            "fonts": [
                {"family": FAMILIES[fam][0], "file": f"fonts/{ps}.ttf",
                 "weight": str(WEIGHTS[ps.split("-", 1)[1]]), "style": "normal"}
                for fam, ps in faces.values()
            ],
            "families": {
                "sans": '"Hanken Grotesk", system-ui, sans-serif',
                "mono": '"DM Mono", ui-monospace, monospace',
            },
            "groups": [{
                "name": "Five sizes",
                "family": "sans",
                "styles": [
                    {"name": n, "fontSize": f"{s}px", "fontWeight": w, "family": fam, "usage": style_notes[n]}
                    for n, s, w, fam in styles
                ],
            }],
        },
    }
    for fam, pairs in families.items():
        block = {"tokens": entries(pairs)}
        if fam in notes.get("families", {}):
            block = {"note": notes["families"][fam], **block}
        out[fam] = block
    return out


def svg_icons(drawables, ink, paper):
    icons = {}
    for fname in sorted(os.listdir(drawables)):
        if not fname.startswith("ic_") or fname.startswith("ic_launcher") or not fname.endswith(".xml"):
            continue
        root = ET.parse(os.path.join(drawables, fname)).getroot()
        if root.tag != "vector":
            raise KitError(f"{fname}: not a vector drawable")
        vw, vh = root.get(ANDROID_NS + "viewportWidth"), root.get(ANDROID_NS + "viewportHeight")
        paths = []
        for el in root:
            if el.tag != "path":
                raise KitError(f"{fname}: cannot read <{el.tag}>")
            attrs = {k.replace(ANDROID_NS, ""): v for k, v in el.attrib.items()}
            known = {"pathData", "fillColor", "strokeColor", "strokeWidth", "strokeLineCap",
                     "strokeLineJoin", "fillType"}
            if set(attrs) - known:
                raise KitError(f"{fname}: cannot read {sorted(set(attrs) - known)}")
            paths.append(attrs)
        icons[fname[3:-4]] = (vw, vh, paths)

    def colour(argb, inline):
        if argb is None:
            return "none"
        code = argb.upper().lstrip("#")[-6:]
        if code == "000000":
            return "currentColor" if inline else ink
        if code == "FFFFFF":
            return "var(--cardFill)" if inline else paper
        raise KitError(f"icon colour #{code} is not ink or white")

    def render(name, inline):
        vw, vh, paths = icons[name]
        parts = []
        for p in paths:
            a = [f'd="{p["pathData"]}"', f'fill="{colour(p.get("fillColor"), inline)}"']
            if "strokeColor" in p:
                a.append(f'stroke="{colour(p["strokeColor"], inline)}"')
                a.append(f'stroke-width="{p.get("strokeWidth", "1")}"')
            if "strokeLineCap" in p:
                a.append(f'stroke-linecap="{p["strokeLineCap"]}"')
            if "strokeLineJoin" in p:
                a.append(f'stroke-linejoin="{p["strokeLineJoin"]}"')
            if p.get("fillType") == "evenOdd":
                a.append('fill-rule="evenodd"')
            parts.append("<path " + " ".join(a) + "/>")
        inner = "".join(parts)
        if inline:
            return f"0 0 {vw} {vh}", inner
        return (f'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" '
                f'viewBox="0 0 {vw} {vh}">{inner}</svg>\n')

    return icons, render


def build(out_dir, theme, notes, drawables, kit_js):
    tokens = tokens_json(theme, notes)
    by_name = {t["name"]: t["value"] for t in tokens["color"]["tokens"]}
    icons, render = svg_icons(drawables, by_name["ink"], by_name["cardFill"])

    os.makedirs(os.path.join(out_dir, "icons"), exist_ok=True)
    os.makedirs(os.path.join(out_dir, "components"), exist_ok=True)
    with open(os.path.join(out_dir, "tokens.json"), "w", encoding="utf-8") as f:
        json.dump(tokens, f, indent=2, ensure_ascii=False)
        f.write("\n")
    for name in icons:
        with open(os.path.join(out_dir, "icons", f"{name}.svg"), "w", encoding="utf-8") as f:
            f.write(render(name, False))

    with open(kit_js, encoding="utf-8") as f:
        source = f.read()
    inline = {n: dict(zip(("viewBox", "body"), render(n, True))) for n in icons}
    header = {"format": 4, "namespace": "Intrada", "components": [{"name": c} for c in COMPONENTS]}
    with open(os.path.join(out_dir, "components", "bundle.js"), "w", encoding="utf-8") as f:
        f.write(f"/* @ds-bundle: {json.dumps(header, separators=(',', ':'))} */\n")
        f.write("/* Generated by `just sketch-kit` from components/kit.js and Android's ic_*.xml. */\n")
        f.write("(function () {\n")
        f.write("var ICONS = " + json.dumps(inline, indent=1, sort_keys=True) + ";\n")
        f.write(source)
        f.write("})();\n")
    return tokens


TOKEN_FILES = ("components/kit.js", "components/bundle.css")


def lint(kit_dir, tokens):
    names = {t["name"] for fam, block in tokens.items() if isinstance(block, dict)
             for t in block.get("tokens", [])}
    names |= {f"font-{k}" for k in tokens["type"]["families"]}
    styles = {s["name"] for g in tokens["type"]["groups"] for s in g["styles"]}
    problems = []
    with open(os.path.join(kit_dir, "icons.json"), encoding="utf-8") as f:
        for symbol, icon in json.load(f)["symbols"].items():
            if icon and not os.path.exists(os.path.join(kit_dir, "icons", f"{icon}.svg")):
                problems.append(f"icons.json: {symbol} maps to {icon}, which has no icons/{icon}.svg")
    files = list(TOKEN_FILES)
    files += [f"components/{c}/preview.html" for c in COMPONENTS]
    for rel in files:
        path = os.path.join(kit_dir, rel)
        if not os.path.exists(path):
            problems.append(f"{rel}: missing")
            continue
        with open(path, encoding="utf-8") as f:
            text = f.read()
        for n, line in enumerate(text.splitlines(), 1):
            if re.search(r"#[0-9A-Fa-f]{3,8}\b|\b(?:rgba?|hsla?)\(", line):
                problems.append(f"{rel}:{n}: a typed colour; use a token from tokens.json")
            if re.search(r"font-size\s*:|fontSize\s*:", line):
                problems.append(f"{rel}:{n}: a typed font size; use a type style class")
            for var in re.findall(r"var\(--([A-Za-z0-9_.-]+)", line):
                if var.endswith("-") and any(n.startswith(var) for n in names):
                    continue
                if var not in names and not var.startswith("ik-"):
                    problems.append(f"{rel}:{n}: --{var} is not in tokens.json")
            for cls in re.findall(r'\btype\("(\w+)"', line):
                if cls not in styles:
                    problems.append(f"{rel}:{n}: no type style {cls}")
    return problems


def main():
    p = argparse.ArgumentParser(description="Build the sketch kit's tokens, icons and bundle from the code (#2700).")
    p.add_argument("--check", action="store_true", help="fail when the committed kit is stale")
    p.add_argument("--theme", default=os.path.join(ROOT, "ios/Intrada/DesignSystem/Theme.swift"))
    p.add_argument("--drawables", default=os.path.join(ROOT, "android/app/src/main/res/drawable"))
    p.add_argument("--kit", default=os.path.join(ROOT, KIT))
    args = p.parse_args()
    notes = os.path.join(args.kit, "token-notes.json")
    kit_js = os.path.join(args.kit, "components/kit.js")

    try:
        if not args.check:
            # Built aside and swapped in, so a failed run never leaves icons/ half written.
            with tempfile.TemporaryDirectory() as tmp:
                tokens = build(tmp, args.theme, notes, args.drawables, kit_js)
                for rel in ("tokens.json", "components/bundle.js"):
                    shutil.copyfile(os.path.join(tmp, rel), os.path.join(args.kit, rel))
                shutil.rmtree(os.path.join(args.kit, "icons"), ignore_errors=True)
                shutil.move(os.path.join(tmp, "icons"), os.path.join(args.kit, "icons"))
            problems = lint(args.kit, tokens)
            if problems:
                raise KitError("the kit strays from tokens.json:\n  " + "\n  ".join(problems))
            print(f"sketch kit: {len(tokens['color']['tokens'])} colours, "
                  f"{len(os.listdir(os.path.join(args.kit, 'icons')))} icons written to {KIT}")
            return 0
        with tempfile.TemporaryDirectory() as tmp:
            tokens = build(tmp, args.theme, notes, args.drawables, kit_js)
            stale = []
            for rel in ("tokens.json", "components/bundle.js"):
                committed = os.path.join(args.kit, rel)
                if not os.path.exists(committed) or not filecmp.cmp(os.path.join(tmp, rel), committed, shallow=False):
                    stale.append(rel)
            fresh = set(os.listdir(os.path.join(tmp, "icons")))
            kept_dir = os.path.join(args.kit, "icons")
            kept = set(os.listdir(kept_dir)) if os.path.isdir(kept_dir) else set()
            for name in sorted(fresh | kept):
                if name not in fresh or name not in kept or not filecmp.cmp(
                    os.path.join(tmp, "icons", name), os.path.join(kept_dir, name), shallow=False
                ):
                    stale.append(f"icons/{name}")
            problems = lint(args.kit, tokens)
        if stale:
            problems.insert(0, "stale, run `just sketch-kit`: " + ", ".join(stale))
        if problems:
            raise KitError("\n  ".join(problems))
        return 0
    except KitError as e:
        print(f"✗ sketch kit: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
