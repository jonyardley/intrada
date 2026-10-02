# Intrada — project notes

## Process — READ FIRST, every iteration
- **`design-process.md` is the canonical process** for organising design files and
  keeping one authoritative view. Follow it on every change: single ownership per
  surface, components + canonical pillar screens in the design system, journeys in
  feature files, fold-in as a one-way ratchet. Run its per-iteration checklist
  before sign-off and record winning-design decisions.

## Theme decision (2026)
- **Light is the MVP default.** The look is "8f" (T24 and T27 in
  `docs/design-principles.md`): paper #F7F4EF, white cards with 3pt corners and a
  faint lift, near-neutral ink #2A2725, and one highlighter as the only bright
  colour, chosen by the musician from eight (butter #FFE9A3 by default). Hanken
  Grotesk for titles and body, DM Mono for metadata.
  `ios/Intrada/DesignSystem/Theme.swift` holds every value.
- **Dark mode is parked, not dropped** — revisit once the app reaches MVP. A dark
  variant of the Focus Player / Library / Practice exists in `Intrada Concepts.dc.html`
  (the "After dark" section) as proof the tokens invert cleanly.

## Files
- `design-process.md` — **design file process & guidelines.** The rules for where
  things live and how to keep files in sync. Reference it before designing or
  folding in.
- `Intrada Design System.dc.html`: the design system page, with the component
  catalogue and motion. It is behind the app until #1678 rebuilds it from
  `briefs/2026-10-design-system-rebuild.md`; where it and `Theme.swift` disagree,
  `Theme.swift` wins.
- `Focus Player.dc.html` — a **shared screen** extracted to one importable DC (the
  Option-B pattern): mounted via `<dc-import name="Focus Player">` in both the design
  system and the related-items journey. Edit it here, once. New shared screens follow
  the same pattern (see `design-process.md` §9).
- `Drill Loop.dc.html` — **practice-coach Session A journey** (3 Aug 2026): A2 during
  play + A3 after a repetition (full, mobile + iPad), A1 Home + A4 block boundary
  (rough passes for Phase 2a). History: the coach work was removed in #1344, and its
  primitives leave the design system with #1678.
- `Visual Direction Exploration.dc.html`: the exploration that chose the 8f look
  (#1676).
- `Intrada Reskin.dc.html`: the 8f token sheet.
- `App Icon.dc.html`: the final app icon and the swipe placements tried for it.
- `Welcome.dc.html`: the welcome animation into profile setup.
- `Faint Ink 1941.dc.html`: metadata text moving from faint ink to secondary ink
  (#1941).
- `Intrada Concepts.dc.html` — exploratory/validated screen concepts (Progress, Focus
  Player with rep counter, one-tap+calendar Practice, Library mastery, session-summary
  celebration, after-dark variant, live motion lab).

## Motion
- Named tokens live in the design system: `fadeUp` (signature page-load reveal),
  `pop`, `barGrow`, `toastIn`, `slideIn`, plus a Reduce-Motion rule.
- **Retired (do not reintroduce):** `breathe` (ambient ring glow) and `metro` (tempo
  pulse dot) — read as distraction, pulled. `glowPulse` (primary-CTA halo) is IN REVIEW.
- Keep rings/content calm and static; motion earns its place only when it carries
  meaning (progress, state change, celebration).
