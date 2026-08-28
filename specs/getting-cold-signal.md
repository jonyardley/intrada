# The getting-cold signal

> Tier 3 spec by domain sensitivity, though the surface is smaller than that
> suggests. Issue [#1416], step 8 of the adopted order in
> [`docs/research/comparison.md`](../docs/research/comparison.md) and Slice 1 of
> the [`space-layer`](../docs/research/space-layer.md) research note. One PR:
> `intrada-core` only. No ViewModel field, no bindings change, no Swift logic,
> no API, no migration, no new storage.

[#1416]: https://github.com/jonyardley/intrada/issues/1416

## Problem

The app has one staleness signal and it is binary. `compute_neglected_items`
flags anything untouched for 14 days, and the Up next card (#1082) counts raw
days in its headline reason and ranks on that same raw count.

A flat clock is wrong about most of the library. A tune marked 3 of 10 and
played once is fragile; the same tune marked 9 of 10 across twenty returns is
consolidated, and continuous motor skills hold for weeks (Adams 1987, via the
research note §4). Fourteen days is far too late for the first and far too
early for the second, and the card says "not practised for 6 days" as though
that number meant the same thing everywhere.

## Approach

One pure module, `staleness.rs`, and three readers.

```text
ItemPracticeSummary  (last practised, latest mark, how many returns)
     │  staleness::assess(practice, mark_the_surface_shows, clock)
     ▼
Staleness { band, days, expected_interval_days }
     │
     ├── .clause()      → the Up next card's headline reason
     ├── .overdue_key() → the Up next ranking, anchor and exercises
     └── .is_fresh()    → compute_neglected_items
```

The estimate is a **gap measured against an interval**, not a gap. The interval
is what the item's own history earns it; the band is where the gap falls
relative to that interval.

## Key decisions

1. **An expected return interval, not a decay curve.** No published scheduler
   is validated for repertoire, and one musician will never produce the
   thousands of graded reviews FSRS needs to fit one (research note §4, §6). So
   the heuristic is fixed, small, and inspectable: base days from the mark,
   plus a bonus for returns.

   | Latest mark | Base days |     | Returns | Bonus days |
   |---|---|---|---|---|
   | none | 10 |  | 0 to 1 | 0 |
   | 0 to 3 | 5 |  | 2 to 4 | +2 |
   | 4 to 6 | 9 |  | 5 to 9 | +5 |
   | 7 to 8 | 16 |  | 10 or more | +9 |
   | 9 to 10 | 30 |  | | |

   Range 5 to 39 days, with the retired 14-day flag inside it rather than at
   either end, so this is neither a wholesale loosening nor a tightening. An
   unmarked item gets 10, between fragile and consolidated: no mark is no
   evidence either way, and the signal must not invent a curve for an item it
   knows nothing about.

   `expected_interval_days` rides on the struct so the heuristic can be argued
   with. Nothing surfaces it to the user yet, so it is crate-internal today;
   the point is that when something does, the number is already there rather
   than buried in a private calculation (research note §6).

   Caveat on `session_count`: `build_practice_summaries` increments it per
   setlist *entry*, so an item drilled as two entries in one session counts as
   two returns. That works mildly against the research's "weight scores from
   returns over scores from drilling" (§6). Pre-existing field semantics, not
   worth changing under this issue, but it means the bonus is slightly generous
   for heavily-drilled items.

2. **Three bands and an Unknown, and `Cold` only at double the interval.**
   `Fresh` inside the interval, `GoingCold` up to twice it, `Cold` beyond,
   `Unknown` for never practised or a date that cannot be read. The doubling is
   the lax bias: over-eager resurfacing nags and gets ignored, which costs more
   than today's silent rot. Bias lax, and let a later slice tighten.

3. **Never practised says so.** `Unknown` is its own band, never `Cold`. An item
   with no history has no interval it could be past, and "cold for 137 days" on
   a piece nobody has ever opened is a lie. It still ranks stalest of all, and
   still reaches the neglected list, because an item the library never got to
   is its own kind of left behind, so `is_fresh()` is the predicate rather than
   an `is_overdue()` that would have to special-case it.

4. **The ranking moves with the copy, not just the copy.** The Up next anchor
   and its exercises rank on `overdue_key` (days as a fraction of the interval,
   in hundredths), not raw days. Decision 3 of
   [`up-next-card.md`](up-next-card.md) requires the headline reason to say the
   same thing the ranking used; grading one and not the other would break that
   silently. Never practised stays `u32::MAX`, the stalest of all, unchanged.

   Consequence, and the point: a fragile piece twelve days cold now outranks a
   consolidated one sixteen days cold. Two tests pin exactly that, each built so
   raw days *and* the title tie-break would pick the other item.

5. **The interval reads the mark the surface shows.** A step mid-climb, or an
   exercise's mark in one piece's context, not the flat `latest_score`: the
   same rule decision 4 of the card spec already applies to ranking. Per-piece
   is the grain #1081 established, and a drill solid under one tune can be rough
   under another.

6. **The gap is said in the unit a musician would use.** Days to a fortnight,
   then weeks, then months, then "over a year". Grading the units matters as
   much as grading the band: "not practised for 137 days" is a number nobody
   reads; "cold for 5 months" is a sentence.

7. **Headline only; the card's item rows keep their one clause.** Decision 6 of
   the card spec holds rows to a single clause, and the coldness that decides a
   suggested *block* is the anchor piece's, which the headline already carries.
   Per-exercise coldness has room on the cold shelf (step 9 of the adopted
   order), where it is the whole point rather than a second clause squeezed
   under a mark. Tracked as a follow-up rather than assumed.

8. **`compute_neglected_items` reads the same estimate.** It took its own fold
   of the sessions and applied its own 14-day cut; it now takes
   `&HashMap<String, ItemPracticeSummary>`, the summaries `build_view` already
   derives, and filters on `is_fresh()`, ordered by `overdue_key` then id. One
   opinion in the core about how cold something is, not two.

   Its `NeglectedItem` shape is unchanged, so no ViewModel field and no
   bindings change. It has no Swift reader today (#1349's band); this PR makes
   it honest rather than either growing it or deleting it, because #764
   ('neglected priority') is the open issue that would consume it.

9. **Nothing gates.** The output is a fact the user reads. No new surface, no
   new tap, no new suggestion; the card that existed says a truer sentence and
   ranks better. This is the line the coach pivot crossed and #1344 reversed:
   the app surfaces evidence and suggests, never prescribes.

## Copy

Mid-sentence, so the card can join it to the priority clause; the caller heads
a line with it. Written against
[`docs/tone-of-voice.md`](../docs/tone-of-voice.md): British, sentence case, no
full stop, no second person, five words at most.

| Band | Clause | Card headline |
|---|---|---|
| Unknown | `not practised yet` | `Not practised yet` |
| Fresh, today | `practised today` | `A priority · practised today` |
| Fresh, yesterday | `practised yesterday` | `Practised yesterday` |
| Fresh | `practised 6 days ago` | `Practised 6 days ago` |
| GoingCold | `going cold after 3 weeks` | `A priority · going cold after 3 weeks` |
| Cold | `cold for 3 months` | `Cold for 3 months` |

`cold` is a temperature, not a claim about the user's memory. "72% retained"
would be borrowed authority; "cold for 3 months" is a fact about the calendar.

## Testing

Test-driven, per CLAUDE.md: pure `intrada-core` logic, so the tests came first.

- **The interval** is asserted as properties, not as its own arithmetic
  restated: a rougher mark earns a shorter interval than a solid one, returning
  to an item lengthens it, it rises monotonically across all eleven marks, and
  an unmarked item lands between fragile and consolidated. The thresholds can
  be retuned without rewriting the suite, which is the point. Three tests do
  depend on the literal numbers and will need updating with them: the two
  `suggestion.rs` copy assertions (`the_headline_grades_the_gap…`,
  `the_lowest_mark_breaks_a_tie…`), whose fixtures are constructed to land in a
  specific band, and nothing else.
- **The defect the flag had** gets its own test at both layers: one gap, a
  fragile item and a consolidated one, and they do not go cold together.
- **Bands** at each boundary, and that `Cold` needs double the interval.
- **Ranking**: more overdue on a shorter gap wins; never practised outranks
  everything practised; a gap no interval could survive does not overflow.
  In `suggestion`, the two graded-ranking tests were mutation-checked against a
  raw-days implementation and both fail under it.
- **Copy** as a table over every gap from 0 to 400 days across five marks and
  five return counts, asserting the properties the screen needs (one line,
  sentence case, no banned punctuation, no second person, inside the clause
  budget, no plural on a single unit) rather than sentences picked to agree
  with the formatter (CLAUDE.md, Testing).
- **The data can be wrong**: an unreadable date degrades to the never-practised
  treatment rather than panicking, and a date ahead of the clock reads as today
  rather than wrapping to the stalest item alive. The unreadable branch is
  defensive only: `build_practice_summaries` writes the string from a
  `DateTime<Utc>`, so the parse cannot fail today.
- **The Swift preview fixture** moved with the copy (`previewStarred` said "not
  practised for 6 days", a sentence the core can no longer produce) and its one
  snapshot reference was re-recorded.

## Follow-ups

- A coldness clause on the card's item rows, per decision 7 (#1418).
- The cold shelf in the builder, and then per-item scheduling state: steps 9 of
  the adopted order, each a fresh decision gated on lived use of this one.
- Whether the thresholds are right is an empirical question this PR cannot
  answer (#1419). Use it for a few weeks first. One thing to watch: a fragile
  item now reaches the neglected list at 6 days where the old flag waited 14,
  which is the one place the research's "bias lax" instinct is not applied. It
  has no Swift reader today, so nothing shows it until #764 consumes the list.
