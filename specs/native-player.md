# Native Active-Session Player (#932)

> Design pass for the native SwiftUI player — the build→practise→reflect loop.
> Tier 2 (SwiftUI + navigation over an **already-complete** core; no FFI, no
> schema, no new domain events). Resolves the long-parked **focus-vs-reflection
> split**. Companion to [`native-ios.md`](native-ios.md) Phase C-Practice.
>
> Decided 2026-06-05 via brainstorm. Visual direction to be rendered in Pencil
> before implementation, per the project's Pencil-first convention.

## Problem

The builder's "Start session" is a present-but-disabled "Coming soon" frontier:
`StartSession` already transitions `Building → Active`, but nothing renders
`ViewModel.active_session`. We need the native player — and more than the
mechanical MVP, we need to resolve how two opposing design forces coexist in it:

- **"The app disappears during practice"** (design-principles §B, T2) — the live
  screen stays bare; the user operates the music, not the interface.
- **"Spend friction deliberately"** (§A, T1) — intention *before* an item and
  reflection *after* it are **good friction**, the place the product's value
  lives. Deliberately kept, sometimes added.

The seam between them is the **transition between items**. That seam is the
design, and this doc settles it.

## What is already done (the core owns it all)

The entire `Building → Active → Summary` lifecycle exists and is tested in
`intrada-core/src/domain/session.rs`. **Slice 1 is a pure rendering job** — no
FFI, schema, or domain-event changes. The only possible core touch in the whole
feature is a one-line phase-guard widening for per-entry intention in Slice 2
(see Surface 2), TDD'd and kept dumb-pipe — not a new event or capability.

- Events: `NextItem`, `SkipItem`, `FinishSession`, `EndSessionEarly`,
  `AbandonSession`, `AddItemMidSession`, `RepGotIt`/`RepMissed`/`InitRepCounter`,
  `UpdateEntryScore`/`Tempo`/`Notes`, `UpdateSessionNotes`, `SaveSession`,
  `DiscardSession`, `RecoverSession`.
- `ActiveSessionView` exposes: current item title/type, position/total,
  `current_item_started_at` (timer anchor — elapsed is derived, not counted, so
  it survives backgrounding), `next_item_title`, full current-rep state,
  `current_planned_duration_secs`, `session_intention`, all entries.
- `SummaryView` exposes total duration, completion status, all entries, notes.
- Mid-session reflection is **already supported**: `UpdateEntryScore`/`Tempo`/
  `Notes` are accepted in **both** Active and Summary phases — the core was built
  for a per-item reflection beat, not only an end-of-session summary.
- The active session is persisted on every advance
  (`AppEffect::SaveSessionInProgress`) for crash recovery.

## The decided design

### Reflection model: per-item interstitial

After each item, a deliberate **transition beat** reflects on what was just
played and sets intention for the next. (Not end-of-session-only.) This is the
richest "good friction" and what the core's dual-phase write support was built
for.

### The spine (Arc 1 — symmetric)

```
Builder
  └─[Begin]→ Aim (item 1)         ← opening half-beat: set an aim before playing
            → Focus (item 1)       ← play
            → Beat (Reflect 1 · Aim 2)   ← full beat between items
            → Focus (item 2)
            → … → Focus (item N)
            → Reflect N            ← closing half-beat
            → Summary (note · Save)
```

