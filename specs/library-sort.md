# Library Sort — toolbar sort control

> Tier 3 (Crux core + FFI bridge change + a new persisted singleton). Spec
> rides with the implementation branch per CLAUDE.md. Native iOS only — the
> Leptos/Tauri shells are paused, so no web UI work here.

## Problem

The library list has exactly one order: newest-added-first, hardcoded in the
core at `app.rs` (`items.sort_by(|a, b| b.created_at.cmp(&a.created_at))`).
A musician has no way to reorder it — to find a piece alphabetically, or to
resurface what they've been neglecting. "What haven't I touched in a while?"
is arguably the most useful question a practice app can answer, and today it
can't.

We want a sort control on the right of the existing filter-pills row: tap a
glyph, pick how the library is ordered, and have that choice stick.

## Approach: sort lives in the core; the shell renders the order

Sort is domain logic. Per the dumb-pipe rule the shell sends a `SetSort`
event and renders `viewModel.items` in whatever order the core returns — no
sorting in Swift.

The control is a native SwiftUI **`Menu`** (the Files/Mail/Photos idiom):
field rows with a checkmark on the active field and an up/down chevron for
direction. Re-tapping the active field flips direction; tapping a different
field switches to it at its natural default. A `.popover` was considered and
rejected — it's for richer custom panels, defaults to a sheet on iPhone, and
reads as more chrome for a three-option pick.

### Data model (core)

New types in `domain/types.rs`:

```rust
pub enum SortField { DateAdded, LastPracticed, Title }
pub enum SortDirection { Ascending, Descending }
pub struct LibrarySort { pub field: SortField, pub direction: SortDirection }
```

`LibrarySort` defaults to `DateAdded` / `Descending` (today's behaviour).

A **separate** model field `active_sort: LibrarySort` — deliberately *not*
folded into `ListQuery`. The filter pills rebuild `ListQuery` from scratch on
every tap (`query(for:)` in `LibraryScreen.swift`), so a sort stored there
would be clobbered when you change the type filter. Sort and filter are
orthogonal; they stay separate.

- New `Event::SetSort(LibrarySort)` → sets `model.active_sort`, persists, renders.
- `ViewModel` exposes `active_sort` so the menu reflects current state.
- `view()` replaces the hardcoded sort with a comparator driven by
  `active_sort`, using `created_at` descending as a stable tiebreaker so equal
  keys (same title, same practice date) don't jitter between renders.

**Natural default direction per field** (used when switching *to* a field):

| Field          | Default direction | Reads as            |
|----------------|-------------------|---------------------|
| Date Added     | Descending        | Newest first        |
| Last Practiced | Descending        | Most recently first |
| Title          | Ascending         | A → Z               |

### "Last practiced" data

Add `last_practiced_at: Option<String>` to `ItemPracticeSummary`, computed in
`build_practice_summaries` as the **max `session.started_at`** across the
item's entries. Today only *scored* / *tempo'd* entries carry a date in the
summary; this is a dedicated field so a plain practice still counts.

**Never-practiced = "ages ago":** the comparator treats `None` as the earliest
possible time. So under Last-Practiced *ascending* ("longest since practiced")
never-practiced items rise to the **top** alongside neglected pieces; under
*descending* they sink to the bottom. One consistent rule.

### UI (SwiftUI)

In `LibraryScreen.swift`, the filter row becomes an `HStack`: `LibraryFilterTabs`
stays left (still horizontally scrollable), the sort `Menu` pins right.

- **Icon only:** `arrow.up.arrow.down` — the standard SF Symbol for *sort*
  (not the funnel `line.3.horizontal.decrease`, which means *filter*).
- Three rows: **Date Added**, **Last Practiced**, **Title**. Active field
  shows a checkmark + a direction chevron; the menu is bound to `active_sort`
  and emits `.setSort(...)`.
- Accessibility: button labelled "Sort"; each row announces field + direction.
  Native `Menu` gives Dynamic Type, VoiceOver, and light/dark for free.

### Persistence — remember the choice

`crux_kv` is **not** wired into the core today; singletons currently persist
via the custom `AppEffect` pattern (`SaveSessionInProgress` /
`ClearSessionInProgress`). The sort preference follows the same lightweight
path rather than standing up a new capability:

- **Write:** `SetSort` emits a fire-and-forget `AppEffect::SaveLibrarySort(LibrarySort)`;
  the shell writes it to `UserDefaults` (iOS) — a tiny blob, not relational data.
- **Read:** at launch the shell loads the stored sort and **re-dispatches the
  same `SetSort` event** with it (no `StartApp` signature change, no new
  read-effect — one event powers both the menu and restore). The restore
  re-emits the save effect with the same bytes, which is harmless/idempotent.

Offline-first invariants hold: it's a small singleton (invariant 8), no network
on the path (1), and no GRDB schema/migration (the practice summary is a derived
view type, not a persisted table).

## Key decisions

1. **Sort in the core, not Swift.** Dumb-pipe rule; the comparator is shared,
   Android-ready, and unit-testable.
2. **`active_sort` separate from `ListQuery`.** Filter rebuilds the query each
   tap and would otherwise wipe the sort.
3. **Native `Menu`, not `.popover`.** The canonical iOS sort idiom; free
   a11y/Dynamic Type/theming.
4. **`None` last-practiced = oldest.** Never-practiced and neglected surface
   together under "longest since practiced" — the most useful sort.
5. **Persist via the existing `AppEffect` singleton path**, not a new
   `crux_kv` capability — least surface area for one small value.
6. **One PR, no phasing.** Sort + persistence ship together.

## Deliberately not doing (v1)

- **Sort by Priority / Key / Tempo / score.** Three fields cover organise,
  find, and resurface. Add more only if asked.
- **Per-filter sort memory** (a different sort per type tab). One global sort.
- **Standing up `crux_kv`.** Deferred until a second singleton justifies it.

## Open questions

- Exact direction-chevron treatment in the menu rows (trailing `chevron.up`/
  `chevron.down` vs. a `Picker`-style checkmark only) — decide against the
  running app during UI build.
- Whether `Title` sort should be locale-aware (`localizedStandardCompare`) in
  the shell vs. a plain case-insensitive compare in the core. Core-side keeps
  it shared/testable; revisit if non-ASCII titles sort oddly.

## Testing

- **Core (TDD, write the failing test first):** each field × direction
  produces the expected order; never-practiced sorts as oldest (top in asc,
  bottom in desc); title compare is case-insensitive; stable tiebreaker on
  equal keys; `SetSort` updates `active_sort` and emits the save effect;
  `StartApp` applies a passed-in sort; `last_practiced_at` is the max session
  date. Exercise in both `local_first` and online modes (invariant 6).
- **iOS:** swift-snapshot-test of the menu control; VoiceOver labels verified;
  preview-driven check that changing sort reorders the list and survives a
  relaunch.
- Bindings regenerated via `just ios-gen` (never hand-edited).
