# Practice coach — user journeys

Ten scenarios the app must handle well, derived from
[`specs/intrada-practice-coach-design.md`](../specs/intrada-practice-coach-design.md)
(v4). Written to inform design work and to be argued with: if a journey needs a
mechanism the spec doesn't have, that's a spec gap, not a design problem to
solve in the UI.

Each carries a **without this** line — the failure mode it exists to prevent —
because that is what makes it testable. Journeys 1, 5, 6 and 7 are the
retention-critical ones; a coach that only handles journey 2 is a scorer.

> Convention: "the app is silent" means Layer 0 of the feedback choreography —
> no verdicts while playing. Every journey obeys the one-interruption budget
> (see the open question at the foot of this doc).

---

## 1. The very first session

**Trigger.** App opened for the first time. No history, no attempts
distribution, no velocity, no trend.

**What the user needs.** To play something within a minute, and to not be
interviewed.

**What the app does.**
1. Asks for the one thing it cannot infer: how long today.
2. Runs the authored content's own starting values — the seeded mastery
   estimates, the declared goal and campaign from `content/intent.md`. No
   placement test (Phase 4, deliberately).
3. States its own ignorance in one line: *early days, still learning your
   level* — and means it, because for the first fortnight every adaptive
   mechanism is running on priors.
4. Scores attempts as normal, but withholds anything that requires a baseline:
   no "above your norm", no horizon in sessions, no circling check.

**Mechanisms.** Seeded mastery as (estimate, confidence) with confidence *low*;
authored gate criteria; the never-bluff principle.

**Without this.** The app either interrogates the user before they've played a
note, or fabricates comparisons it has no data for — and the first wrong
confident statement is the one that costs trust permanently.

---

## 2. The ordinary day

**Trigger.** The habitual slot. Twenty minutes.

**What the user needs.** To not decide anything, and to finish feeling capable.

**What the app does.**
1. Opens on the plan and its one-line why, citing the declared campaign.
2. Warm-up on something owned — 90%+ success, scored silently.
3. Frontier block, gated. Silent during play; a tick or cross and one fact per
   rep; gate progress visible; auto count-in between reps.
4. Phrase advance, two keys, a few minutes.
5. Integration: restricted improv, softer gate, feel self-rated.
6. Close on the head. Summary one line at the piano; the full narrative waits.

**Mechanisms.** Planner as a pure function; the four-layer choreography;
gates-not-minutes with a visible ceiling; music at both ends.

**Without this.** The session becomes admin — choosing, configuring, logging —
and the thing that was supposed to remove friction adds it.

---

## 3. Lesson day — a list of targets arrives

**Trigger.** A teacher has said: rootless voicings under the melody, the
Rollins phrase in more keys, bridge from memory, make the last A sing.

**What the user needs.** To hand over the list and get a fortnight's plan,
without typing a curriculum.

**What the app does.**
1. Accepts the list as written — typed, pasted or dictated.
2. Matches each target: two to skill nodes, one to a pipeline stage, one to
   nothing.
3. Keeps the unmatched one as an **opaque target** — prescribable, self-confirmed,
   never scored, never crammed into an ill-fitting node.
4. Back-chains: shells first, because the voicings sit behind them.
5. Shows the structural gap read and an honest horizon from the user's own pace.
6. Sequences against the in-flight caps, saying what's queued rather than
   dropping it.

**Mechanisms.** Multi-target campaigns; opaque targets; back-chaining;
structural gap read; caps outranking intent.

**Without this.** Either the user hand-builds sessions (the thing being
deleted), or the teacher's actual instructions live outside the app and it
guides them somewhere else.

---

## 4. Hitting a wall

**Trigger.** Third consecutive failed rep on the same thing.

**What the user needs.** The next attempt to be winnable, without a lecture.

**What the app does.**
1. Cross and one actionable fact — "late into bar 3", never bare "wrong".
2. On the third, acts rather than narrates: drops tempo 20%, or shrinks scope to
   one hand / one change / one key, or changes mode (sing it, tap it).
3. Frames it as the plan: *let's take it to 100* — not as remediation.
4. Names the wall once if this is a quit-point, with normalisation and a smaller
   step. Never repeats that encouragement in a later session.

**Mechanisms.** Escalation ladder from the method pack; anti-nag rules;
name-the-wall-once.

**Without this.** A wall becomes an exit. This is the single most common way
self-directed practice ends.

---

## 5. Circling — succeeding and unable to stop

**Trigger.** The gate opened nine reps ago and the user is still going. Or
attempts-to-pass is far above their own norm. Or one node has eaten the
fortnight.

**What the user needs.** Permission to stop, without being told off.

