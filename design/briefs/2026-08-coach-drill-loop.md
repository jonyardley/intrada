# Design brief — the practice-coach drill loop

> Handover to Claude Design, updated 2026-08-04 for spec v7 (#1170; supersedes
> the 2026-08-03 v5 handover, #1169's orientation refresh). Work against
> `design/intrada-design-system.dc.html` (tokens canonical in
> `ios/Intrada/DesignSystem/Theme.swift`). Mockups land under
> `specs/<feature>/design/<screen>.dc.html` per `docs/design-workflow.md`.
> Design context: `specs/intrada-practice-coach-design.md` (v7) and the eleven
> scenarios in `docs/coach-user-journeys.md`. Cross-check screen states against
> the wireframes in `docs/coach-orientation.html` (rail 1, "the loop") — if this
> brief and those wireframes disagree, the wireframes are the more recent
> snapshot; flag the conflict rather than silently picking one.
>
> **Note on `docs/journeys.md`:** earlier briefs cite its numbered steps. It is
> retired — the notebook-era journey. Cite `docs/coach-user-journeys.md` instead.
>
> **What changed for v7 (decisions 18–19, both 4 Aug 2026):**
> - **Decision 18 — machine listening deferred.** There is no microphone, no
>   MIDI, no per-note fact, and no "the app isn't sure" state. A2 and A3 below
>   are rewritten around the **tap-verdict**: the user hears their own attempt
>   and taps pass or fail against a countable criterion ("Clean at 80?"). This
>   is not a placeholder for a future machine verdict — it is the shipped
>   mechanism, and it must read as a real, permanent way of working, not a
>   degraded one.
> - **Decision 19 — user-created items and the built session.** Two new
>   surfaces join Session B: creating a user item when a teacher's exercise
>   matches nothing in the graph, and building today's session by hand from
>   pipelines, nodes and the user's own items. See B5 and B6.
>
> **Ten screens across two sessions**, and that is not the whole app — see "What
> this brief deliberately does not cover" before assuming a surface is missing by
> accident.
>
> **Run this as two sessions.** Session A is the drill loop: four screens seen
> every single day, and the ones that must be right. Session B (now six
> screens: B1–B4 plus decision 19's B5–B6) depends on A's decisions about how
> the app speaks. Ten screens in one pass produces ten mediocre screens.
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

- **No microphone yet, and say so once, plainly** (decision 18): a quiet,
  dashed-border note — *"No microphone yet — play it, then tell it how it
  went"* — sits between the orientation strip and the stuck button, per
  `docs/coach-orientation.html` wireframe 2. It is context, shown once per
  block boundary or first-run, not a persistent apology — don't let it read as
  a bug report or a missing-feature nag.
- The orientation must read as information, never as a countdown or a verdict.
- *Failure mode to avoid:* anything that rewards looking at the screen. If it
  pulls the eyes, it ruins the next phrase.

### A3. After a repetition — one glance, one tap  · **full treatment (Phase 1)**

*The moment:* the attempt just ended. The gate criterion is countable and
already on screen ("clean at 80, both hands"); the user's own ears are the
instrument, and their tap is the verdict — not a fact the app detected.

**The tap-verdict pattern (decision 18), per `docs/coach-orientation.html`
wireframe 3:**
- The gate criterion as the question, restated plainly: **"Clean at 80?"**
  — never "temporal deviation metric", always the musician's own words for
  the thing they just tried to do.
- Two big targets, asymmetric weight: **"Yes — clean"** as the wide primary,
  **"No — missed it"** as a calm ghost secondary directly beneath. Both must
  be equally easy to hit without looking down for long — this is still a
  glance, not a decision screen.
- Gate progress as filled/empty dots plus a count ("2 of 3"), updating on tap.
- **"I'm stuck"** stays present, same place as A2 — the tap-verdict doesn't
  replace the stuck path.
- The next count-in follows the tap automatically. No per-note facts, no
  "2 wrong notes", no "rushing bars 3–4" — there is nothing listening to
  produce those, and inventing plausible-sounding detail the app doesn't
  actually know is worse than the plain binary ask.
- **Design the "No — missed it" state with equal care** — factual, calm,
  never shaming; it is the user's own honest report, not a failure the app
  caught them in.
- *Drop, don't design:* the "uncertain / app isn't sure — agree?" variant from
  the earlier v5 pass. There is no machine confidence to be uncertain about
  (decision 18) — the tap *is* the ground truth, full stop.
- *Failure mode to avoid:* a scoreboard, or the tap-verdict reading as a
  downgrade from some imagined "real" scored version. This is the shipped
  mechanism, not a stopgap.

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

### B3. The stuck ladder — help that acts

*The moment:* third failed rep on the same thing. They are close to giving up.

The app has already acted rather than asked: tempo dropped, or scope shrunk to one
hand / one change / one key. Show the new plan and one cue, framed as the plan —
*"Let's take it to 100"* — never as remediation. Primary action: count me in.

- **Design this and B4 as siblings.** They are mirrors — one fires on failure, the
  other on success-you-can't-leave — and they must feel like the same voice being
  kind in two directions. Designing them apart is how one ends up reading as an
  error and the other as a nag.
- If this is a quit-point, the wall is named **once**: normalisation plus a smaller
  step ("everyone's enclosures sound mechanical for three weeks; here's the smaller
  version"). Never the same encouragement twice.
- *Failure mode to avoid:* anything that reads as a failure report. The user knows
  they failed; the screen's job is to make the next attempt look winnable.

### B4. The circling check — permission to stop

*The moment:* the gate opened nine reps ago and they are still going. Nothing has
failed.

An observation plus permission: *"You've run this fourteen times and passed on the
fourth. It's in."* Primary action banks it and moves on. **"I want to keep going"
must be a real, undiminished option.**

- No tally of wasted minutes. No scolding tone. Fixation usually arrives with
  shame already attached.
- *Failure mode to avoid:* an alert. Design it as the calm sibling of the stuck
  screen, not an interruption.

### B5. Creating a user item — "your teacher gave you something new"

*The moment:* a lesson handed over an exercise that matches nothing in the
graph (decision 19, journey 11). The user types, pastes or dictates it.

- **Three-way resolution, shown as one clear question, never three form
  fields.** The app proposes a match first — *"this looks like a shells
  exercise — track it under shells?"* — and only falls back to a **user
  node** when nothing plausibly fits. Confirm / decline is one tap either way;
  nothing is silently filed.
- **A declined or no-match item still gets a real home**: gate criteria in the
  same schema as an authored gate ("clean at 80, both hands, 4 keys"), written
  at creation via one short form or one dictated sentence, LLM-parsed and
  **shown back for confirmation** before it saves (decision 12 — interpretation,
  never silent).
- An optional ***serves* tag** (which circle it feeds) — explicitly optional,
  never a required field; an untagged user node is a fine, complete state.
- **This must read as ordinary, not as a fallback tier.** A user node is
  "your own material, now tracked" — due, spaced, cold-tested like anything
  authored — not a second-class shelf for things the graph couldn't place.
- *Failure mode to avoid:* a data-entry form. This is a teacher's exercise
  being handed over, same register as B1's target list, not a settings screen.

### B6. Building today's session — composing from your own material

*The moment:* the user wants to assemble today's blocks themselves — some
pipeline work, one of their own items, one thing from the plan (decision 19,
journey 11) — from the steer surface.

- **Composition, not a mode.** Blocks are chosen from pipelines, nodes and the
  user's own items in one list; each keeps its ordinary gate, and evidence
  lands exactly as if the app had prescribed it. Entering this screen must not
  feel like leaving the app's tracking behind.
- **The session-shape template is advice, shown once, declinable in one tap**
  — a warm-up first, music at the end — and declining it is not remarked on
  anywhere later. Never a nag to accept the recommended order.
- **Tomorrow resumes unoffended**: no reconciliation step, no "catching up"
  language. The built session's evidence is just folded into the plan.
- *Failure mode to avoid:* anything that reads as "going off-script" or asks
  the user to justify the built session. It is a legitimate, equal-dignity way
  to spend the day, not an escape hatch that needs explaining.

## What this brief deliberately does not cover

Eight surfaces exist in the design and are **out of scope here**, so nobody
mistakes this for the whole app. Each waits for the phase that builds it, because
a mockup made months early is a mockup that gets redrawn:

| Surface | Journey | Phase |
|---|---|---|
| Session-end narrative (trends, the thread, tomorrow's draft) | 2, 10 | 3 — the prose needs the voice |
| The bad day / shorter-session offer | 6 | 2b |
| Coming back after a gap | 7 | 2b |
| Off-piano queue | 8 | 2b |
| Off-piste and unmonitored play | 9 | 2b |
| Something lands — milestone and pipeline advance | 10 | 2b |
| The phrase's per-key pipeline view | — | 2b |
| Time-by-circle (Track) | — | 2b |

**Never design here:** navigation and tabs, settings, onboarding, placement
(Phase 4), or anything putting a mastery number on screen. The word "mastery"
does not appear in the UI at all.

## The durable output is components, not screens

Per `design/design-process.md`, what survives is the component catalogue. These
screens need primitives that don't exist yet — a gate-progress dot row, the
tap-verdict pair (wide primary / ghost secondary), the ambient orientation
strip, target-match rows, a match-proposal row (B5's "track it under shells?"),
a session-builder block list (B6), a route list. **Those go into
`design/intrada-design-system.dc.html` and `Theme.swift` together**, and they
are what makes the eight deferred surfaces above cheap to design later. Ten
screens is the visible deliverable; the primitives are the valuable one.

## Deliver

Mobile screens first, then the iPad variant of A2 and A3 (the two being
built). A1, A4 and Session B do not need iPad passes yet. For each screen note
the **states** covered — for A3 that's first run, empty (no gate progress
yet), pass tap, fail tap; there is no "uncertain" state to design (decision 18
removed it) — and **any component or token you had to extend, and why**; those
go back into `Theme.swift` and the design system together, per
`design/design-process.md`.
