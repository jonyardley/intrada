# intrada Design Principles

> Living document. Started 2026-05-31. These are **guiding principles**, not
> hard gates — they set direction and frame decisions. When a principle pulls
> against another (or against what the app does today), that's a decision to
> make deliberately, not a rule to mechanically enforce. Document those
> decisions in the **Open tensions & decisions** log at the bottom.
>
> Visual/token detail lives in code (`ios/Intrada/DesignSystem/Theme.swift`)
> and the design rules in `CLAUDE.md`. This doc is the *why* and the
> *interaction* layer those don't cover.
>
> What a screen *says* is decided by [`tone-of-voice.md`](tone-of-voice.md), the
> writing layer of this doc. It inherits these principles; check it before
> writing any user-facing string.
>
> Design is now produced in **Claude Design** — see
> [`design-workflow.md`](design-workflow.md). The native iOS design system lives
> at [`design/intrada-design-system.dc.html`](../design/intrada-design-system.dc.html),
> derived from `ios/Intrada/DesignSystem/Theme.swift`. (Pencil is retired.)

## How to use this doc

- Designing a new surface? Read the interaction principles first, then check
  the open-tensions log for anything relevant.
- Principles conflict? Don't silently pick one. Surface it, decide, and add a
  dated entry to the log so the next person inherits the reasoning, not just
  the outcome.

---

## Visual principles

These are mostly settled and live in code; captured here so the *intent* behind
them isn't lost. (See `CLAUDE.md` → Design System Rules for the enforcement
rules and primitive catalogue.)

- **Dark-on-dark glassmorphism.** Neutral gray-900→near-black gradient,
  whisper-soft white-opacity surfaces (3/5/12%). Backdrop-blur on *chrome only*
  (header, tab bar, overlays) — never on content surfaces.
- **Type is colour-coded.** Gold = Piece, blue = Exercise, teal = Set. The
  mapping repeats across gradient bars, badges, and inline indicators. Colour is
  an *accelerator* for recognition — never the only signal (see accessibility
  tension below).
- **Two accent families.** Warm indigo = interactive/primary; gold/amber = warm
  accent for achievements, streaks, progress.
- **Warmth bias in semantics.** Danger is warm coral, not pure red; success is
  warm teal-green. The palette leans warm even where convention is harsh.
- **Serif headings, sans body.** Source Serif 4 for page titles (signals a
  musical, editorial space); InterVariable for everything else.
- **Reuse before creating; extend, don't clone.** Hand-rolled markup that
  duplicates a primitive is the top source of visual drift.

---

## Interaction principles

The app's job is to get the user *practising*, not to make them operate
software. Every screen should feel like the shortest honest path to the thing
they came to do — except at the two moments where stopping to think *is* the
thing they came to do.

### A. Spend friction deliberately

- **Friction is a tool, not a defect.** There are two kinds, and they get
  opposite treatment:
  - *Bad friction* — admin, setup, planning, navigation, configuration. Remove
    it ruthlessly. The path to practising should be near-frictionless, intuitive,
    and never overwhelming.
  - *Good friction* — setting an intention just before a practice item, and
    reflecting in the moment right after each item. This is where the product's
    value lives. Preserve it; sometimes deliberately add it.

  The goal is not "fewest steps" everywhere. It's *no wasted steps in the admin,
  and deliberate, well-placed steps where they create meaning.*
- **Keep admin & setup flows short.** Adding a piece, building a routine, saving
  a set, navigating the library — these should feel near-frictionless. If one
  grows steps, treat it as a signal to rethink, not to accept.
- **One primary action per screen.** Each screen has one obvious next step,
  visually dominant (hero CTA / circular button). Secondary actions recede. Two
  actions competing to be "the" action means the screen is ambiguous.
- **Defaults beat configuration.** Every flow should be completable with zero
  setup. Sensible defaults up front; tuning is available but never required to
  proceed.
- **Direct manipulation where the platform allows.** Prefer acting on the thing
  itself (swipe, drag, tap-to-toggle) over opening a form. (Platform caveat in
  tensions log — web needs visible affordances.)

