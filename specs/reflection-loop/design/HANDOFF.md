# Handoff — Reflection & narrative, Phase 1 (brief: design/briefs/2026-07-reflection-and-narrative.md)

## Commit
- Copy `Reflection Narrative.dc.html` into `design/` (siblings `support.js` and
  `Focus Player.dc.html` already live there — the file imports the Focus Player
  via `<dc-import>` and loads `./support.js`, so it must sit next to them).
  If you keep it under `specs/NNN-reflection-and-narrative/design/`, copy those
  two files alongside it or the frame won't render.
- Open in a browser (or Claude Design) to review; four annotated phone frames.

## The four surfaces — decisions to implement

### 1. Mid-item quick capture (Focus Player)
- A **compact bottom sheet**, one swipe up from the bare player (also via •••).
  No resident chrome added to the player.
- Contents: "QUICK NOTE" label, timestamp chip (stamped at gesture time, e.g.
  "at 3:12"), 3-row note field, primary **Save note**, text **Close**.
- Timer never pauses; sheet never advances the item; Save/Close return to the
  running timer. Wire to the existing `updateEntryNotes` path (mid-session
  writes on a not-yet-completed entry — note this is a *pending* entry, unlike
  the hand-off sheet's completed-entry writes; may need a notes-append or
  timestamped-note event in core).

### 2. Hand-off sheet + achieved tempo
- Insert a **TempoStepper** row between the ScoreSelector and the note field:
  label "TEMPO REACHED · target ♩ = N", 44×44 −/+ buttons (±2 bpm, clamp
  40–208), value prefilled at the item's target.
- **New primitive candidate** — flagged, not invented: hairline border
  `#D8D0BD`, card surface `#FCFAF3`, radius 12, Inter 24 semibold tabular.
  If accepted: add to `Theme.swift` + the design system in the same PR.
- "Skip rating" skips everything, unchanged. Save order per #1042 stays
  (notes → NextItem → score; tempo rides with the entry updates).

### 3. Structured end-of-session reflection (Session Summary)
- New blocks above the ledger: **intention echo** (gold card, serif italic,
  quotes the builder intention verbatim) + three prompt rows:
  **Improved** (green/trending-up) · **Still rough** (gold/wrench) ·
  **Next target** (indigo/target). Single-line each, all optional.
- Blank prompts save as nothing; **Save session stays the only primary action**
  — reflection never gates saving.
- "What you played" collapses to a one-line row (count + mastery move in the
  subtitle); tap expands the shipped ledger rows unchanged.

### 4. Session intention (Session Builder)
- Optional single-line field directly under the "Build session" header:
  label "TODAY'S INTENTION · OPTIONAL" (feather icon, gold), standard input
  chrome. Placeholder: "What's today about? One line."
- Text is stored on the session and echoed verbatim by surface 3.
- Rejected: in the sticky Start bar (competes with the one primary action)
  and a post-Start interstitial (a screen for one sentence).

## Hard constraints honoured (from the brief)
- Capture never forces an advance; returns straight to the running timer.
- Player stays bare — everything new is one gesture down.
- End-of-session reflection is skippable and never gates saving.
- One primary action per screen. Tokens only — zero new colours/spacing.

## Fold-in list (on sign-off, per design-process.md)
1. **TempoStepper** → Theme.swift + DS catalogue (the one new primitive).
2. **Compact BottomSheet detent** → variant on the shared BottomSheet.
3. **ReflectionPromptRow** (icon circle + label + line) → DS if reused.
4. Intention field pattern; summary/builder pillar updates in the DS gallery.
5. Strip superseded frames from this journey file after fold-in.