**What the app does.**
1. Says nothing during play.
2. At the next boundary, once: *you've run this fourteen times and passed on the
   fourth. It's in.*
3. Offers the exit as the primary action; **keep going** stays a real,
   undiminished option.
4. Does not mention it again this session. Does not count wasted minutes.

**Mechanisms.** The circling check; permission-not-correction framing; the
interruption budget.

**Without this.** The app cheerfully watches a month disappear into one
voicing — and because nothing failed, every other safeguard stays quiet.

---

## 6. The bad day

**Trigger.** Tired, low motivation, ten minutes. Possibly two days missed.

**What the user needs.** The app to be *lighter* than usual, not heavier.

**What the app does.**
1. Offers the shorter session by default, not as a downgrade.
2. Drops the frontier and the grind. Front-loads a win.
3. Keeps real music at the end regardless of length.
4. Offers the grind trade explicitly: *skip it today, two blocks tomorrow* —
   logged as a deal.
5. Ends early on one tap, banked, no confirmation dialogue and no guilt.

**Mechanisms.** Design-for-the-bad-day; grind trade; soft-landing exit; streaks
that count any engagement.

**Without this.** The bad day becomes the last day. Abandonment compounds; a
shortened session recorded does not.

---

## 7. Coming back after a gap

**Trigger.** Five days off. Confidence has decayed in the model; probably in the
hands too.

**What the user needs.** No scolding, and an honest read of what slipped.

**What the app does.**
1. Says nothing about the gap beyond acknowledging it factually, once.
2. Re-derives the plan from current state: decayed confidence pulls maintenance
   passes forward ahead of new keys.
3. Re-scopes the campaign if its horizon has passed, rather than reporting it as
   failed.
4. Front-loads something that will still feel good.

**Mechanisms.** Confidence decay driving spacing; weekly re-derivation; campaign
review; progress framing over loss framing.

**Without this.** The return is met with a broken streak and a stale plan, which
is the moment most people quit for good.

---

## 8. No instrument

**Trigger.** Commute, lunch break, or a day the lid never opens.

**What the user needs.** Useful practice that doesn't need a piano, offered
where they actually are.

**What the app does.**
1. Serves the off-piano queue — its own queue with its own cue, not blocks
   inside a piano session.
2. Commute-sized items: a targeted listen, an analysis read, the audiation rung
   (work the phrase out in your head, first attempt at the keys is the test).
3. Counts toward the streak and the head-circle tally.

**Mechanisms.** `away` mode tags; the off-piano queue; listening as a
prescribable item; the two-circle tally.

**Without this.** Head-circle work competes for piano minutes and always loses,
which is exactly the starvation the fluency frame predicts.

---

## 9. Just playing

**Trigger.** The user wants to play, not practise.

**What the app does.** Two distinct offers, and the difference is consent:
- **Off-piste** — no plan, still listening and logging. Ends with *keep this as
  a drill?* because some wanders reveal a gap in the graph.
- **Unmonitored** — time logged, nothing scored, nothing inferred, no prompt.

**Mechanisms.** Off-piste mode; unmonitored play (decision 16); the wander log.

**Without this.** Every session is an assessment, and the practice room stops
being one. Some days the honest answer is "not today".

---

## 10. Something lands

**Trigger.** The phrase reaches solid in all twelve keys. Or the campaign's
destination gate passes.

**What the user needs.** To feel it, and to see the thread back to the work.

**What the app does.**
1. Marks it plainly — no confetti, no badge shower.
2. Draws the thread: *Monday's voicing drill is what your left hand just did in
   bar 5.*
3. Advances the pipeline and puts the next thing in flight, so the loop stays
   open (the itch that brings you back).
4. Retires the campaign and asks what's next, rather than silently rolling on.

**Mechanisms.** Peak-end rule; the narrated thread; pipeline advance; caps
freeing a slot; Zeigarnik.

**Without this.** Progress happens and is never felt — which is the failure the
whole motivation model exists to prevent.

---

## Open questions these journeys expose

1. **Interruption budget.** Journeys 4, 5, 6, 7 and the gap read each own a
   "once per session" intervention. Five politely-limited features still add up
   to five interruptions. There must be one budget and a priority order across
   all of them, arbitrated in one place — not per-feature limits.
2. **Cold start for user zero** (journey 1). Every adaptive mechanism needs a
   baseline the first fortnight doesn't have. What is suppressed, and when does
   each switch on?
3. **Self-confirmed vs measured in one mastery model.** Journeys 3 and 9 produce
   self-report; journey 2 produces measurement. Storage is separate by design —
   whether they feed one mastery estimate is unspecified.
4. **Campaign completion.** Journey 10 assumes a destination gate. "Improvise
   confidently over Strasbourg" needs a definition of done or it runs forever.
