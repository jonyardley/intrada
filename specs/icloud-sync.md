# iCloud sync: the core decides which copy wins

> Tier 3, sensitive surface (the bridge). Issue #2354, step 1 of epic #2353.
> Core first; the iOS sync engine (#2355) follows. No store migration. The
> effect and events the shell fulfils are provisional until the two-device
> trial (#2361) reports.

## Problem

iPhone and iPad sync through the musician's own iCloud, free, with no account
(#2353). When two devices change the same piece while apart, both must keep
the same copy. The merge lands first; the iOS engine only carries records.

## What syncs

One record per thing the store already writes as one unit:

| Kind | Body | Change time |
| --- | --- | --- |
| Piece or exercise | `Item`, sections and exercise links inside | later of `updated_at` and `deleted_at` |
| Variation | `Variation` | later of `updated_at` and `deleted_at` |
| Saved session | `PracticeSession`, entries, plays and taps inside | `completed_at` |

Variations are their own record because one shared variation is named by many
pieces (#2246).

A whole piece wins or loses together: a section edit on the iPad and a title
edit on the iPhone, both offline, keep whichever was later. A section or
exercise link only the losing device has is soft deleted, stamped with the
winner's change time.

Not synced here: settings (#2357) and photo bytes (#2358).

## The record

```rust
pub struct SyncRecord {
    pub kind: RecordKind,          // Item, Variation, Session
    pub id: String,
    pub schema_version: u32,       // sync::SCHEMA_VERSION when written
    pub changed_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub body: Vec<u8>,             // JSON of the core type
}
```

The body is JSON, because a newer app must read an older device's records and
positional bincode cannot decode a shape with a field added (#2234). A field
added to a synced type takes `#[serde(default)]`, never `skip_serializing_if`
(#846).

A test pins each kind's body; a field added, renamed or removed fails it:
bump `SCHEMA_VERSION`, then re-pin.

## The merge

`sync::decide(local, arrived)` is pure, and returns keep, take or park:

1. **Newer app.** `arrived.schema_version > SCHEMA_VERSION`: park it, untouched.
2. **Nothing here.** No local copy, tombstones included: take it.
3. **Later wins.** The later `changed_at` wins.
4. **Exact tie.** A delete beats an edit; otherwise the record whose body
   sorts lower, byte by byte, wins. Identical records keep the local one.

Steps 3 and 4 put every pair in one order, so two devices end with the same
copy. A delete is soft and syncs its last body, so a stale edit loses to it.

**Built-in variations** are seeded with a fixed early `updated_at`, so a new
iPad never overwrites a rename made on the iPhone. Clocks are trusted.

## The bridge

All provisional until #2361 reports.

```rust
pub enum SyncOperation {           // Effect::Sync, fire and forget
    Upload(Vec<SyncRecord>),
    Park(Vec<SyncRecord>),
    Unpark(Vec<RecordKey>),
    Settled,
}

pub enum SyncEvent {               // Event::Sync
    RecordsArrived(Vec<SyncRecord>),
    UploadEverything,
    AccountChanged,
    // internal: the stored copies loaded, the merge written
}
```

New persistence operations, run by `intrada-store`, so neither shell changes:

- `LoadRecords(Vec<RecordKey>)`: the stored copies, tombstones included, and
  in `StoredRecords::unreadable` the named rows this app cannot read (a piece
  counts when one of its sections or links will not read).
- `LoadAllRecords`: every item, variation and session, tombstones included.
- `ApplyMerged(StoredRecords)`: the winners, in one transaction, writing
  `deleted_at` exactly as given.

## The flow

**A local save** uploads the record only once the store acknowledges the
write. A parked id is never uploaded, and nothing uploads while paused.

**Records arrive.** Newer-app records go straight to `Park`. The core loads the
stored copies of the rest, decides each, writes the winners in one
`ApplyMerged`, and reloads the lists it changed. A body that will not decode,
or an arrival whose stored row this app cannot read, is parked.

One merge runs at a time; a batch arriving mid-merge waits. A local save on a
touched list, sent after the stored copies were asked for, makes the core ask
again. A parked id is unparked by an arrival as late as the parked copy or
later.

**The change token (#2355).** The core sends `Settled` when nothing is waiting
and every batch is stored. The shell advances its change token only then, so a
failed batch comes back on the next fetch.

**Unreadable rows** are never uploaded until a later load reads them:
uploading the readable part of a piece would delete the unread part on the
other device.

`UploadEverything` is a device's first sync. An account change pauses uploads
(#2356). At launch the shell sends parked records back as `RecordsArrived`
first.

## Android

Android gets no sync for now. Its bindings gain the new shapes and the Kotlin
shell ignores `Effect::Sync`; the store operations already run there because
they live in `intrada-store`.

## Tests

- Two-device cases against `decide`, merged in both orders.
- Each kind's body pinned; an older body still decodes.
- `LiveBridge` for the new shapes (#846), and Android's `BridgeRoundTripTest`.

## Out of scope

The CloudKit engine, outbox, parked table and Settings switch (#2355);
switching Apple ID beyond pausing (#2356); settings (#2357); photos (#2358).
