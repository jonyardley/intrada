# One section or link change at a time

> Tier 3 spec (rides as the first commit of the core PR). Issue: #2447.
> Epic: #2220. Background: the shell rules review in #2223.

## Problem

The core takes a piece's sections and its exercise links only as whole lists:
`UpdateSections` carries every section, `SetPieceLinks` every link. So the
iPhone builds those lists in Swift on every change:

- `SectionEdits.saving` and `.removing` copy each section back into a
  `SectionEdit` and splice one row in or out.
- `SectionsSection.finishReordering` copies every section again in the new
  order.
- `ExerciseLinkSets` rebuilds the piece's link set with one exercise's rows
  replaced, or the exercises in a new order.

Where a new section goes, how a moved exercise reorders the rest and what
ticking a section does to an exercise's other links are domain rules. The
Android piece page (#2442) would have to copy them into Kotlin, and the two
copies would drift.

## Approach

Two new `ItemEvent`s, each carrying one change. The core builds the whole list
from what it holds and hands it to the existing whole-list writes, so
validation, reconciliation, tombstones and persistence stay where they are.

```rust
ChangeSection { id: String, change: SectionChange }
ChangePieceLink { piece_id: String, change: LinkChange }

enum SectionChange {
    Save(SectionEdit),                    // no id: new, last; an id: that section, in place
    Remove { section_id: String },
    Arrange { section_ids: Vec<String> }, // new order; any left out are removed
}

enum LinkChange {
    Set { exercise_id: String, whole_piece: bool, section_ids: Vec<String> },
    Move { exercise_id: String, to: usize },
    Unlink { exercise_id: String },
}
```

### Sections

- **Save** with no id appends a new section after the last. With an id it
  replaces that section where it stands. An id that is not a live section of
  the item is refused, so a sheet left open after the section was removed
  never brings it back as a new one.
- Only the incoming row is validated. The others go through as the core holds
  them, so a limit tightened since they were saved cannot refuse an unrelated
  change.
- **Remove** of a section that is already gone changes nothing. Removing one
  still takes its links with it, never the exercise (#2248).
- **Arrange** is the reorder list's Done: one write for the new order and the
  removals, so a refusal loses nothing. An id that is not a live section, or
  one named twice, refuses the whole change: a list carried over from another
  item must never remove this item's sections.

There is no single-step move for sections. The iPhone saves a reorder on Done,
not per drag step, and Android follows it; a move event would have no sender.

### Links

The piece's card lists each linked exercise once, in the order of its first
live link. A change works on that card order.

- **Set** replaces one exercise's links in place on the card: the whole piece
  first when ticked, then its sections in score order, whatever order they
  were ticked in. An exercise not yet on the card goes last. Nothing ticked
  takes it off the piece.
- **Move** puts one exercise at `to` on the card, clamped to the last place.
  An exercise not on the card changes nothing.
- **Unlink** takes one exercise off the piece; one not on it changes nothing.

Each builds the full set and calls the code behind `SetPieceLinks`, which
validates the sections, numbers the positions and reconciles rows by exercise
and section, reviving a tombstone before minting.

## Retiring the whole-list events

The iPhone PR moves every sender to the new events. `UpdateSections` and
`SetPieceLinks` then have no sender in either shell and leave the bridge in
that PR; the bridge tests that seeded data with them move to the new events.
Their handlers stay as the core's internal writes.

`SetExerciseLinks` already has no sender in the app (the piece picker from
#2379 replaced it). It is noted on #2223 and left alone here.

## Release check

The release is right if:

- On the iPhone, adding, renaming, removing and reordering sections, and
  ticking sections for an exercise, moving it and removing it, behave as
  before.
- Nothing under `ios/Intrada/` builds a `[SectionEdit]` or `[LinkEdit]`.
- `just ios-test` and the existing section and link UI tests pass unchanged.
- #2442 can build the Android page sending only these events.

## Tests

Table tests in `domain/item/tests.rs`, from what a musician does: add a third
section, rename the middle one, remove one with a linked drill, arrange leaving
one out, arrange with a stranger's id, move the last exercise to the top, tick
B then A, untick everything, change a removed section. A bincode round trip
covers both change enums.

## Out of scope

- The exercise-side link events and the two pickers from #2379.
- The text the Bars field opens with (`SectionEdits.barsText`), which formats
  values the core already holds.
- The Android piece page itself (#2442).
