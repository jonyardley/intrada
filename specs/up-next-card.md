# The Up next card

> Tier 3 spec. Issue [#1082]. The suggestion is the Practice hero
> (design-principles T15). Design:
> [`up-next-card/design/up-next-card.dc.html`](up-next-card/design/up-next-card.dc.html).

[#1082]: https://github.com/jonyardley/intrada/issues/1082

One derived projection, `ViewModel.up_next`, built by `rank_blocks` in
`suggestion.rs`, and one event that seeds the builder from it. It suggests and
never gates: `up_next` is `None` when nothing qualifies.

## Key decisions

1. **Input is the projected view list, not the model.** `rank_blocks` takes
   `&[LibraryItemView]`, before filter and sort, and a `LocalClock`, so the card
   and the Library screens cannot disagree about a mark.

2. **Ordered tuples, not a weighted score.** Every reason is shown in words, so
   the ranking has to be sayable. Every level ends in a deterministic
   tie-break.

3. **The anchor piece is the unit.** Candidates are pieces with at least one
   live linked exercise, ranked by: the priority star first; the piece's
   **own** staleness, most stale first; its latest mark, lowest first; then
   title and id. The headline reason says the same thing the ranking used.

4. **Two exercises, then the piece.** Up to two of the piece's live linked
   exercises, ranked by the mark the row will show (in this piece's context),
   lowest first, then staleness, then link order; the anchor piece last.

5. **An exercise with variations brings its first unsolid one**, and the
   seeding event attributes the entry to it. Where every variation is solid,
   the exercise rides along unattributed.

6. **Reasons are core-owned strings.** The shell renders, never composes; the
   star is drawn from `SuggestedSession.priority`, never from the reason text.
   One clause per item row, up to two for the headline, joined with ` · `.

   | Where | String |
   |---|---|
   | Card headline | `A priority · going cold after 3 weeks` |
   | Card headline, unstarred | `Cold for 3 months` / `Practised 6 days ago` / `Practised yesterday` / `Practised today` / `Not practised yet` |
   | Exercise row | `Marked 4 of 10 last time` / `Not marked with this piece` |
   | Step row | `Marked 4 of 10 last time` / `Variation not marked yet` |
   | Piece row | `Marked 6 of 10 last time` / `Not marked yet` |

   The headline says *when*, each row says *how it went*. Nothing on the card
   prints the same sentence twice. The headline is the getting-cold clause
   (#1416); see [`getting-cold-signal.md`](getting-cold-signal.md) decision 7
   for why rows wait (#1418).

7. **The estimate is drawn from real minutes**: each item's average time,
   5 minutes for one never practised, summed and rounded to the nearest 5,
   floor 5. It is an estimate, not a target.

8. **One event, no derived content on the wire.**
   `SessionEvent::StartBuildingFromSuggestion { now }` re-derives the
   suggestion and seeds the builder. Valid from `Idle` only, and a no-op when
   nothing is derived.

9. **Dismissal is shell state, and never a one-way door.** "Build my own
   instead" hides the card and opens the empty builder; "Show suggestion"
   brings it back. No domain consequence, no persistence (T22).

10. **No wire hazard to manage.** `ViewModel` is never stored, and
    `ActiveSession` is untouched.

## Open questions

- **One card or a short list.** One, for now.
- **Standalone exercises and unlinked pieces** are invisible to this card by
  decision 3. If that turns out to be most of a real library, the fix is
  linking, not widening the derivation.