### B. Simplicity — defer complexity, don't remove capability

- **Progressive disclosure.** Show the common path first; advanced options live
  behind a sheet, "More", or Settings. The 80% case shouldn't pay the UI cost of
  the 20% case.
- **Reversible by default; confirm only the irreversible.** Reversible actions
  are instant, optimistic, and undoable (toast). Reserve confirmation dialogs for
  genuinely destructive, non-recoverable actions.
- **The app disappears during practice.** Active practice is the reason the app
  exists. Strip non-essential chrome (focus mode). The user operates the music,
  not the interface.

### C. No clutter — easy to reason about

- **Content over chrome.** Maximise the share of the screen showing the user's
  own data (pieces, sessions, progress) versus navigation and decoration.
- **Glanceable, not just readable.** A screen should be parseable at a glance.
  Lean on the type-colour system and consistent layout so users decode by
  pattern, not by reading every label.
- **Consistency is a simplicity tool.** The same gesture means the same thing
  everywhere; every list→detail behaves the same. Novelty in interaction is a
  cost the user pays, not a feature.
- **One concept per screen.** A screen answers one question or completes one
  task. The urge to add a second unrelated section usually means a second screen
  (or a sheet).

---

## Open tensions & decisions

The principles above pull against each other and against today's app in real
places. Each entry: the conflict, the options, and the decision (or `OPEN`).

### T1 — Where friction belongs: admin vs intention/reflection
**Status:** DECIDED 2026-05-31.
Resolved by the *spend friction deliberately* principle (§A). Setup, planning,
and admin are **bad friction** — make them near-frictionless and unoverwhelming.
But there is **good friction** we deliberately keep: setting an intention just
before each practice item, and reflecting in the moment right after it. A
pre-start review/intention step is therefore not "an extra step to eliminate" —
it's the good kind, *provided* it sits at the intention moment and isn't padded
with admin. Open follow-on: keep the *resume-a-known-routine* path short (don't
force the full builder when the user just wants to continue) — distinct from the
intention beat, which stays. Tracked: jonyardley/intrada#760.

### T2 — Mid-session configurability vs "the app disappears during practice"
**Status:** DECIDED 2026-05-31.
Mid-session editing of duration/reps/focus stays **one layer down** — revealed by
a deliberate gesture (e.g. tap-to-reveal), never persistent chrome. The default
live-session screen stays bare so focus mode holds. Config is reachable, not
resident.

### T3 — Decode-by-colour vs accessibility
**Status:** DECIDED (revisit) 2026-05-31.
**Colour is always an accelerator, never the sole carrier of meaning.** Every
type signal must also carry a shape or text cue (dot, badge text, icon). Today's
`InlineTypeIndicator` (dot + colour) and `TypeBadge` (text) already satisfy this.
Marked for possible revisit, but the rule holds for now.

