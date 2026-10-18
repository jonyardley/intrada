---
paths:
  - "ios/Intrada/**/*.swift"
---

# Paper & Score: the design system rules

Off-white paper, near-neutral ink, one highlighter as the only bright colour,
the musician's choice of eight through `@Environment(\.marker)` (#1677);
Hanken Grotesk for words, DM Mono for figures only. Every token lives in
`ios/Intrada/DesignSystem/Theme.swift`; the shareable export,
`design/intrada-design-system.dc.html`, is derived from it.

Read `docs/design-principles.md` before a new surface, layout, flow or
interaction, and append a decision there rather than settling a tension
silently. Write every user-facing string against `docs/tone-of-voice.md`; its
sweep checklist is the review pass for a PR that changes a string. Where the
two disagree, the design principle wins and the tone doc gets a new example.

## Eyes before ready

A change to what a musician sees opens as a draft with the `just ios-run`
screenshot under **What it looks like** and the line "Waiting for Jon's word on
the screenshot". It goes ready only after Jon says yes, and the ready step
replaces that line with "Jon approved the screenshot on YYYY-MM-DD"; a no is a
push to the open PR (#1891). The Android half follows
`.claude/rules/android-quality.md`.

## Tokens, then modifiers, then components, then screens

1. Every colour, font, spacing and radius traces to `IntradaColor`,
   `IntradaFont`, `IntradaSpacing` or `IntradaRadius`: never a raw hex,
   `.padding(16)` or `cornerRadius: 12`. Genuine one-offs (a fixed component
   height, a 2pt baseline nudge) stay literal.
2. Reuse before creating: read the component's page in
   `design/components/README.md` before using or changing it; a change to a
   component starts with its page (#2620). Parts
   not listed yet live in `ios/Intrada/DesignSystem/` and
   `ios/Intrada/Views/Components/`: check both before building anything.
3. Every top-level screen is built from `ScreenScaffold`, which owns chrome,
   safe areas and background.
4. If a primitive almost fits, add a parameter to it. Extract a shared
   primitive the moment a second screen would copy one. Never ship a parallel
   one-off: hand-rolled copies are the main source of visual drift.
5. Typography through `IntradaFont`'s five sizes and no others (T35): nothing in
   capitals, no screen-local size; section titles, field labels and toggle titles go through
   `SectionTitle` and `FieldLabel`. Spacing, icon sizes, opacity, shadows and
   motion come from their `Theme.swift` and `Motion.swift` tokens too.

New screens are built from the catalogue components (`design/components/README.md`)
and theme values only; `scripts/check-theme-values.sh` fails a raw colour or
spacing number (#2699). A value the theme lacks becomes a token in both theme
files; the allowlist holds only what existed on 18 October 2026 and shrinks
under #2706.

Deviation is allowed only in an explicit redesign, a flagged conversation
(Claude Design first, then the plan comment) that updates the tokens and
primitives in `Theme.swift`, not a clone in one view.

## Animated reveals need an opaque backing

Anything that slides or fades over other content paints an opaque token
(`paperTop` or `cardFill`, never `clear`), or the transition ghosts. Chrome it
emerges from behind is also opaque and sits on top (`.zIndex(1)`).

## Native feel

- Haptics only through `Haptic` in `Store+Feedback.swift`: `selection` for
  tabs, `impact` for taps, `warning` for destructive confirms, `success` only
  after the core confirms, through `store.send(_:onSuccess:)`.
  `scripts/check-haptics.sh` enforces it (#2013).
- iPad list-to-detail screens use `ListDetailSplit`, built with the view.

Design happens in Claude Design (`docs/design-workflow.md`). Mock against the
existing kit first; for something new, update `Theme.swift` and the design
reference (with its `support.js`) together, and re-export
`design/intrada-design-system.html`.
