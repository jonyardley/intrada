# The Up next card

> Tier 3 spec. Issue [#1082], step 6 of the adopted order in
> [`docs/research/comparison.md`](../docs/research/comparison.md). Ships in two
> PRs: **core first** (this spec, the derivation, the `ViewModel` projection,
> the seeding event, tests), **screens second** (the card on the Practice tab,
> snapshot + VoiceOver). Scope: `intrada-core` + native iOS. No API, no
> migration, no new storage.

[#1082]: https://github.com/jonyardley/intrada/issues/1082

## Problem

The Practice tab asks "what do you want to practise?" and offers a blank
builder. Everything needed to answer it honestly is already derived and on
screen elsewhere: per-piece marks for each related exercise (#1095/#1097), the
step ladder's current rung (#1083), last-practised dates, and the priority
star. Nobody has put them in one place at the point of decision.

This is the first honest version of "what do I practise today?", weeks before
any scheduler. While a piece has related exercises linked, the answer is
usually simple: resume that block, weakest and stalest first.

## Approach

One derived projection, `ViewModel.up_next`, and one event that acts on it.

```text
LibraryItemView[]  (unfiltered, already carries priority, per-piece marks,
     │              step ladders, last-practised)
     │  compute_up_next(items, clock)          // pure, suggestion.rs
     ▼
SuggestedSession { piece, reason, estimated_minutes, items[2..=3] }
     │                                    each item: reason, mark, step
     ▼
ViewModel.up_next: Option<SuggestedSession>
     │  SessionEvent::StartBuildingFromSuggestion { now }
     ▼
BuildingSession: one block, exercises first, the piece last, steps attributed
```

The derivation reads the same `LibraryItemView` list the Library screens read,
before the filter and sort are applied. That is deliberate: the card and the
piece-detail rings cannot disagree about a mark, because there is only one
derivation of it. To share it, the item-projection loop comes out of
`build_view` as `build_library_item_views(model)`, which both the view and the
seeding event call. The move is what makes the seeding event testable: the
derivation takes an injected clock rather than reaching for `Utc::now()` the
way `build_view` does.

**Suggest, never gate.** `up_next` is `None` whenever nothing qualifies, and
nothing else on the Practice tab changes. The manual builder keeps working
exactly as it does today.

## Key decisions

1. **Input is the projected view list, not the model.** `compute_up_next` takes
   `&[LibraryItemView]` and a `LocalClock`, nothing else. Cheaper than
   re-deriving from sessions, impossible to drift from the Library screens, and
   the whole thing is a pure function with one obvious fixture. Cost: staleness
   parses `last_practiced_at` back out of RFC3339, which is trivial and tested.

2. **Ordered tuples, not a weighted score.** Every reason the card gives is
   shown in words, so the ranking has to be sayable. A weighted score is not.
   Ranking is lexicographic on the fields below, and every level ends in a
   deterministic tie-break so two renders never disagree.

3. **The anchor piece is the unit.** Candidates are pieces with at least one
   live linked exercise. A piece with no exercises has no block to resume, and
   an exercise on its own is not a session. Ranked by:

   | # | Key | Direction |
   |---|-----|-----------|
   | 1 | `priority` | starred first |
   | 2 | days since the piece was last practised | most stale first, never practised counts as stalest |
   | 3 | the piece's own latest mark | lowest first, unmarked first |
   | 4 | title (case-folded), then id | ascending, determinism only |

   The star is first because it is the user's own voice, and the card must not
   argue with it. Staleness is the piece's **own** last-practised, not the
   block's most recent: a tune you drilled around yesterday but have not played
   for a fortnight is exactly the one to suggest, and the headline reason then
   says the same thing the ranking used.

4. **Two exercises, then the piece.** The suggested setlist is up to two of the
   piece's live linked exercises, weakest and stalest first, followed by the
   anchor piece. Two to three items, matching the mockup, and the order matches
   how the builder already lays out a block (`AddExerciseToBlock` positions
   exercises before the anchor). Exercises are ranked by:

   | # | Key | Direction |
   |---|-----|-----------|
   | 1 | the mark the row will show: the current step's where a step was chosen, otherwise the mark in **this piece's** context (`piece_context_score`) | lowest first, never marked first |
   | 2 | days since the exercise was last practised | most stale first, never practised first (the same convention as the anchor) |
   | 3 | link order, then id | ascending, determinism only |

   Deliberately not the exercise's flat mark: per-piece is the grain #1081
   established, and a drill that is solid under one tune can be rough under
   another. Ranking on the mark the row will *show* is one rule rather than
   two: the row the card puts first is always the row whose reason says why.

5. **A laddered exercise brings its current step.** Where an exercise has a
   step ladder, the suggestion carries the `is_current` step (the first rung not
   yet solid) as `variant_id` + `variant_label`, and the seeding event
   attributes the entry to it. Its reason and mark then read from the step, not
   the exercise. A fully solid ladder has no current step; the exercise rides
   along unattributed.

6. **Reasons are core-owned strings.** The shell renders, never composes. One
   clause per item row, up to two for the card headline, joined with the house
   separator ` · `. Written against
   [`docs/tone-of-voice.md`](../docs/tone-of-voice.md): British, sentence case,
   no full stop, no second person (this is chrome, V2), `priorities` not
   `starred` (V3), `mark` not `score` (V4).

   | Where | String |
   |---|---|
   | Card headline | `A priority · not practised for 6 days` |
   | Card headline, unstarred | `Not practised for 6 days` / `Practised yesterday` / `Practised today` / `Not practised yet` |
   | Exercise row | `Marked 4 of 10 last time` / `Not marked with this piece` |
   | Step row | `Marked 4 of 10 last time` / `Step not marked yet` |
   | Piece row | `Marked 6 of 10 last time` / `Not marked yet` |

   The piece row says its mark rather than repeating the headline's staleness:
   the headline says *when*, each row says *how it went*. Nothing on the card
   prints the same sentence twice.

7. **The estimate is drawn from real minutes.** Per item, the average time
   actually spent on it (`total_minutes / session_count`); items never
   practised fall back to `UNPRACTISED_ESTIMATE_MINS` (5 min). Summed, rounded
   to the nearest 5, floor 5, so the CTA reads `Start · 15 min`. It is an
   estimate of what this usually takes, not a target, and nothing enforces it.

   The fallback is the suggestion's own const, not
   `validation::DEFAULT_PLANNED_DURATION_SECS`, which this PR deletes: it had
   no reader, and borrowing it would have meant a later change to "the
   builder's default planned duration" silently moving this estimate instead.

8. **One event, no derived content on the wire.**
   `SessionEvent::StartBuildingFromSuggestion { now }` re-derives the
   suggestion and seeds the builder from it, the same way `CommitScaffold`
   re-derives rather than trusting the shell's copy. The shell sends no item
   ids, no reasons, no minutes. Valid from `Idle` only; a no-op with the
   standard "A practice is already in progress" error otherwise, and a no-op
   when the derivation yields nothing (the CTA cannot be on screen in that
   case, but a race must not panic).

   `now` is carried because the derivation is clock-dependent, matching
   `StartSession { now }`. The seeded block gets one fresh `group_id`,
   exercises before the piece, `variant_id` set where a step was chosen, and no
   planned durations, because the estimate is a projection, not stored state.

9. **Dismissal is shell state.** "Build my own instead" hides the card for the
   app run and is `@State` in Swift: no domain consequence, no persistence, and
   the issue's "no new storage" holds. If it should ever stay dismissed across
   launches it becomes a `crux_kv` singleton, which is a later decision, not
   this one.

10. **No wire hazard to manage.** `up_next` is appended to `ViewModel`, which is
    rebuilt every render and never stored, so the positional-bincode
    stale-blob hazard does not apply. `ActiveSession`, the one graph that is
    persisted as positional bincode, is untouched, so no UserDefaults key
    bump. Bindings regenerate as a build precondition.

## Testing

Test-driven, per CLAUDE.md: the derivation is pure `intrada-core` logic, so the
tests come first. Fixture-based (`LibraryItemView::fixture()` composed with
struct update) so a new view field costs one edit.

- **Ranking**, one test per level of each tuple, each asserting the level it
  names is what decides: a starred fresh piece beats an unstarred stale one, a
  stale piece beats a fresh one at equal star, a lower mark breaks a staleness
  tie, and equal-on-everything falls to title.
- **Eligibility**: a piece with no live linked exercise is never suggested; an
  exercise is never the anchor; an empty library and a library with no linked
  pieces both yield `None`.
- **Setlist shape**: at most two exercises, the piece last, one shared
  `group_id`, the weakest exercise first, never-marked-here ahead of marked.
- **Steps**: a laddered exercise carries its current rung and reads that rung's
  mark; a fully solid ladder carries none.
- **Reasons**, as a table of realistic states asserting the property the next
  stage needs: every reason is a non-empty single-line string inside the copy
  budget, with no banned punctuation. Not hand-picked sentences that agree with
  the formatter by construction (CLAUDE.md → Testing).
- **Estimate**: averages from real minutes, falls back to 5 for the unpractised,
  rounds to 5, never returns 0.
- **The event**: seeds exactly the derived block from `Idle`; refuses when a
  practice is in progress; is a no-op when the derivation is empty; sets
  `variant_id` where a step was chosen.
- **FFI**: `assert_round_trips` on the new event and on `SuggestedSession`
  through `BincodeFfiFormat`, before either is wired to a screen.
- **Local-first**: the derivation is read-only over data already in the model,
  so the event's effects are asserted to be `Render` and nothing else.
- **The pre-filter invariant has its own test**: with the library filtered to
  exclude the anchor piece, `up_next` still names it. A stated invariant has to
  be an enforced one, and this one is a single line of refactoring from
  inverting silently.

## Open questions (deferred, tracked on #1082)

- **One card or a short list.** One is the decision here (one primary action per
  screen). A second-choice row is a later call, once the card has been used.
- **The getting-cold signal** (step 8 of the adopted order) lands as new reason
  lines on this card, replacing the flat day count with a graded estimate. The
  reason strings are the seam it arrives through, which is why they are
  core-owned from the start.
- **Standalone exercises and un-linked pieces** are invisible to this card by
  decision 3. If that turns out to be most of a real library, the fix is
  linking, not widening the derivation.
