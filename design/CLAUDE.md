# Intrada: project notes

## Process: read first, every iteration
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
- **Dark mode is parked, not dropped**: revisit once the app reaches MVP. A dark
  variant of the Focus Player / Library / Practice exists in `Intrada Concepts.dc.html`
  (the "After dark" section) as proof the tokens invert cleanly.

## Files
- `design-process.md`: **design file process and guidelines.** The rules for where
  things live and how to keep files in sync. Reference it before designing or
  folding in.
- `Intrada Design System.dc.html`: the design system page, a picture of
  `Theme.swift` rebuilt from `briefs/2026-10-design-system-rebuild.md` (#1678).
  Paper #F7F4EF, near-neutral ink #2A2725, the eight highlighters with their hero
  gradients, Hanken Grotesk and DM Mono, 3pt corners and the faint card lift
  (ink at 5%, blur 1, y 1). Its values come only from that brief; where it and
  `Theme.swift` disagree, `Theme.swift` wins.
- `Focus Player.dc.html`: a **shared screen** extracted to one importable DC (the
  Option-B pattern), mounted via `<dc-import name="Focus Player">` in the
  related-items journey. The design system page no longer embeds screens. Edit it here, once. New shared screens follow
  the same pattern (see `design-process.md` §9).
- `Visual Direction Exploration.dc.html`: the exploration that chose the 8f look
  (#1676).
- `Intrada Reskin.dc.html`: the 8f token sheet.
- `App Icon.dc.html`: the final app icon and the swipe placements tried for it.
- `Welcome.dc.html`: the shipped first run (welcome, profile, first piece, Start here card), with the unbuilt animated intro kept below.
- `Faint Ink 1941.dc.html`: metadata text moving from faint ink to secondary ink
  (#1941).
- `Intrada Concepts.dc.html`: exploratory/validated screen concepts (Progress, Focus
  Player with rep counter, one-tap+calendar Practice, Library mastery, session-summary
  celebration, after-dark variant, live motion lab).

## Motion
- Named tokens live in `Theme.swift` (`IntradaMotion`): `standard`, `snappy`,
  `gentle`, `fadeUp`, `barGrow`, `countUp`, `pop`, and the `press` dip. Under
  Reduce Motion every reveal renders its final state with no animation (#2239).
- **Retired (do not reintroduce):** `breathe` (ambient ring glow) and `metro` (tempo
  pulse dot): read as distraction, pulled.
- Keep rings/content calm and static; motion earns its place only when it carries
  meaning (progress, state change, celebration).
