---
name: intrada-design-system
description: "The Paper & Score design system rules for the native iOS app: the tokens-to-screens hierarchy (IntradaColor/IntradaFont/IntradaSpacing/IntradaRadius, never a raw hex or literal padding), the existing primitives to reuse before building new markup, opaque backings for animated reveals, when deviation is allowed, iOS native-feel rules (haptics, iPad split view, safe areas, motion tokens), and the Claude Design workflow. MUST read before any UI or UX change: new surface, layout, flow, component, or interaction."
---

## Design System Rules

The native app uses a "Paper & Score" light theme: warm paper backgrounds, serif
titles (Source Serif 4), sans body text (Inter). All tokens live in `Theme.swift`
(`ios/Intrada/DesignSystem/Theme.swift`); the shareable export is
[`design/intrada-design-system.dc.html`](design/intrada-design-system.dc.html).

**Consult [`docs/design-principles.md`](docs/design-principles.md) before making
any UI/UX design decision** — new surface, layout, flow, or interaction. It is
the source of truth for how the app should feel: the "spend friction
deliberately" model, one-primary-action-per-screen, content-over-chrome,
progressive disclosure, reversible-by-default. It carries a dated decisions log
(T-numbered); when a new decision is made, append to that log rather than deciding
silently.

**Every user-facing string is written against
[`docs/tone-of-voice.md`](docs/tone-of-voice.md)** — titles, buttons, labels,
empty states, errors, accessibility labels. Plain British English in a
musician's words; no cheerleading, no AI-isms, no em dashes. Its sweep
checklist is the review pass for any PR that adds or changes copy.

### Hierarchy: Tokens → Modifiers → Components → Screens

1. **Tokens first**: every colour, font, spacing and radius value traces to a
   named token (`IntradaColor`, `IntradaFont`, `IntradaSpacing`, `IntradaRadius`).
   Never hard-code a hex, a raw `.padding(16)`, or `cornerRadius: 12`.
2. **Reuse before creating**: check `ios/Intrada/DesignSystem/` and
   `ios/Intrada/Views/Components/` before building new markup.
3. **Known primitives to reach for**: `TagChip`, `TypeBadge`, `ScoreRing`,
   `BottomSheet`, `SegmentedPills`, `CardSurface`/`CardShadow`, `GlobalBanner`,
   `FormErrorBanner`, `PlaceholderContent`, `ScreenScaffold`, `SectionHeader`,
   `HairlineDivider`, `SegmentedProgress`.
4. **Every top-level screen** is built from `ScreenScaffold`
   (`ios/Intrada/DesignSystem/ScreenScaffold.swift`) so navigation chrome, safe
   areas, and background stay consistent.

### Animated reveals need an opaque backing

Anything that **slides or fades in/out over other content** — a search bar, an
expanding row, a banner, a sheet-like panel — must paint an **opaque background
token** (`paperTop` / `cardFill`, never `clear`), or the transition **ghosts**
and you see both components overlap mid-animation.

- The **moving** view gets an opaque background so it hides what it travels over.
- When it should emerge from *behind* sibling chrome, that chrome must also be
  opaque **and** sit on top (`.zIndex(1)`), or it can't occlude anything.

It's the background that hides the motion, not the transition. Don't ship a
reveal animation without checking what shows through behind it.

### Don't deviate from the system unless you're explicitly redesigning

Hand-rolled views that duplicate an existing primitive are the #1 source of
visual drift in this codebase. Before writing UI code:

- **Grep first.** About to hand-roll a chip, badge, sheet, or card that already
  exists under `DesignSystem/` or `Views/Components/`? Use the existing one.
- **Extend, don't clone.** If a primitive *almost* fits, add a parameter to the
  shared component (as `SegmentedPills` and `LibraryItemCard` already do). Don't
  ship a parallel one-off.
- **Typography**: use `IntradaFont` tokens (`.pageTitle`, `.cardTitle`,
  `.sectionTitle`, `.fieldLabel`), never a raw `.font(.system(...))`.
- **Spacing**: use `IntradaSpacing` tokens (`controlGap`, `cardCompact`, `row`,
  `card`), never a literal `.padding(16)`.

Deviation is only acceptable when **explicitly redesigning** a surface, and that
is a deliberate flagged conversation (Claude Design first, then Plan mode), not
an accident inside an unrelated feature PR. A redesign produces *updated tokens
and primitives* in `Theme.swift`, not a hand-rolled clone in a single view.

### iOS native-feel rules

- **Haptics**: use `UIImpactFeedbackGenerator` / `UISelectionFeedbackGenerator`
  via the `Store+Feedback` helpers — `selection` for tabs, `light` for taps,
  `success` for saves (only after the core confirms), `warning` for destructive
  confirms.
- **iPad**: list→detail screens use `LibrarySplitView`. Build it with the view,
  not as a retrofit.
- **Safe areas**: respect them by default (`ScreenScaffold` handles this); don't
  fight SwiftUI's layout with manual insets unless genuinely edge-to-edge.
- **Animations**: use the tokens in `Motion.swift`, not ad hoc `.spring(...)`.

## Design Workflow

Design happens in **Claude Design**; full process in
[`docs/design-workflow.md`](docs/design-workflow.md). The living reference is
[`design/intrada-design-system.dc.html`](design/intrada-design-system.dc.html),
**derived from `Theme.swift`**, which stays the canonical token source. Required
for new views and significant UI changes: mock the screen against the existing
kit first, reuse tokens and components, and if something new is needed update
`Theme.swift` and the design reference (plus its `support.js`) together.

Pencil (`design/intrada.pen`) is **retired** — do not use it or edit that file.
`design/light-mode-exploration.md` remains as provenance.

