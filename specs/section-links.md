# Section links: an exercise aimed at the parts of a piece it prepares

> Tier 3 (a bridge shape and a GRDB migration). Rides as the first commit of
> the core PR for #2248, step 4 of epic #50. Closes #2232 and the #1363
> exploration. Both shells regenerate bindings.

## Problem

A scale drill that prepares bar 12 of one piece and the coda of another can
only be linked to each piece as a whole, so its history cannot say which part
it serves. Today the link is a list of exercise ids on the piece
(`Item.linked_exercise_ids`, a JSON column on `item` since `v6`). An exercise
can already sit on several pieces; what it cannot do is name a section.

Linking is also diffed in Swift: the piece screen and the exercise screen each
work out what to link and unlink and send one event per change (#2232). Left
there, the Android screens would need a second copy of that rule.

## Data shape

New module `domain/link.rs`. Links live on the piece, beside its sections,
tombstones included:

```rust
pub struct ExerciseLink {
    pub id: String,                  // ulid
    pub exercise_id: String,
    pub section_id: Option<String>,  // None: the whole piece
    pub position: usize,             // order on the piece's card
    pub updated_at: DateTime<Utc>,   // per-row LWW (offline-first)
    pub deleted_at: Option<DateTime<Utc>>,
}
```

`Item` loses `linked_exercise_ids` and gains `exercise_links: Vec<ExerciseLink>`,
appended last with `#[serde(default)]`. Only pieces hold links; an exercise's
own list is always empty.

- **A link is one row per exercise and target.** Linking a drill to A2 and to
  the coda of the same piece is two rows. Rows reconcile on
  (exercise, section), so linking again after an unlink revives the old row
  and keeps its id.
- **Whole piece and sections together are allowed.** One exercise can hold a
  whole-piece link and section links on the same piece at once, with no rule
  between them (Jon, 2026-10-04).
- **The host is a piece.** A section on an exercise cannot be a link target,
  as today only a piece can host one (`validate_link_exercise`).
- **Unlinking tombstones.** Nothing reads a link's history yet, but #2250's
  section scores may, and a hard delete now is a rule to unpick later, as
  sections decided.

## Events

Two variants appended last on `ItemEvent` (the wire is positional), each the
whole chosen set in one write, the pattern `UpdateSections` set:

```rust
SetPieceLinks { piece_id: String, links: Vec<LinkEdit> },
SetExerciseLinks { exercise_id: String, targets: Vec<LinkTarget> },

pub struct LinkEdit { pub exercise: ScaffoldEntry, pub section_id: Option<String> }
pub struct LinkTarget { pub piece_id: String, pub section_id: Option<String> }
```

- **`SetPieceLinks`** is the piece screen's card: list order is the card order.
  `ScaffoldEntry::Existing` names an exercise; `ScaffoldEntry::New` creates one
  and links it, replacing the per-draft `AddLinkedExercise` loop (#1431, #2224).
  A live link the list leaves out is tombstoned.
- **`SetExerciseLinks`** is the exercise screen's "Used in" picker. It touches
  every piece it adds to or removes from, saved in one `SaveItems` batch so a
  failed write leaves no half-linked exercise (#1106).
- **Both refuse whole on the first invalid row, nothing saved:** an unknown or
  non-exercise item, a host that is not a piece, a section that is not a live
  section of that piece, a refused draft. A set identical to what is stored
  writes nothing and leaves `updated_at` alone.
- **Adding and removing in one change is one event** (#2232's done line).

`LinkExercise`, `UnlinkExercise`, `ReorderLinkedExercises`, `AddLinkedExercise`,
`CommitScaffold` and `AddPieceInFull` keep working in the core PR, rewritten
over the link table and making whole-piece links only. The screens PR moves
both screens to the two new events, then deletes the first three, which have
no other reader (#1176). The last three stay: they create items.

## View

- **`LinkedExerciseView`** (the piece's card) gains `whole_piece: bool` and
  `sections: Vec<LinkedSectionView { id, label }>` in score order, the label
  taken from `SectionView`. One row per exercise, ordered by its first link's
  position.
- **`ExerciseUsageView`** (the exercise's "Used in" rows) gains the same two
  fields on linked rows, so the exercise shows "Nocturne, A2" and "Etude,
  coda" (#2248's first done line).
- Tombstoned links, links to tombstoned sections and links to deleted
  exercises are hidden, as a dangling id is today.
- Building a session from a piece (`domain/session/building.rs`) and Up next
  (`suggestion.rs`) read the distinct live exercises, so a section-linked
  drill is offered with its piece as a whole-piece one is now. Offering by
  section waits for #2249.

## Deletion

- **A section removed** by `UpdateSections` tombstones every live link naming
  it in the same `SaveItem`, stamped with the same time. The exercise row is
  not written. A whole-piece link to the same exercise is untouched.
- **An exercise deleted** leaves its link rows; views hide them, as today.
- **A piece deleted** takes its links with it: links load through their piece.

## Persistence and migration

GRDB migration `v20_exercise_link` (the next free name when the core PR is
built):

```sql
CREATE TABLE exercise_link (id TEXT PRIMARY KEY NOT NULL,
  piece_id TEXT NOT NULL, exercise_id TEXT NOT NULL, section_id TEXT,
  position INTEGER NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT);
CREATE INDEX index_exercise_link_on_piece_id ON exercise_link(piece_id);
INSERT OR IGNORE INTO exercise_link
  SELECT 'link|' || item.id || '|' || j.value, item.id, j.value, NULL,
         MIN(j.key), item.updated_at, NULL
  FROM item, json_each(CASE WHEN json_valid(item.linked_exercise_ids) THEN
      CASE WHEN json_type(item.linked_exercise_ids) = 'array'
      THEN item.linked_exercise_ids END END) AS j
  WHERE j.type = 'text'
  GROUP BY item.id, j.value;
```

- **The copy is the migration's SQL, not a core fold** (Jon, 2026-10-04), on
  the precedent of `v17_item_metre`. A one-time move by the core, as #2246 does
  for steps, was rejected: the core would keep reading the old column.
- **Existing links come across as whole-piece links, in the same order.** A
  repeated id becomes one row at its first position. The id is derived, so the
  copy is deterministic and the test can name the rows.
- **A value that is not a JSON array is skipped, not cleared; entries that
  are not text are dropped.** The id joins with `|`, which no id holds, so
  hyphenated ids cannot collide. `linked_exercise_ids`
  stays on `item`, no longer read or written; dropping it rides the coach-era
  table release (#2317), as `variant` does. An unreadable list therefore keeps
  its only copy (#1117).
- **Writes ride the item's transaction.** `LibraryStore.swift` upserts each link
  row beside the item and its sections, keyed by id, with no delete-missing;
  loads read tombstones too. `ItemCodec.swift` stops reading the old column.
- **No new persistence operation:** `Item` carries its links.
- **Android:** the in-memory store needs only regenerated bindings and
  `Fixtures.kt`; the androidx.sqlite store creates `exercise_link` at birth.
- **The practice in progress blob is untouched:** `ActiveSession` holds no
  `Item`, so `BLOB_VERSION` does not move.

## What crosses the bridge

`ExerciseLink`, `LinkEdit`, `LinkTarget`, `LinkedSectionView`, the `Item` field
swap, the two view fields, two `ItemEvent` variants. Appended last; no
JSON-only serde attributes (#846).

## Out of scope

Offering or scoring by section (#2249, #2250); links hosted on an exercise's
sections; dropping the old column (#2317); the screens, which are the second PR.

## Two PRs

1. **Core, this spec first:** types, events, validation, views, the rewritten
   link handlers, `v20_exercise_link`, `LibraryStore.swift`, `ItemCodec.swift`,
   the fixtures, both bindings, the bridge test. Nothing visible changes. Its
   Swift half waits for #2247 to merge (parallel streams).
2. **Screens:** the piece card and the exercise's picker send the two new
   events and choose sections; the three retired events go.

## Tests (test-first)

- **`SetPieceLinks`:** add a whole-piece link, add a section link, add and
  remove in one event, reorder, a new draft created and linked to a section;
  refused: an unknown section, another piece's section, a tombstoned section,
  a piece as the exercise; a refused row saves nothing; an identical set emits
  no `SaveItem`; unlink then relink keeps the row id.
- **`SetExerciseLinks`:** A2 of one piece and the coda of another saves both
  pieces in one batch and both show on the exercise's "Used in" rows.
- **Deletion:** removing A2 in `UpdateSections` tombstones only A2's link; the
  exercise is not saved; the whole-piece link stays. Deleting the tombstone
  line must fail this test.
- **Unchanged behaviour:** the existing link, scaffold, add-in-full, building
  and Up next tests pass over the link table.
- **Views:** section labels in score order, tombstones hidden, links to a
  removed section hidden.
- **Bridge:** both events and an `Item` with links round-trip through
  `LiveBridge` (#846).
- **iOS, core PR:** a database built at `v19` with `["a", "b", "a"]` on a piece
  loads two whole-piece links in order a, b; an unreadable column loads none
  and keeps its text; a link tombstone survives a reload; `just ios-test`.
