# Design brief — the practice-coach drill loop

> Handover to Claude Design, 2026-08-03, from the practice-coach design phase.
> Work against `design/intrada-design-system.dc.html` (tokens canonical in
> `ios/Intrada/DesignSystem/Theme.swift`). Mockups land under
> `specs/<feature>/design/<screen>.dc.html` per `docs/design-workflow.md`.
> Design context: `specs/intrada-practice-coach-design.md` (v5) and the ten
> scenarios in `docs/coach-user-journeys.md`.
>
> **Note on `docs/journeys.md`:** earlier briefs cite its numbered steps. It is
> retired — the notebook-era journey. Cite `docs/coach-user-journeys.md` instead.
>
> **Run this as two sessions.** Session A is the drill loop: four screens seen
> every single day, and the ones that must be right. Session B depends on A's
> decisions about how the app speaks. Seven screens in one pass produces seven
> mediocre screens.
>
> **Within Session A, only two screens are being built now.** Phase 1 ships the
> drill screen and nothing else: **A2 (during play) and A3 (after a repetition)
> get the full treatment.** A1 (Home) and A4 (block boundary) arrive in Phase 2a,
> so give them **rough passes only** — enough to establish the voice and prove the
> visual language holds across the loop, not polished comps. Same pattern as the
> 2026-07 reflection brief, which rough-passed two look-ahead surfaces because
> they shaped the surrounding tab. Polishing a screen months before it is built is
> how mockups go stale, and A1/A4 will be better designed once you have used A2 and
> A3 at a real piano.

## The product in three sentences

The app decides what you practise, listens over MIDI while you play, scores each
attempt against a known target, and opens a gate when you're done. The user is
seated at a piano, hands on the keys, phone or iPad on the music stand, often
mid-flow. It is a quiet colleague, not a cheerleader or an invigilator.

## Global constraints (non-negotiable)

- **One screen, one action**, at every moment. No dashboards mid-session, no
  navigation bait.
- **T2 / T7**: the app disappears during practice. Anything added to the drill
  screen is one gesture down, never resident chrome.
- **Silence while playing** is a design requirement, not an omission. No live
  scoring, no wrong-note flashes. Verdicts land *between* reps.
- **Orientation is not feedback** (spec decision 15). Time elapsed, this block's
  ceiling and blocks-remaining are *always* visible and carry no judgement. The
  thing that must never appear mid-drill is a score.
- **T3**: never decode by colour alone. A miss is taupe, never red — calm, not
  shaming. Mastery is monochrome; the *count* carries meaning.
- **No gamification vocabulary or ornament**: no XP, gems, lives, badges,
  confetti. The reward is the trend line and the sound of your own playing.
- **Light theme only** (`design/CLAUDE.md`) — dark is parked. Practice rooms are
  dim; solve legibility with contrast and type size, not a dark variant.
- **Musician's language, British English.** "3 clean passes at 120", "left hand
  late". Never "temporal deviation metric".
- Every screen ships with **VoiceOver labels and Dynamic Type**, and must stay
  legible at arm's length in poor practice-room light. Show the largest
  accessibility size for the drill screen — that is the one read from a metre away.
- **iPad regular-width** designed *with* each screen, not retrofitted.

## Session A — the drill loop

### A1. Home — "ready when you are"  · *rough pass (Phase 2a)*

*The moment:* the user has sat down with twenty minutes and does not want to make
a single decision.

The entire screen is a start button. Today's prescribed plan in one line, its
one-line reason beneath, and a secondary "I've only got 10 minutes" offered as an
equal option rather than a downgrade. A quiet one-tap way to say "something else
today" must exist without shouting.

- **Also design the first-run state** (journey 1): no history, no trend. The app
  says *early days, still learning your level* and asks only how long today.
- *Failure mode to avoid:* a dashboard. Any stat that isn't today's plan is noise.

### A2. During play — deliberately empty  · **full treatment (Phase 1)**

*The moment:* hands on the keys, four bars into an attempt, eyes mostly on the
keyboard.

Drill name, tempo, which click level is running, a passive bar/beat position
indicator, and ambient orientation (elapsed / ceiling / blocks left). One large,
easy target: **I'm stuck** — hittable without looking.

- The orientation must read as information, never as a countdown or a verdict.
- *Failure mode to avoid:* anything that rewards looking at the screen. If it
  pulls the eyes, it ruins the next phrase.

### A3. After a repetition — one glance, about a second  · **full treatment (Phase 1)**

*The moment:* the attempt just ended; the next count-in starts on its own.

Tick or cross, at most **one** fact ("clean", "2 wrong notes", "rushing bars
3–4"), and gate progress as filled/empty dots with "2 of 3". Nothing to tap.

- **Design the cross state with equal care** — factual, calm, never shaming.
- **Design the uncertain variant**: when the app isn't sure, it asks rather than
  asserts — *"that sounded clean — agree?"*, one tap. Being asked is respectful;
  being wrongly failed is a trust-ender.
- *Failure mode to avoid:* a scoreboard. This is a glance, not a result screen.

### A4. Block boundary — the only place it speaks in sentences  · *rough pass (Phase 2a)*

*The moment:* a natural rest. The gate just opened, or the block just ended.

Three things and no more: why this drill (citing the campaign it serves), the
trend, and exactly one thing to listen for next. Primary action continues;
**"bank it and stop here" is an equal-dignity secondary** — ending early is
celebrated, never guarded by a guilt dialogue.

- The voice appears roughly **once per session**, not at every boundary. Design
  the silent boundary too: what does a boundary look like with nothing to say?
- *Failure mode to avoid:* a wall of text. One thought, not a paragraph.

## Session B — declaration and guard rails

Run after A has settled the voice; these inherit it.

### B1. Declaring a campaign — "what do you want to work on?"

*The moment:* the user has just come out of a lesson with four things to work on.

They type, paste or dictate the list. Show what each target matched to: an
existing skill, something already in flight, something new, or **"your call"** for
targets the app cannot measure ("make the last A sing"). Primary action: *show me
the route*.

- Unmatched targets must look **deliberate and fine** — never like errors or
  warnings. They are practised and self-confirmed, never scored.
- *Failure mode to avoid:* a form. This is a list being handed over, not data entry.

### B2. The route and the gap — "here to there"

*The moment:* they've asked for something their hands aren't ready for.

What must come first and why, in one sentence ("you can't aim at the 3rds and 7ths
until you know where they live"). A short list of what's missing. An honest
horizon in the user's own pace — "roughly 9 sessions" — explicitly not an average
of other people.

- *Failure mode to avoid:* reading as a rejection or a report card. It is a route.

### B3. The circling check — permission to stop

*The moment:* the gate opened nine reps ago and they are still going. Nothing has
failed.

An observation plus permission: *"You've run this fourteen times and passed on the
fourth. It's in."* Primary action banks it and moves on. **"I want to keep going"
must be a real, undiminished option.**

- No tally of wasted minutes. No scolding tone. Fixation usually arrives with
  shame already attached.
- *Failure mode to avoid:* an alert. Design it as the calm sibling of the stuck
  screen, not an interruption.

## Do not design here

Navigation and tabs, settings, onboarding, placement (Phase 4), the Track pillar,
or anything requiring a mastery number on screen. The word "mastery" does not
appear in the UI.

## Deliver

Mobile screens first, then the iPad variant of A2 and A3 (the two being built) plus B2. A1, A4 and Session B do not need iPad passes yet. For each screen note
the **states** covered (first run, empty, uncertain, failure) and **any component
or token you had to extend, and why** — those go back into `Theme.swift` and the
design system together, per `design/design-process.md`.
