# The rules the app keeps, moved into the core

> Tier 3 (bridge shapes on the `ViewModel`, the `Event` and the picker call).
> Issues #1999 and #1957, part of epic #1995. Native iOS only. Ships as two
> PRs: the core first, the screens in the same sitting. Plan: the comment on
> #1999.

## Problem

The core is meant to own every rule the app shows, but a handful still live in
Swift: which items a filter leaves, how a drag in the session builder becomes a
move, how many items of a finished session were done and which one improved
most, which variations an entry can be tagged to, whether a photo read gave
anything, when "Practise your priorities" can show, and how many of an
exercise's variations are solid. Each is a second definition to keep in step
with the core's, and none can be table-tested beside it. The drag maths is
covered only by two retrying UI tests.

Already settled elsewhere, and not here: the week clock (#2046), the Progress
bars (#1940), the entry ranges (#1512), picker sort and the plain FFI calls
(stay, #1653), the legacy row fold (stays a storage codec).

## Decision: filters

`ListQuery` gains a trailing `priority_only: bool`. `matches_query` drops an
unstarred item when it is set, so the Library and Add to session read the star
filter from `visible_ids` like every other filter. The star leaves both
screens' `@State` and joins the query they already share; Add to session saves
and restores the Library's query around itself (#1440), so the star behaves as
before. Add a related exercise sets `item_type` on its own query scope rather
than filtering by type.

The link picker (the item screen's "link an exercise" and "link a piece", and
the add form's exercise staging) sorts and searches through the plain call
`sort_and_filter_picker_candidates`. Its candidates gain `kind` and
`priority`, and the call gains a `PickerFilterArg { kind, priority_only,
tags }` read by the same predicate as the Library's, so the picker's star, tag
and type scoping stop being Swift. `kind` crosses as a `PickerKind` copy of
`ItemKind`, as the sort field does, since the core stays UniFFI-agnostic.

## Decision: the builder's drag

Two new `SessionEvent`s replace the pair the builder uses today:

```rust
MoveUnit { entry_id: String, new_position: usize },     // the entry's whole unit
MoveRelated { entry_id: String, new_position: usize },  // within its block
```

`MoveUnit` finds the unit holding the entry (a block if it has a `group_id`,
else the entry alone) and moves it to that unit index, clamped to the end, as
`ReorderBlock` does. `MoveRelated` moves a related exercise among its block's
related exercises; the anchor piece stays last. An unknown entry, a piece, an
ungrouped entry or a position past the related run raises an error and changes
nothing.

Which unit or slot a dropped row means is list geometry over rows the app
draws, so it stays in Swift, pulled out of the view into a pure function that
returns the event to send, with a table test. The screens PR deletes
`ReorderSetlist` and `ReorderBlock`, whose only reader is the builder.

## Decision: projections

```rust
pub struct SummaryView {      // appended
    pub completed_count: usize,
    pub top_mover: Option<ScoreChange>,
}
pub struct BuildingSetlistView {   // appended
    pub entry_variations: Vec<EntryVariationsView>,
}
pub struct EntryVariationsView { pub entry_id: String, pub variations: Vec<PickerVariationView> }
pub struct PhotoRecognitionView { /* appended */ pub read_nothing: bool }
pub struct LibraryItemView { /* appended */ pub solid_variation_count: usize }
pub struct ViewModel { /* appended */ pub shows_priorities: bool }
```

- `top_mover` is the Progress rule (the largest rise this week, ties to the
  lower item id) over this session's items, matched by item id, not title. It
  reads every change this week, not the five Progress lists.
- `entry_variations` holds one row per builder entry whose item has
  variations, built by the function behind the Focus Player's
  `current_variations`, so the two sheets cannot disagree.
- `read_nothing` is true only for a finished read whose draft has no title,
  composer, tempo or chart.
- `shows_priorities` is `has_priorities` while nothing is being built, played
  or summarised (#981). The screens PR deletes `has_priorities`, whose only
  reader is that rule.

The builder is outside the `ActiveSession` blob, so `BLOB_VERSION` stands.

## The two PRs

Core: the query field, the picker filter, the two events, the projections, the
spec and the bindings. Swift changes only where the compiler forces them (the
new `ListQuery` and picker arguments), passing "no filter", so nothing on
screen changes.

Screens: every site reads the projection or sends the new events; the Swift
rules and their tests go; `ReorderSetlist`, `ReorderBlock` and
`has_priorities` are deleted from the core.

## Tests

Test first, in the core:

- The star filter alone and with text, type and tags; a starred item outside
  the type still drops.
- The picker filter: kind, star and tags, each alone and together, over the
  same sort; no filter returns what the call returns today.
- `MoveUnit`: a standalone up and down, a block past a standalone, to either
  end, past the end (clamps), an entry inside a block moves its whole block,
  unknown id. `MoveRelated`: to the first and last related slot, past the
  run, the piece, an ungrouped entry, an unknown id. Blocks stay contiguous
  after every case.
- `top_mover` with two items sharing a title, a fall, a first mark, a tie,
  and an item that moved this week but was not in the session.
- `completed_count` for a session ended early with a skipped entry.
- `read_nothing` for an empty draft, for each field alone, and while reading.
- `shows_priorities` in each state that hides it.
- `entry_variations` equals the Focus Player's for the same item.
- A bincode round trip of every new event and field (#846).

Swift, screens PR: the drop-row table; `PracticePrioritiesTests` removed with
the rule it covered; the bridge tests read the new fields through `LiveBridge`;
snapshots unchanged.

## Out of scope

- Parsing a stored key for the key wheel: #2074, after #1939.
- The related sheet's "already under another piece" exclusion: #2075.
- The chord chart counts on the item screen: charts are parked (#2025).
- The session card's day line: #2053.
