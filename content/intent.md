# Intent — the goal, and the campaign in flight

The declared destinations the fortnight is practised against, per
[`specs/intrada-practice-coach-design.md`](../specs/intrada-practice-coach-design.md)
§"Intent: goals, campaigns and steering". A destination is a *target*, never a
route: the route comes from back-chaining through the graph's prerequisites and
the pipelines' stage order.

This file is the paper version of what Phase 2 automates. Filling it in and
practising from it is the test of whether the mechanism works at all.

## The goal

**Improvise confidently over jazz standards** — hearing a line and being able
to play it, in any key, over the changes of a tune I know.

Horizon: months. Revised when it stops being true, not on a schedule.

## The campaign in flight

**Destination: restricted improv over Strasbourg / St. Denis** — landing the
3rd or 7th on beat 1 of every change, through the full form, with the ♭9 lick
available as vocabulary.

Declared: 2026-08-03 · Horizon: ~2 weeks (a guess; the gate-attempt log will
correct it)

### Targets in this campaign

A campaign holds a *set*. This is the shape a lesson's worth of targets takes —
some map onto graph nodes, some don't, and the ones that don't are kept as-is
rather than forced (spec decision 13).

| Target (as stated) | Resolves to | Status |
|---|---|---|
| Land 3rds and 7ths on the changes | `chord-tone-targeting` | matched — node |
| Get the rootless voicings under the melody | `rootless-a-b` → `rootless-under-melody` | matched — node + gate |
| The ♭9 lick in more keys | `phrase-transposition` / [`p001`](phrases/p001-flat9-turnaround.md) | matched — pipeline stage |
| Know the bridge changes from memory | `tune-form-memory` (stub) | matched — stub, needs authoring |
| Make the last A section sing | — | **opaque** — self-confirmed, not scored |

The opaque one is deliberate and worth watching: if targets like it recur
across campaigns, that is the queue telling me which node to author next.

### The route, back-chained by hand

Derived from the target set, not authored as a plan. Prerequisite order does
the sequencing:

1. `shells-ii-v-i` must be solid first — chord-tone targeting can't be aimed
   until the 3rds and 7ths are mapped. Currently 0.7 / medium, so this is
   maintenance rather than learning.
2. `rootless-a-b` continues on the traversal schedule (2–3 new keys a session).
   It is the frontier, and it is the grind block.
3. `phrase-transposition` advances p001 two keys per session, in parallel —
   a few minutes, not a block's worth.
4. `chord-tone-targeting` starts at C1 (the guide-tone skeleton) as soon as
   shells hold, then C2, then C3 over the full form. C3 is the destination.
5. "Make the last A sing" rides along as a self-rated close, never gated.

### The structural gap read

What the graph says is missing between here and the destination — reliable on
day one because it is pure prerequisite arithmetic, no history needed:

- **`chord-tone-targeting` has never been attempted.** All three of its gates
  are unopened; C1 hasn't been run once.
- **`tune-form-memory` is a stub**, so "the bridge from memory" has no gate to
  pass. Either author the node or keep the target opaque for now.
- **`micro-transcription` is untouched this campaign** — the only head-circle
  node in the set, and the fluency frame predicts exactly this starvation.
  Worth one block a week even though no target names it.

The statistical read (decayed confidence, flat velocity) needs weeks of logs
and is deliberately not attempted here — see spec decision 4.

## Today's steer

The lowest altitude: one line, only when today differs from the plan. Log it
here so the wander log and the why log stay comparable.

| Date | Steer | What the plan became |
|---|---|---|
| *2026-08-03* | *tired — hands only, no new keys* | *maintenance passes + p001 in C, no frontier block* |
