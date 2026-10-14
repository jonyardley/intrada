# The getting-cold signal

> Tier 3 by domain sensitivity. Issue [#1416]. `intrada-core` only
> (`staleness.rs`).

[#1416]: https://github.com/jonyardley/intrada/issues/1416

Time since an item was practised is measured against an interval its own
history earns, not a flat 14 days. The Up next headline and ranking and
`compute_neglected_items` all read `staleness::assess`.

## Key decisions

1. **An expected return interval, not a decay curve.** One musician will never
   produce the graded reviews a scheduler needs, so the heuristic is fixed,
   small and inspectable: base days from the mark, plus a bonus for returns.

   | Latest mark | Base days |     | Returns | Bonus days |
   |---|---|---|---|---|
   | none | 10 |  | 0 to 1 | 0 |
   | 0 to 3 | 5 |  | 2 to 4 | +2 |
   | 4 to 6 | 9 |  | 5 to 9 | +5 |
   | 7 to 8 | 16 |  | 10 or more | +9 |
   | 9 to 10 | 30 |  | | |

   Range 5 to 39 days. `expected_interval_days` rides on the struct so the
   heuristic can be argued with (#1419).

2. **Three bands and an Unknown, and `Cold` only at double the interval.**
   `Fresh` inside the interval, `GoingCold` up to twice it, `Cold` beyond,
   `Unknown` for never practised or an unreadable date. Bias lax.

3. **Never practised says so.** `Unknown` is never `Cold`, but still ranks
   stalest of all.

4. **The ranking moves with the copy.** The Up next anchor and its exercises
   rank on `overdue_key` (days as a fraction of the interval), so the headline
   says what the ranking used (decision 3 of [`up-next-card.md`](up-next-card.md)).

5. **The interval reads the mark the surface shows**: a step mid-climb, or an
   exercise's mark in one piece's context.

6. **The gap is said in the unit a musician would use.** Days to a fortnight,
   then weeks, then months, then "over a year".

7. **Headline only; the card's item rows keep their one clause.** Decision 6 of
   the card spec holds rows to a single clause, and the coldness that decides a
   suggested *block* is the anchor piece's, which the headline already carries.
   Per-exercise coldness has room on the cold shelf, where it is the whole
   point rather than a second clause squeezed under a mark (#1418).

8. **`compute_neglected_items` reads the same estimate**, filtering on
   `is_fresh()`. One opinion in the core about how cold something is.

9. **Nothing gates.** The output is a fact the user reads; the app suggests,
   never prescribes.

Copy: a mid-sentence clause of five words at most, such as `going cold after
3 weeks` or `cold for 3 months`.

## Follow-ups

- A coldness clause on the card's item rows, per decision 7 (#1418).
- Whether the thresholds are right, after real use (#1419).