Intention is a **ritual the app insists on**: every item gets an aim, set in the
moment (not front-loaded into the builder's admin phase). The opening and
closing half-beats are the only asymmetric points; the middle beats are uniform.

### Surface 1 — Focus screen ("calm card")

The live, while-playing screen. Bare, but **anchored to intention**:

- The **intention you just set**, echoed as a quiet one-line reminder.
- Item title + type, position (`2 of 5`) with a thin progress bar.
- A **count-up** timer (open-ended — play as long as it serves). If the item has
  a planned duration it's a *faint reference*, never a hard countdown that nags.
- **Reps**: only when the item has a rep target — surfaces `Got it / Missed`
  controls on the card. Items without a target stay bare.
- One primary action: **Done** (→ the transition beat). `Next ·  <next title>`
  hinted faintly beneath.
- `•••` reveals secondary/destructive actions (see Mechanics).

### Surface 2 — Transition beat ("one rich screen")

One screen, two halves, one CTA — the full reflective moment held in view:

- **Just played** (top): item title, "How did that go?" → score pips
  (`UpdateEntryScore`), opt-in `+ tempo` (`UpdateEntryTempo`) and `+ note`
  (`UpdateEntryNotes`).
- **Up next** (bottom): next item title + an aim field (`SetEntryIntention`…
  see note), CTA **Start <next> ›**.
- Fields are optional — you can score and advance in one tap — but everything is
  visible (not hidden behind expanders).

Note: per-entry intention during the Active phase. `SetEntryIntention` is
currently a Building-phase event; the beat's "aim for next" either reuses it
(verify it's accepted/extended for Active) or the aim is captured and applied
when the next item becomes current. Confirm the exact event wiring in Slice 2
planning — no new *capability* is needed, but the event's phase guard may need a
one-line widening (a core change, TDD'd, kept dumb-pipe).

### Surface 3 — Summary ("quiet ledger")

Reached after the closing reflection, so per-item scores are already captured.
This screen **reviews and saves**:

- Warm one-line header (`Nice work · 38 min · 4 of 4 done`). No analytics.
- The items as they happened: title, type dot, duration, score pips. Skipped /
  not-attempted shown muted. Entries stay **tappable** — a last chance to nudge
  a rushed score (the core accepts edits in Summary phase).
- A whole-session note field (`UpdateSessionNotes`).
- Primary **Save session** (`SaveSession`); secondary **Discard**
  (`DiscardSession`).

Cross-session highlights ("strongest item", "tempo gains") are **out of scope** —
they belong to the Track pillar where the cross-session data lives, not
half-baked here.

## Mechanics

1. **Full-screen takeover.** The player is a `fullScreenCover` from the builder —
   no tab bar, no nav chrome. The app *becomes* the session; you return to the
   Practice tab only after Save/Discard.
2. **Exit behind the `•••`** (T2). The card stays bare; the menu reveals *End
   session early* (`EndSessionEarly` → closing reflection → Summary, marking the
   rest not-attempted), *Skip this item* (`SkipItem`), *Add an item*
   (`AddItemMidSession`), and mid-session config (duration/reps). Nothing
   destructive is one tap.
3. **Resume after crash/quit.** On relaunch with a session in progress, the
   Practice tab offers **Resume / Discard** (`RecoverSession` / `AbandonSession`)
   — the existing #197 flow rendered native. No core work.
4. **Timer is count-up; planned duration is a soft guide.** No nagging countdown.

## Scope — two slices

**Slice 1 — the loop closes (satisfies #932).**
Focus card + `Next`/`Skip`/`End early` + `fullScreenCover` presentation +
quiet-ledger Summary + `Save`, wired to the builder's Start button. Reflection
captured at the Summary only (scores/notes there). Resume/Discard on relaunch.
A complete, shippable build→practise→save loop.

Per-screen quality baked in (native-ios.md): swift-snapshot tests, VoiceOver +
Dynamic Type, iPad consideration. Snapshot the load-bearing states (focus card
with/without reps, summary completed/ended-early).

**Slice 2 — the beats (the soul, follow-up PR/issue).**
The per-item transition beat (reflect + intend), the opening aim half-beat, and
the intention echo on the focus card. The "good friction" layer landing on a
working loop. Includes confirming the per-entry-intention event wiring for the
Active phase (above).

## Deferred — to track as issues at implementation time

- **Slice 2 (the transition beats)** — own issue, `pillar:practice`, `ios`,
  `horizon:next`.
- **Per-entry intention in Active phase** — verify/widen `SetEntryIntention`'s
  phase guard (folds into Slice 2).
- **Summary highlight line / reward pulse** — revisit when the Track pillar
  lands; explicitly *not* in the player.

## Non-goals

End-of-session-only reflection; a hard countdown timer; analytics on the summary;
any core/FFI/schema change in Slice 1; persisting per-item intention to a new
field (the existing `intention` on `SetlistEntry` carries it).
