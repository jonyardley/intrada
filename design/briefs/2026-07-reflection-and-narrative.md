# Design brief — reflection loop & narrative progress

> Handover to Claude Design, 2026-07-14, from the vision/journey audit.
> Work against `design/intrada-design-system.dc.html` (tokens canonical in
> `ios/Intrada/DesignSystem/Theme.swift`). Mockups land per
> `docs/design-workflow.md`: `specs/<feature>/design/<screen>.dc.html`.
> These are the surfaces for build phases 1 and 2 (`docs/journeys.md`,
> steps 7, 8, 9), plus two look-ahead surfaces worth rough passes now
> because they shape the Practice tab.

## Global constraints (non-negotiable)

- **T1**: intention-before and reflection-after are good friction; keep them
  deliberate, never pad them with admin.
- **T2**: the app disappears during practice; anything added to the player is
  one gesture down, never resident chrome.
- **T7**: mid-item capture never forces an advance; end-of-session reflection
  is structured, skippable, never a gate on saving; everything captured must
  be re-readable later.
- One primary action per screen. Comeback framing, never shame. Every screen
  ships with VoiceOver labels + Dynamic Type.

## Phase 1 — the reflection loop

### 1. Mid-item quick capture (Focus Player)

A musician notices something in bar 12 and wants it out of their head and
back to the keys in under five seconds.

- Current: notes exist only in the hand-off sheet, and saving one advances
  the item. The core already accepts mid-item notes; this is pure surface.
- Design questions: where does the affordance live (player chrome vs the
  options menu vs a gesture)? What is the capture surface: a sheet, an inline
  field over the timer? A second capture on the same item: append visibly, or
  edit the existing note? Does the timer visibly keep running during capture
  (it should feel like it never stopped)?

### 2. Reflection hand-off sheet + tempo (Focus Player)

The existing between-items beat (score 1–10 + note) gains achieved tempo.

- Current: `ReflectionSheet` is score + note + "Save & continue".
- Design questions: how does tempo enter without bloating the beat: stepper,
  numeric pad, prefill from the item's target? Is it visible by default or
  only when the item has a tempo target? (Lean: only when relevant;
  progressive disclosure.)

### 3. Structured end-of-session reflection (Session Summary)

Three prompts: what improved, what's still broken, what to target next time,
shown against the session's stated intention.

- Current: one unlabelled free-text box.
- Design questions: three fields or one guided flow? How does the stated
  intention get echoed (header quote, ghost text)? How does "skippable" look
  without reading as "skip me"? Where does the next-target answer preview
  that it will resurface ("we'll suggest this as the aim next time")?

### 4. Session intention (Session Builder)

"What is this session for?" — set once at build time, echoed in the player
and at reflection.

- Current: the core supports it; iOS never sends it. The per-entry Aim
  exists; this is the session-level sibling.
- Design questions: where in the builder does it live without becoming a
  form? Is it ever prompted, or purely optional? Relationship to a goal
  (phase 4): does picking a goal prefill the intention?

## Phase 2 — narrative progress

### 5. Session detail (new screen)

Past sessions become tappable: a read-only recap of intention, per-item
scores and notes, session reflection.

- Current: `SessionCard` rows are dead ends; everything written is invisible
  after saving.
- Design questions: how much hierarchy does a recap need (it is a reading
  surface, the first one in the app)? Does it reuse the Summary's recap rows
  or want a calmer, journal-like treatment?

### 6. Item note history (Library detail)

An exercise's own timeline of what you said about it, next to its score ring.

- Design questions: interleave notes with the score history or a separate
  list? How many entries before truncation + "see all"?

### 7. Progress, with your words in it (Progress tab)

Reflections quoted next to the quantitative deltas; aim-attainment as the
headline stat; cold-score trends when cold scoring lands.

- Design questions: what does a quote look like in the Paper &amp; Score
  language (this is the surface where the serif voice could earn its keep)?
  How do words and numbers pair in one card without clutter? What is the
  empty state before enough reflections exist?

## Look-ahead (rough passes only)

### 8. Goals v1 surfaces (phase 4)

A rough exploration exists (Claude Code artifact "Goals v1 sketch",
2026-07-14; ask Claude Code to re-share): Practice-tab goal card, creation
sheet, item-detail chip. Redo properly against the kit when phase 4 nears.
The shape is locked (statement + linked items + optional freeform target);
the design question is how quiet the Practice-tab presence can be while
still framing the recommendation card.

### 9. Today's plan (phase 5)

The recommendation card: named entries with one-line reasons, one-tap start,
"tweak it instead" as the escape hatch. Worth a rough pass now because it
competes with the hero for the Practice tab's one primary action; decide
early which wins when a plan exists.
