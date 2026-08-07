# Tune pipeline — Strasbourg / St. Denis

The test fixture (Roy Hargrove). The tune is the vehicle, the skill is the
cargo: drills in [`nodes.md`](nodes.md) are parameterised by this tune
("rootless under the melody, Strasbourg A section"), and the in-flight
phrase integrates over it at its stage 7.

Each stage carries up to two gates (decision 20, acquisition before the clock):
**know it** (l0: no click, no tempo, in chunks from memory) and **own it in
time** (the clocked gate, entered below target tempo and ramped up on clean
passes). Chunks follow the tune's structure, not a bar count: enter at the
section, split at the nearest phrase mark on a fumble, merge on clean passes
with each join its own rung, then the full form. The size is never asked or
authored; only the boundaries are (sections here, phrase marks per stage,
TODO(jon) with the seeds). The
know-it gate is a default, never a wall: Skip jumps it, and passing the
clocked gate retires it. The engine can't run l0 yet (#1244, #1245), so until
then the know-it column is practised by hand, Phase 0 style.

Seeds marked `TODO(jon)` — fill from where the tune actually is (README,
day-one checklist).

| Stage | Know it (l0, from memory) | Own it in time | State |
|---|---|---|---|
| 1. Form | form + changes named from memory, away from the keys | (away mode: no clocked gate) | TODO(jon) |
| 2. Melody | head from memory, section-wise (split where it fumbles), then the joins, then the full head, out of time | head from memory, click L2, 2 clean passes, ramped to target | TODO(jon) |
| 3. Shells | shells found under the melody, chunk-wise, out of time | melody over shells, full form, clocked — `shells-under-melody` extended past the A section | TODO(jon) |
| 4. Rootless | rootless LH found chunk-wise, out of time | melody over rootless LH, full form — `rootless-under-melody` extended | — |
| 5. Arpeggiate changes | (usually skipped: the shapes are known by here) | 1-3-5-7 up each chord in time through the form | — |
| 6. Guide tones | line worked out chunk-wise, out of time | continuous guide-tone line through the form (`guide-tone-lines` stub graduates here) | — |

Cap check: one phrase (p001) + one tune in flight. Within budget (the
anti-overwhelm cap is one phrase, one or two tunes).
