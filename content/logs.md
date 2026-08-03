# The four Phase 0 logs

One table each, one line per entry, filled at the piano or immediately
after. These logs are the actual Phase 0 deliverable: they spec the
self-assessment boundary, the first tolerance numbers, the graph's
completeness, and the size of off-piste mode. Example rows are italicised —
delete them once real rows exist.

## 1. Divergence log — machine-score vs felt-score

One line whenever what the criteria say and what your ears say disagree.

| Date | Drill | Criteria verdict | Felt verdict | The one-line difference |
|---|---|---|---|---|
| *2026-08-03* | *phrase-nearby-keys (D)* | *clean* | *stiff* | *notes right, swing died at the resolution — criteria can't hear feel* |

## 2. Gate-attempt log — attempts, pass/fail, felt difficulty

Every gate attempt, pass or fail. Felt difficulty 1–5 (3 ≈ the desirable-
difficulty band). This log is what recalibrates `gates.toml`.

**Attempts is the most valuable column here** — how many tries it actually took.
It answers whether "3 clean passes" costs four attempts or forty, and its spread
is what the circling check later compares against to spot a perfectionism loop.
Count reps *after* the gate opened separately, in the last column: that number is
fixation, not effort.

| Date | Gate | Tempo | Attempts | Pass? | Felt (1–5) | Reps after passing | Note |
|---|---|---|---|---|---|---|---|
| *2026-08-03* | *rootless-transition (F)* | *60* | *11* | *no* | *5* | *—* | *third fail — ladder fired, dropped to 48* |
| *2026-08-03* | *phrase-home-key (C)* | *120* | *5* | *yes* | *2* | *9* | *kept going after it passed — worth noticing* |

## 3. Why log — every block's one-line why

Written before the block, citing node state. A why that can't be written is
a missing node — flag it.

| Date | Block | The why | Destination it serves |
|---|---|---|---|
| *2026-08-03* | *rootless R2 in F* | *frontier node at 0.3/low; F is next in whole-step order; transition rung is where it broke yesterday* | *restricted improv over Strasbourg* |

A why that can't be written is a missing node. A why whose destination column
is blank is a missing campaign — see [`intent.md`](intent.md).

## 4. Wander log — off-plan time

Trigger, duration, and the keep-as-drill answer. Some wanders are the graph
revealing a gap.

| Date | Minutes | Trigger | Keep as drill? |
|---|---|---|---|
| *2026-08-03* | *8* | *avoided the grind block, played the head instead* | *no — comfort loop, already templated* |
