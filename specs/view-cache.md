# Build the library and Progress once per change, not once per render

> Tier 3 (a bridge shape on the `ViewModel`). Issues #1998 and #1956, part of
> epic #1995. Native iOS only. Ships as two PRs: the core first, the screens in
> the same sitting. Plan: the comment on #1998.

## Problem

Every tap that ends in a render rebuilds every Library row (each exercise's
use across pieces, each variation's scores over every session), Up next, the
week strip, the session list and all of Progress. A pass tap mid-practice pays
for the whole history, and the cost climbs with every session saved.

The library also crosses the bridge three times per render: `items` (the
filtered list), `all_items` (the same rows unfiltered) and
`recently_practised` (five more copies).

## Decision: cache the projections in `update`

`view()` takes `&Model`, so the cache is filled at the end of `update` and only
read in `view`.

`Model.items`, `Model.sessions` and `Model.practice_summaries` become a
`Tracked<T>`: reading goes straight through, any mutable access takes a new
revision from a process-wide counter. No handler can forget to invalidate, and
replacing the whole value (`model.items = v.into()`) also takes a fresh
revision, so a new value can never match an old key. A mutable borrow that
changes nothing still invalidates; that costs one rebuild, never a stale
screen.

The key is the three revisions, the local day, the UTC offset and the sort.
The local day and the offset are there because staleness, Up next, the week
strip and Progress all read `LocalClock.today` (#1694): without them the
screens would go stale at midnight. `update` rebuilds when the key moved.
`view` uses the cache only when its key still matches the clock; after a
midnight with no event in between, it builds fresh without storing, and the
next event refills it.

Cached: the library rows in sort order, the tag and composer vocabularies,
Up next, `has_priorities`, recently practised, the session list, the week
strip, Progress and "last practised". Built per render: the filter, the
counts under it, the session being built or played, the summary, the banner
state, the profile greeting (it reads the hour) and the page reader.

No skip for Progress during a session. With the cache nothing is rebuilt
until a session is saved, and the summary reads `analytics.score_changes`.

## Decision: the library crosses once

The filtered list becomes ids into the whole library, not a filter the shell
applies (#1999 moves rules the other way).

```rust
pub struct ViewModel {
    pub items: Vec<LibraryItemView>,        // the whole library, in sort order
    pub visible_ids: Vec<String>,           // the filtered subset, same order
    pub recently_practised_ids: Vec<String>, // up to 5, most recent first
    // all_items and recently_practised are gone
}
```

Every picker reads `items`. Today three of them (Add to session, Add related
exercise, the entry settings variation list) read the filtered list, so a
Library search hides their rows; this fixes that. Their own type and priority
filters in Swift stay until #1999.

## The two PRs

Core: the cache and the two id lists appended to `ViewModel`. `items`, `all_items` and `recently_practised` are unchanged, so the
app builds and behaves as before.

Screens: the Library screen and the split view render `visible_ids` looked up
in the library, the builder's quick-add reads `recently_practised_ids`, the
pickers read the whole library. Then `items` becomes the whole library, and
`all_items` and the `recently_practised` rows are deleted from the core.

## Tests

Test first, in the core:

- A cached render equals a fresh build after every event family: items added,
  edited and deleted, a practice saved and acknowledged, the store loading,
  the sample data, the offset changing, the sort changing.
- The cache is kept (the same allocation) across a filter change, a banner
  dismissal and a tap during a running practice.
- A render the day after the last event equals a fresh build for that day.
- `visible_ids` follows the filter and the sort; `recently_practised_ids`
  matches the rows it replaces.
- `test_performance_10k_items` asserts a cached render against a tight
  budget, and keeps the cold build under its current bound.
- A bincode round trip of the view model through the FFI wire (#846).

Swift, screens PR: `LibraryQueryFilterTests` read `visible_ids`; the bridge
tests that read `items.first` keep working, since `items` only grows.

## Out of scope

- The `ViewModel` clone per render: `view` returns an owned value.
- `derive_up_next` and `derive_priorities`, which build fresh on the tap that
  starts a practice.
- Moving the pickers' own filters into the core (#1999).
