# Light-mode "paper" exploration — design notes

> Added 2026-05-31. **Status: exploration, not shipped.** The live app is still
> the dark glassmorphism theme (`crates/intrada-web/input.css`). These notes
> document a light "paper / jazz score" direction explored in Pencil so it can
> be referenced when picking up the native iOS rebuild.

All frames live in `design/intrada.pen` (open in Pencil), grouped to the right
of the existing screens (canvas x ≈ 16000–17500). The `.pen` is encrypted —
this markdown is the readable record.

## The idea

A light theme inspired by **Moleskine paper** and a **jazz Real Book / manuscript
page**: warm cream surfaces, ink-like text, clean minimal type. Explored four
palettes on the Mobile Library screen (Sepia monochrome, Ink + oxblood, Deep
teal/petrol, Indigo carryover), then locked **Indigo carryover** — the brand
indigo deepened to sit on paper, for continuity.

Typography held constant: **serif headings + clean sans body** (Source Serif 4
over Inter in the mock).

## Locked palette — "Indigo · Refined" (light/paper)

| Role | Value |
|------|-------|
| Paper background | gradient `#F4F1E8 → #EBE7D9` (cooler cream so indigo stays crisp) |
| Card surface | `#FCFAF3` |
| Hairline / border | `#E5DECD` (cards), `#E0D9C8` (rules / tab bar) |
| Ink (primary text) | `#2B2A26` (warm near-black, not pure black) |
| Secondary text | `#6E6557` |
| Muted / meta text | `#9C927F` |
| Accent (indigo) | `#4C3FA6`; signature left bar = gradient `#6346E5 → #4C3FA6` |
| Tab bar surface | `#EFEBDF` |

Cards sit flat on the paper (border + slightly brighter fill for separation; a
soft shadow was tried and removed — the flatter look read more "printed page").

## Type language — Piece vs Exercise

The strongest decision to carry forward. Each type is a **colour + icon pair**,
used together everywhere (list, detail header, session setlist):

| Type | Colour | Icon (Lucide mock) | Meaning |
|------|--------|--------------------|---------|
| **Piece** | indigo `#4C3FA6` | `music` (♪) | repertoire |
| **Exercise** | gold `#9E7B33` | `repeat` (reps/cycle) | technical drill |

Badge tints: Piece `#E7E3F4` / `#4C3FA6`, Exercise `#F0E5CC` / `#8A6A2E`.

Rules:
- **The type-coloured left bar is the always-on signal** (zero horizontal space).
- **Filtered list** (Pieces / Exercises tab) → **bar only, no badge.** The tab
  already tells you the type; a per-row label is redundant.
- **Mixed list** (search · "All" · a Set's contents · a session setlist) →
  **icon chip** (small tinted square + glyph). Compact (≈ half the width of the
  old text pill) and colour-blind-safe because the glyph differs, not just hue.

This fixed the earlier incoherence where the bar was always indigo while the
badge was type-coloured — bar and chip now share one hue per type.

The exercise glyph (`repeat`) was a Lucide placeholder; pick deliberately when
building the real icon set (see native note below).

## Carrying this into the native iOS rebuild

The pixels are throwaway; the **direction, tokens, and type language** are not.
Native SwiftUI also *improves* several of these answers:

- **Type icons — solved natively.** SF Symbols has `metronome` (ideal for
  Exercise) and `music.note` / `music.quarternote.3` (Piece). Use those instead
  of the Lucide placeholders.
- **Typography — no font bundling needed.** **New York** (the system serif,
  `Font.system(.largeTitle, design: .serif)`) over **SF Pro** body is the native
  expression of "Source Serif + Inter" and pairs beautifully with iOS large
  titles. Bundle Source Serif 4 only if exact brand match matters.
- **Colours → an asset catalog** (`Color` sets) or a small `Theme`. Asset-catalog
  colours get **automatic light/dark** for free — which is the natural place to
  decide whether native ships light-paper, keeps dark, or supports both.
- **Native primitives replace web hacks.** `NavigationStack` large titles give the
  big serif page title; `List`/insetGrouped or custom rows give the cards; safe
  areas, no-zoom inputs, and view transitions are free. The CSS reset / 16px-font
  / `env(safe-area-*)` / View-Transitions plumbing from the web shell does **not**
  carry over.
- **Card → a row:** leading type-coloured accent (a thin `Capsule`/`Rectangle`,
  or a tinted row), title / composer / `key · tempo` meta, trailing type chip
  **only in mixed lists**.
- **Do not carry:** Tauri / WKWebView specifics, Leptos component names,
  glassmorphism, or any exact Pencil pixel metrics — use system metrics.

## Open questions

- Does native launch in **light-paper**, keep **dark**, or support **both** (the
  asset catalog makes both cheap)?
- Final **exercise glyph** (lean `metronome` on native).
- **Paper warmth per pillar** (Plan / Practice / Track) — held constant here.
- Apply the language to **Item Detail** (type chip in header) and the **session
  setlist** (mixed by nature — where the chip pays off most).
