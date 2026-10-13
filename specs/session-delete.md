# Delete a saved session, and give it a change time

> Tier 3, sensitive surfaces (the store and a bridge change that retypes a
> field). Issue #2581, decision 1 of the data model review (#2577, ruled
> 2026-10-13). One PR across the stack: core and store commits first,
> reviewed alone, then the iPhone and Android screens. No
> migration: the `session` table has carried `updated_at` and `deleted_at`
> since v3.

## Problem

A saved session cannot change. The store writes `updated_at` as the session's
`completed_at`, a save clears `deleted_at`, and a sync record for a session is
always live with `completed_at` as its change time. That holds only while
nothing edits or removes a session, and the data model review (#2577) expects testers to want to remove one left
running overnight. Once sync ships, a delete on the iPhone has to reach the
iPad and stay there.

## What changes

A saved session gains the two facts every other synced record has: when it
last changed, and when it was deleted. They sit beside the session, as
`StoredItem` holds a piece's tombstone, not inside `PracticeSession`:

```rust
pub struct StoredPracticeSession {
    pub session: PracticeSession,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
```

Keeping them out of `PracticeSession` leaves the sync body, the session's
JSON, exactly as it is: no sync schema bump, no optional field with a meaning
for its absence, and none of the places that build a session change. The
crash-recovery blob does not hold a `PracticeSession`, so it is untouched.

## The store

| Operation | What it writes |
| --- | --- |
| `SaveSession` | A new row: `updated_at = completed_at`, not deleted. An existing row: its content, with `updated_at` and `deleted_at` left as they were, so a later save never brings a deleted session back |
| `DeleteSession { id, deleted_at }` | `deleted_at` on that row, as `DeleteItem` does for a piece |
| `RestoreSession { id, restored_at }` | `deleted_at` cleared and `updated_at = restored_at`, so the undo is the later change |
| `ApplyMerged` | Each winning session whole, `updated_at` and `deleted_at` written as given |

`LoadSessions` already leaves out deleted rows, so history, Progress and the
practice summaries lose a deleted session with no other change.
`LoadRecords` and `LoadAllRecords` return `StoredPracticeSession`, tombstones
included, for the merge. Times are never compared in SQL: the stored text
varies in its fractional digits, so text order is not time order.

## Sync

A session's record now carries its change time, the later of `updated_at`
and `deleted_at`, and its `deleted_at`, exactly as a piece's does. `decide`
already ranks a later change first and a delete over an edit at the same
instant, so nothing in the merge rule changes.

The body holds no `updated_at`, so a session taken from a record is stored
with the record's change time as its `updated_at`. For a live session that is
exact. For a deleted one it is the delete time, which is never earlier than
the true value and gives the same record when it is sent on.

No device has the sync engine yet (#2355 is open), so no older app can meet a
deleted session's record and miss its delete.

## The core

Four events, each appended after the last variant of its enum:

- `SessionEvent::DeleteSaved { session_id, now }` takes the session off the
  list and rebuilds the practice summaries at once, keeps it for undo, writes
  `DeleteSession`, and uploads the deleted record once the store has it.
- `SessionEvent::UndoDelete { now }` puts the last deleted session back, writes
  `RestoreSession`, and uploads the live record with `now` as its change time.
- `Event::SessionDeleteWritten(PersistenceOutput)` answers both writes. Its own
  event, not `SessionStoreWritten`, whose `Ack` closes a save in flight. A
  refused write raises the storage error and reloads the list, which rolls the
  change back (#825).
- `SessionEvent::DismissUndoDelete` closes the undo banner and lets the
  delete stand.

Only the last delete can be undone, which is all the screen's undo needs.
The view says so in `undo_delete`, appended last to `ViewModel`: "Session
deleted" while there is a delete to take back, and `None` once it is
undone, dismissed, taken over by the next delete, or put back by the reload
after a refused write. That reload forgets the delete, so a later delete of
the same session from another device does not bring the banner back.

## Tests, first

- Store: a deleted session stays deleted after a later `SaveSession` of the
  same id; a restore clears the delete and moves `updated_at`; a merge writes
  both times as given; a merge load returns tombstones with their times.
- Sync: a deleted session's record carries its delete and the later change
  time; a restore after a delete wins over the delete on another device.
- Core: delete takes the session out of the list and the summaries and sends
  `DeleteSession` with `now`; undo puts it back and sends `RestoreSession`; a
  refused delete reloads the list.
- Bridge: the new operations and events round trip through the real bridge on
  iOS (`LiveBridge`) and Android (`BridgeRoundTripTest`).

## Phases

1. This spec, then the store and core, test first, paired with `reviewer` on
   the core commits alone.
2. The screens on iPhone and Android at parity, with an undo, on the layout
   Jon picks from the mock-up. Same branch and PR.

A save always uploads a live record stamped `completed_at`, so a re-save of a
deleted id would win elsewhere; nothing re-saves one today.

## Out of scope

Editing a saved session's content. The store keeps `updated_at` on a re-save
so a later edit feature decides how it moves, not this one.
