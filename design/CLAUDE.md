# Intrada — project notes

## Process — READ FIRST, every iteration
- **`design-process.md` is the canonical process** for organising design files and
  keeping one authoritative view. Follow it on every change: single ownership per
  surface, components + canonical pillar screens in the design system, journeys in
  feature files, fold-in as a one-way ratchet. Run its per-iteration checklist
  before sign-off and record winning-design decisions.

## Theme decision (2026)
- **Light is the MVP default.** Ship the "Paper & Score" light theme only for MVP.
- **Dark mode is parked, not dropped** — revisit once the app reaches MVP. A dark
  variant of the Focus Player / Library / Practice exists in `Intrada Concepts.dc.html`
  (the "After dark" section) as proof the tokens invert cleanly.

## Files
- `design-process.md` — **design file process & guidelines.** The rules for where
  things live and how to keep files in sync. Reference it before designing or
  folding in.
- `Intrada Design System.dc.html` — the living design system (light "Paper & Score"),
  derived from `ios/Intrada/DesignSystem/Theme.swift`. Canonical component catalogue + motion.
- `Focus Player.dc.html` — a **shared screen** extracted to one importable DC (the
  Option-B pattern): mounted via `<dc-import name="Focus Player">` in both the design
  system and the related-items journey. Edit it here, once. New shared screens follow
  the same pattern (see `design-process.md` §9).
- `Drill Loop.dc.html` — **practice-coach Session A journey** (3 Aug 2026): A2 during
  play + A3 after a repetition (full, mobile + iPad), A1 Home + A4 block boundary
  (rough passes for Phase 2a). Its six primitives are canonical in the design system
  under *Components · Coach primitives*; the feature file holds only the journey.
  A2/A3 and the primitives are **built** in SwiftUI
  (`ios/Intrada/DesignSystem/Coach/`, `ios/Intrada/Views/Screens/DrillScreen.swift`).
  Nine primitives are now canonical in the design system: `TapVerdict` folded in
  6 Aug, with `BlockEntryCard` and `PlanBlockRow` from the continuous-pulse
  rework. Seven have Swift; `BlockEntryCard` and `PlanBlockRow` are marked
  TO BUILD in the catalogue, so the design system is their whole spec until
  #1223/#1225 land.
- **Coach entry surfaces (6 Aug 2026, #1223/#1225)** — the block-entry card and
  the press-start session overview live in the **design system** (Screens, marked
  TO BUILD, with a "how they compose" panel), not in a feature file. The Claude
  Design project's `Drill Loop.dc.html` still carries the working `#pulse`
  section they came from: strip it to a reference on the next design pass, per
  the fold-in ratchet.
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
