# Phase 0 content — the paper teacher

The minimal content set from
[`specs/intrada-practice-coach-design.md`](../specs/intrada-practice-coach-design.md)
(Build plan, Phase 0): 5 skill-graph nodes, 2 method packs, 1 phrase with
per-key state, gate criteria as data, and the tune-pipeline position for the
test fixture. No code reads these files yet. For a fortnight, **you** are the
app: run your practice from this content and keep the four logs. The logs are
the deliverable; the content is the instrument.

## Files

| File | What it is |
|---|---|
| [`nodes.md`](nodes.md) | The 5 fully-specified skill-graph nodes + stubs for the rest of the improvisation branch |
| [`method-packs/voicing-traversal.md`](method-packs/voicing-traversal.md) | How to attack shell + rootless voicing work |
| [`method-packs/phrase-transposition.md`](method-packs/phrase-transposition.md) | How to take a phrase through the keys |
| [`phrases/p001-flat9-turnaround.md`](phrases/p001-flat9-turnaround.md) | The one in-flight phrase: notation, devices, pipeline stage, per-key state |
| [`gates.toml`](gates.toml) | Every gate criterion as tunable data — day-one guesses, calibrated from the gate-attempt log |
| [`tunes.md`](tunes.md) | Strasbourg / St. Denis tune-pipeline position |
| [`logs.md`](logs.md) | Templates for the four Phase 0 logs (divergence, gate-attempt, why, wander) |

## Circle tags (the fluency frame)

Every node carries two tags from the spec's fluency-frame subsection
(drills inherit their node's tags unless marked otherwise):

- **Circle**: `head` (music you can hear/imagine), `hands` (what you can
  execute), or `bridge` (grows the overlap — the point of the whole thing).
- **Mode**: `keys`, `away` (no instrument needed), or `keys→away` (a
  difficulty ladder from the instrument toward pure audiation).

Known imbalance, stated up front: this v1 set is 2× hands, 2× bridge,
1× head — measurable-first content leans hands-circle exactly as the frame
warns. The fortnight's wander and why logs should watch for head-circle
starvation; the long-term fix (time-by-circle Track view, planner bias) is
recorded Phase 4 debt.

## Day-one checklist

1. Correct the **seed mastery values** in `nodes.md` and the per-key table in
   `phrases/p001-flat9-turnaround.md` — they are plausible guesses, not
   measurements. Ten minutes at the piano settles each one.
2. If the phrase you're actually working on isn't the ♭9 turnaround, replace
   the notation in `p001` and keep the file's structure — the structure is
   the schema.
3. Fill the `TODO(jon)` rows in `tunes.md` from where Strasbourg actually is.
4. Print or open `logs.md` where you practise. A log you can't reach doesn't
   get written.

## The fortnight's writing tasks (besides the logs)

From the spec's Phase 0 list, to be written during/after the fortnight, not
on day one: the failure stories and desired app responses; what the app
should have done on the inevitable bad day; the placement session as a
thought experiment, checking the criteria aren't jon-shaped.
