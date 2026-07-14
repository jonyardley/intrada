# Reflection & narrative — Phase 1 · direction explorations & decisions

Brief: `design/briefs/2026-07-reflection-and-narrative.md`.
Mockups: this folder, one `.dc.html` per surface. The annotated journey view
(all four frames + designer notes) is `design/Reflection Narrative.dc.html`.

## Hard constraints honoured (from the brief)
- Mid-item capture never forces an advance; returns straight to the running timer.
- The player stays bare — anything new is one gesture down, not resident chrome.
- End-of-session reflection is skippable and never gates saving.
- One primary action per screen.
- Tokens only — no new colours or spacing; new primitives flagged, not invented.

---

## Surface 1 — Mid-item quick capture (`focus-player-quick-capture.dc.html`)

Directions explored:
- **A. Resident note button in the transport row** — rejected: resident chrome
  on the bare player, the exact thing the brief bans.
- **B. Compact bottom sheet, one swipe up** — ✅ recommended. Reuses the
  hand-off sheet's grammar (handle, paper surface, dim scrim); the timer ring
  stays visible and ticking behind it; Save/Close land back on the running
  timer; also reachable via •••.
- **C. Long-press the item title → inline field** — rejected: invisible
  affordance, collides with future title actions.

Details: note stamped at gesture time ("at 3:12" chip, gold); one primary
action (Save note); Close is a text button; swipe-down also dismisses.
This decision sets the app's capture grammar — sheets over a live player —
which surfaces 2–4 follow.

## Surface 2 — Achieved tempo on the hand-off sheet (`reflection-sheet-tempo.dc.html`)

Directions explored:
- **A. Stepper prefilled at the item's target** — ✅ recommended. Most results
  land a few taps from target; 44×44 buttons; ±2 bpm; clamp 40–208; no
  keyboard mid-sheet. Untouched = a true "played at target".
- **B. Free numeric field** — rejected: summons a keyboard over the sheet.
- **C. Slider** — rejected: hopeless precision across ♩ = 40–208.

Placement: between ScoreSelector and the note. "Skip rating" still skips
everything. **Flag: TempoStepper is the one new primitive candidate** — built
from existing tokens (hairline `#D8D0BD`, card `#FCFAF3`, radius 12); goes into
`Theme.swift` + the DS together if accepted.

## Surface 3 — Structured reflection on the summary (`session-summary-reflection.dc.html`)

Directions explored:
- **A. Inline on the summary** — ✅ recommended. Intention echoed verbatim
  (gold card, serif italic), then three prompt rows: **Improved** (green ·
  trending-up) / **Still rough** (gold · wrench) / **Next target** (indigo ·
  target). Blank rows save as nothing.
- **B. Post-save interstitial** — rejected: gates the primary action.
- **C. Single free-text** — what ships today; loses the structure the brief asks for.

To make room, "What you played" collapses to a one-line row (mastery move in
its subtitle); tap expands the shipped ledger unchanged. **Save session stays
the only primary action.** Tweak `reflectionFilled` shows the empty state.

## Surface 4 — Session intention in the builder (`session-builder-intention.dc.html`)

Directions explored:
- **A. Optional single line under the "Build session" header** — ✅ recommended.
  Intention-before friction spent where the plan is made; feather icon, gold;
  placeholder "What's today about? One line."
- **B. Inside the sticky Start bar** — rejected: competes with the one primary action.
- **C. Interstitial after tapping Start** — rejected: a whole screen for one sentence.

The text threads to the summary echo (surface 3) — `intentionText` tweak
drives both mockups.

---

## Fold-in list on sign-off (per design-process.md)
1. **TempoStepper** → `Theme.swift` + DS catalogue (the one new primitive).
2. **Compact BottomSheet detent** → variant on the shared BottomSheet.
3. **ReflectionPromptRow** (icon circle + eyebrow + line) → DS when reused.
4. Intention field pattern; builder/summary pillar updates in the DS gallery.

## Implementation notes for the core
- Mid-item capture writes notes on a **pending** entry (the hand-off sheet
  writes on a completed one) — may need a notes-append/timestamped-note event.
- Achieved tempo rides with the existing entry-update events at hand-off
  (order per #1042: notes → NextItem → score).
- Intention + the three reflection prompts are session-level fields.
