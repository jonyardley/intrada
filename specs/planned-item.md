# One planned-item shape for sessions, routines and the week

> Tier 3, sensitive surfaces (the crash-recovery blob and a bridge change that
> retypes fields). Issue #2582, decision 5 of the data model review (#2577,
> ruled 2026-10-13). One PR across the stack: core commits first, reviewed
> alone, then the iPhone and Android commits. Nothing changes on screen, and
> no store migration.

## Problem

What a session plans for an item lives only inside a session entry
(`SetlistEntry`), mixed in with what then happened. Routines (#1348) and the
week's practice set (#2579) need exactly the plan half as a list, and sync
will need one record kind for each. Without a shared type, each of them grows
its own copy of the same ten fields, and the copies drift the first time
someone adds a plan field.

## The type

```rust
/// What is planned for one item: enough to start practising it, and nothing
/// about how it went.
pub struct PlannedItem {
    pub item_id: String,
    pub item_type: ItemKind,
    pub intention: Option<String>,
    pub focus: Option<IntentionFocus>,
    pub duration_secs: Option<u32>,
    pub group_id: Option<String>,
    pub variation_ids: Vec<String>,
    pub rep_target: Option<u8>,
    /// The sections, in order, with their time.
    pub segments: Vec<Segment>,
    /// `None` is the written key.
    pub key: Option<Key>,
}
```

Inside the type the `planned_` prefixes go: the type name says it. It gets
exactly the derives `SetlistEntry` has and no serde attributes, because a
JSON-only attribute breaks the bincode bridge silently (#846).

## The session entry

```rust
pub struct SetlistEntry {
    pub id: String,
    pub plan: PlannedItem,
    pub item_title: String,
    pub position: usize,
    pub duration_secs: u64,
    pub status: EntryStatus,
    pub notes: Option<String>,
    pub plays: Vec<Play>,
    pub intention_met: Option<IntentionMet>,
    pub felt: Option<Felt>,
    pub got_in_the_way: Vec<Obstacle>,
    pub note_points: Vec<NotePoint>,
}
```

What stays on the entry, and why:

| Field | Why it is not plan |
|---|---|
| `id` | names this entry in this session; a routine's rows get their own |
| `item_title` | a copy taken on the day, which #2583 replaces with the live name |
| `position` | the slot in the list; a list of plans is ordered by itself |
| `duration_secs`, `status`, `plays` | what happened |
| `notes`, `intention_met`, `felt`, `got_in_the_way`, `note_points` | the answers after playing |

`item_type` moves with `item_id`: a plan that names an item says what kind it
is, so a routine can be shown before the library loads.

Starting a session from a list of plans is a copy:
`SetlistEntry::from_plan(id, position, item_title, plan)` sets every played
field to empty. The title comes in beside the plan because it belongs to the
entry until #2583. The builder's existing entry constructor calls it, so
there is one way to make an entry.

## How routines and the week hold it

A routine is a name and a `Vec<PlannedItem>`. A week's set is a date range
and a `Vec<PlannedItem>`, routines inside it per #2579. Neither is built
here. When sync arrives, each is its own record kind carrying its list; a
`PlannedItem` is never a record on its own, because it has no life outside
its list.

## The three ways an entry is written down

1. **Crash recovery (positional bincode).** The shape changes, so
   `ActiveSession::BLOB_VERSION` goes from 8 to 9 in the core change and the
   pinned hex is re-recorded. The shells build their storage key from
   `session_blob_version()`, so they follow without an edit. A practice in
   progress at the moment of updating is lost once, as with every bump.
2. **The bridge (typed bindings).** Both `ios/generated/` and
   `android/generated/` are regenerated, and every shell read of a moved
   field becomes `entry.plan.<field>`. A LiveBridge round trip on each app
   carries an entry with every plan field set.
3. **The saved session (JSON through `StoredEntry`).** Unchanged. The stored
   entry stays flat and maps to and from the nested type in
   `stored_session.rs`, so every session saved by an older build reads as
   before and no migration runs.

Sync's session body is `PracticeSession` as JSON, so its shape nests too.
Sync is not live on any device, so `SCHEMA_VERSION` stays at 1.

## Tests

- A stored session row written by today's build reads back to the same
  plan, field by field; dropping the mapping of any plan field fails it.
- The crash-recovery pin fails until re-pinned at version 9.
- `from_plan` gives an entry whose plan equals the input and whose played
  fields are empty: the "loading a routine is a copy" claim.
- LiveBridge round trip of a fully set plan on iOS and Android.
- The existing core, iOS and Android suites pass with no behaviour change.

## Out of scope

The routine and week types themselves, any sync record kind, any screen
change, live item names (#2583), and a store migration.

## After merge

#1348 and #2579 link here as the shape they build on.