### T4 — Direct manipulation vs one UI codebase (web + iOS)
**Status:** DECIDED 2026-05-31.
The principle is **"the most direct affordance the platform offers"**, with one
hard floor: **no action may be reachable *only* through a hidden gesture.** iOS
gestures (swipe-to-delete, long-press menu) are accelerators layered on top of a
visible, non-gesture path that exists on every platform (e.g. the action lives in
the detail screen / sheet too). Web users always get a visible affordance.
Follow-on: spot-check that every gesture-only action today also has a non-gesture
path on web (rolls into the T6 audit, jonyardley/intrada#761).

### T5 — Content over chrome vs the 2026 visual identity
**Status:** DECIDED 2026-05-31.
Chrome earns its space on **navigation/structural** surfaces (tab bar, headers,
the brand backdrop). On **content** surfaces, every visual element must serve
recognition, hierarchy, or navigation — not pure ornament. The type gradient
bars stay because they *encode type* (functional, not decorative). Test before
adding a visual flourish to a content screen: *does this help the user decode or
navigate?* If it's ornament only, cut it.

### T6 — Reversible-by-default vs today's confirm sheets
**Status:** DECIDED (principle) + audit pending, 2026-05-31.
Rule adopted: **recoverable actions → optimistic + undo toast; only genuinely
irreversible actions → confirm dialog.** New destructive actions default to the
undo-toast path unless the data loss is unrecoverable. Existing destructive
actions (e.g. item delete's confirm sheet) need a per-action audit to reclassify
— tracked as a follow-on issue.

### T7 — Reflection capture vs "the app disappears during practice"
**Status:** DECIDED 2026-07-14 (vision/journey audit).
Reflection is the good friction T1 protects, but capture must never cost flow.
Rules: **mid-item capture is one tap from the player, saves, and returns
straight to the running timer; it never forces an advance** (the current
hand-off sheet conflates "note" with "next"; that coupling is a bug, not a
design). **End-of-session reflection is structured**: what improved, what's
still broken, what to target next time, shown against the session's stated
intention, skippable, and never a gate on saving. What's captured must feed
forward: the "next target" answer surfaces as the item's suggested aim next
time, and reflections are re-readable (session detail, item history, progress
report). A note the user can never see again is admin, not reflection.

### T8 — Segmented pills are inputs, not a second view of a list
**Status:** DECIDED 2026-07-15 (#1081 B2).
`SegmentedPills` (and pill/tab controls generally) are for genuine **inputs** —
an either/or choice (the Piece/Exercise form toggle, `KindSegment`) or tag
selection. **Do not add a pill row as a filter over content already shown as a
list on the same screen** — that's two controls for one insight and reads as
clutter (violates principle C). B2 originally proposed context filter-pills on
the exercise detail to scope a session history, directly above a "By piece"
rows list that already carried the same per-piece breakdown; the pills were cut
and the rows made to carry it. When a list is on screen, let the rows (with
their own scores/meta, tappable) do the work; reach for pills only for choice
and tags.

### T9 — Session builder: direct manipulation always on, Edit is bulk remove only
**Status:** DECIDED 2026-07-15 (builder UX audit vs `design/Linked Exercises.dc.html`).
The builder's arrange/configure actions must not sit behind a mode switch.
Every line (block header, nested exercise, standalone) is its **own List row**,
so the platform's long-press lift reorders any of them outside Edit (static
grip glyphs advertise it), swipe-to-remove works on every row (mock frame 13),
and tapping a row opens its settings sheet (which carries the visible "Remove
from this session" path — T4's non-gesture floor). Edit/Done shrinks to
**bulk remove** (system minus-circles), per mock frame 14. Invented affordances
that deviate from the mock (chevron tap-steppers, Edit-gated settings glyphs)
were removed. This also **overturns decision 3 of
`specs/related-exercises-redesign.md` (2026-07-01)**: the per-exercise
"include today" toggle returns (off = stays visible but dimmed, excluded from
totals, dropped at session start) — needs a core event, tracked as
jonyardley/intrada#1101. Recorded deviation: the mock draws the block dragging
as one visual unit; here a block moves by its **header row** and the exercises
follow on drop. A custom in-card drag that preserved the unit visual lost a
touch-delivery race against the List's own lift (holding a nested grip lifted
the whole block), so consistency won: the same gesture means the same thing on
every row.

### T10 — The count-in draws on the during-play page, not the verdict glance
**Status:** DECIDED 2026-08-04 (jonyardley/intrada#1184, from playing #1183 at
the piano). RETIRED 2026-08-13. The surface this decided was removed with the
coach (#1344). Kept as log history.
The A3 glance after a tap is deliberate (one fact, nothing to read), but the
count-in is preparation for the *next* rep, so parking it on the page that
judged the previous one read as jarring. Ruling: the glance is the verdict and
gate dots alone, no count-in dots; the first count-in click turns the page to
A2 (tempo, click level, beat position), where the dots draw in place of the
stuck target. The first click is the boundary because it needs no new timer,
and the shortened glance (about half a second at 120bpm) is fine because the
verdict is the user's own tap, an acknowledgement rather than news. Options
considered: turning immediately on the tap lost the glance; keeping A3 for the
whole count-in kept the next rep's facts off screen until the hands were
already moving.

### T11 — The pulse runs for the block, and the beats go either side of it

**Status:** DECIDED 2026-08-06 (jonyardley/intrada#1223, #1224, #1225, from the
Phase 0 fortnight at the piano; design folded into
`design/intrada-design-system.dc.html` in #1232). RETIRED 2026-08-13. The
surface this decided was removed with the coach (#1344). Kept as log history.

**Supersedes the mid-block half of T10.** T10 ruled that the first count-in
click turns the page from the verdict glance back to the during-play facts.
There is no longer a count-in between reps to turn it: the click runs unbroken
for a whole block, counted in once at block entry. So the glance is ended by
the next beat instead, which lands in the same half-second at 120bpm and reads
the same way. Everything else T10 decided stands: the glance is still the
verdict and gate dots alone, and the count-in still draws on the during-play
page, in place of the stuck target, wherever a count-in does happen (block
entry, and after a ladder rung that changed the tempo or the phrase).

Why: the per-rep restart was a cost of deferring machine listening (decision
18), not a design goal. The original loop was continuous (hands never leave
the keys), and stopping the click to ask a question each pass turned a
practice loop into a quiz. The verdict window now opens at each phrase
boundary and rests for one pass while the hands play the next, so the tap is
still a verdict on one bounded attempt (decision 17 is untouched) and missing
one costs a dropped rep rather than a frozen screen.

Three surfaces come with it, all instances of T1: intention friction is worth
keeping, admin friction is not.

- **The block-entry card.** Blocks stop auto-advancing silently. Every block,
  including the first, opens on a silent card carrying four facts and one
  sentence: kind, minutes, drill title with its section, and the plan's own
  why line. Start is the single primary; Skip sits under it with no border or
  fill, because skipping is cheap and the eye should not be drawn to it. This
  is T1's intention beat, and it is where the breath between blocks goes now
  that the loop itself has none. Nothing on the card is configurable (T2) and
  nothing about the block just finished appears on it, so the beat cannot turn
  into a report card. Skip takes no confirm: it is reversible by the planner,
  and a confirm sheet is exactly the stall the card exists to remove (T6).
  Where a boundary CoachNote fires it resolves before the card, never stacked
  with it, so there is at most one thing to read at a boundary. The ceiling
  starts when the hands do, not when the card goes up.
- **Discard.** A false start is not a fail. "Don't count that" records nothing
  at all: no attempt, no evidence, no gate progress, no miss run. Without it
  the continuous pulse makes fumbles expensive, because the loop no longer
  pauses for you to gather yourself. It is a *cheap* action, not a confirmed
  one: reversible-by-default applied to the record rather than to the data.
- **Press start.** The hero shows the shape of the whole session, not just
  block one and a count: the upcoming blocks as read-only rows, and the lines
  the plan could not fit today, worded as the plan wrote them. Deferred lines
  are never dropped, because being told what is not in today is what makes the
  plan trustworthy enough to press start without editing it. The rows carry no
  chevron, no tap target and no reordering, since a per-block decision point
  before playing is a stall, not a choice (research foundation §7). Expanding
  the list shrinks the hero rather than pushing it off screen, so Start stays
  under a thumb at every disclosure state.

Options considered: keeping the per-rep count-in and shortening it (still
breaks the loop, still a quiz); letting the verdict window stay open across
passes until tapped (ambiguous which pass a late tap judged, which is exactly
the attempt bounding decision 17 protects); a confirm on Skip (rejected
above); a tappable block row opening a detail sheet (rejected, it promises a
screen that must not exist); keeping the "N blocks, about M minutes" footnote
alone (rejected, it tells you nothing about what you agreed to).

### T12 — The ramp is a default, not a wall (acquisition before the clock)

**Status:** DECIDED 2026-08-07 (decision 20 in the design doc, from planning
real tune-learning: shells and roots learnt out of tempo in chunks;
engine + planner work tracked in jonyardley/intrada#1244, #1245). RETIRED
2026-08-13. The surface this decided was removed with the coach (#1344). Kept
as log history.

New material opens at the least demanding rung the content authors (l0: no
click, chunked, from memory) and the loop ramps upward: chunks, joins, full
form, then the click below target tempo, then the target. A metronome at first
contact asks for time-keeping from hands still finding the notes; the demand
was arriving a week early.

The guiding-not-prescriptive rules, since familiarity differs per user, per
piece and per session:

- Prescriptions default to the ramp; nothing asks where to start.
- Chunk size is never a parameter: content authors the musical boundaries
  (sections, phrase marks), the block enters at a whole section, a fumble
  splits at the nearest boundary and clean passes merge. Two bars for one
  player's corner, the whole A for another's read-through, from the same
  authored structure.
- Skip on the entry card is the whole "I already know this" interface (T2: no
  mid-session configuration, and no questionnaire before playing).
- Passing a later gate retires the rungs beneath it, so the ramp can never
  hold back what the hands already own.
- Pace is inferred from verdicts alone: clean taps merge chunks and raise
  tempo; struggle falls onto the existing escalation ladder.
- The clock still arrives: an l0 gate that keeps passing gets its clocked
  sibling offered next plan, and the circling check watches acquisition that
  never ends.

Options considered: a "learning mode" toggle (rejected: decision 11, no mode
switches); asking the user to self-rate familiarity at block entry (rejected:
admin friction at the piano, T1, and self-assessed familiarity is exactly what
decision 17 quarantines); keeping acquisition outside the app as unmonitored
prose (rejected: the plan would skip the first week of any new material, and
the evidence from that week would be lost to the mastery track).

### T13 — Voice splits by surface, and the engine never speaks on screen

**Status:** DECIDED 2026-08-07 (graduated from
`design/briefs/2026-08-copy-language.md` v2, decided with Jon while reviewing
the Built Session A/B/C mockups; lands with jonyardley/intrada#1256). RETIRED
2026-08-13. The surface this decided was removed with the coach (#1344). Kept
as log history.

The first copy pass fixed vocabulary but kept the stance: the app narrated its
own data model in nicer words. The fix is not better sentences; it is fewer,
and different rules per surface class.

- **In-session — the silent tool** (drill loop, run-throughs, feel moments,
  off-piste, unmonitored play): prose is budgeted like taps, at most eight
  words beyond labels and buttons. No captions explaining mechanics: what is
  and isn't being recorded is carried by the interface itself (the altitude
  chip; the presence or absence of instrumentation). Buttons and chips are the
  vocabulary: "Held / Broke down", "Fought it / Getting there / It sang".
- **Set-up and composition — the plain peer** (steer sheet, resolution
  questions, composed-session view, altitude choice): one short sentence per
  card, maximum. Explain a thing once, at the decision point, then never
  again; repeat visits get labels only. Questions are fine when they are the
  actual decision.
- **Reflective and narrative — warmth allowed** (session summary, reflection,
  the morning proposal, the weekly thread): the user's own words do the
  emotional work, quoted in serif; the app's words stay brief and concrete
  around them. The only surface class where the app may have a personality.

Universal: engine vocabulary never appears on screen: gate, verdict,
evidence, mastery, prerequisite, node, steer, prescribed, judgement track,
countable, cold test (as a noun). Screen translations: target / the buttons
themselves / history / progress / "you added this" / "today's plan" / "by
ear" / "from cold" (as lived experience). Never explain the data model inline
(if a screen seems to need that paragraph, the design is wrong, not the copy);
no philosophy headlines; buttons name the action, not the mechanism ("Keep it
as a drill", never "write the gate").

The full spec, with worked per-frame examples, is
`design/briefs/2026-08-copy-language.md`; the design system gains a Voice
section beside the tokens so future frames are written against it.

Options considered: one app-wide voice with a softer register (rejected: what
reads warm at session close is chatter mid-drill; the surfaces have opposite
jobs); keeping the consent explanation as on-screen captions (rejected: the
deleted three-bullet consent list on A5 is the type specimen; the interface
carries it, or an ⓘ does); letting spec vocabulary through where it is
"technically accurate" (rejected: core says `variant`, the screen says
**Steps**, and the precedent holds everywhere).
