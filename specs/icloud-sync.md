# iCloud sync: the core decides which copy wins

> Tier 3, sensitive surface (the bridge). Issue #2354, step 1 of epic #2353,
> milestone v0.19.0. Core first; the iOS sync engine (#2355) follows. No store
> migration. The effect and events the shell fulfils are provisional until the
> two-device trial (#2361) reports.

## Problem

Each device is its own notebook. #2353 decided on 3 October 2026 that iPhone
and iPad sync through the musician's own iCloud, free, with no account. When
two devices change the same piece while apart, something has to pick the copy
both keep, and both devices must pick the same one. Nothing in the core merges
two devices' copies today: `reconcile_sections` merges one edit into one
device's list.

The merge is the riskiest part of sync, so it lands first, on its own, with
the iOS engine only carrying records to and from iCloud.

## What syncs

One record per thing the store already writes as one unit:

| Kind | Body | Change time |
| --- | --- | --- |
| Piece or exercise | `Item`, sections and exercise links inside | later of `updated_at` and `deleted_at` |
| Variation | `Variation` | later of `updated_at` and `deleted_at` |
| Saved session | `PracticeSession`, entries, plays and taps inside | `completed_at` |

Variations are their own record because they left the item in v0.16 (#2246):
one shared variation is named by many pieces. Folding it into each piece would
copy it into every one.

A whole piece wins or loses together. A section edit on the iPad and a title
edit on the iPhone, both offline, keep whichever was later. Merging section by
section would add record kinds before anyone has hit the case. A section or
exercise link only the losing device has is deleted (soft, stamped with the
winner's change time), so the losing device ends with the winner's piece.

Sessions are never edited or deleted today, so two copies of one session never
differ. The record carries `deleted_at` anyway, for when they can be.

Not synced here: the library sort, profile and practice defaults (#2357), piece
photos (#2358). A piece's `photo_id` travels with it; the photo does not yet.

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

The body is JSON, not bincode. A device on the newer app must read an older
device's records, and positional bincode cannot decode a shape with a field
added (the reason stored sessions are JSON, #2234). A field added to a synced
type takes `#[serde(default)]`, never `skip_serializing_if` (#846).

`SCHEMA_VERSION` starts at 1. A test pins each kind's body for a fixture with
every optional field set and every list holding one, down to a play's taps. A
field added, renamed or removed anywhere in a synced type fails it, and the fix
is to bump `SCHEMA_VERSION`, then re-pin. A new enum variant does not fail it.

## The merge

`sync::decide(local, arrived)` is pure, and returns keep, take or park:

1. **Newer app.** `arrived.schema_version > SCHEMA_VERSION`: park it, untouched.
2. **Nothing here.** No local copy, tombstones included: take it.
3. **Later wins.** The later `changed_at` wins.
4. **Exact tie.** A delete beats an edit; otherwise the record whose body
   sorts lower, byte by byte, wins. Identical records keep the local one.

Steps 3 and 4 put every pair of records in one order, so two devices merging
the same records, in either order, end with the same copy. Jon chose the body
tie-break over the lower device id on 2026-10-09: it needs no stored writer id
and so no migration, and exact ties are rare.

A delete is soft. A deleted piece syncs with `deleted_at` set and its last
body, so a stale edit from a device that was off for a week loses to it.

**Built-in variations** (Hands separately and the rest) are seeded with a
fixed early `updated_at`, not the seeding time. Seeded at "now", a new iPad
would overwrite a rename made on the iPhone yesterday. For #2355: a fresh
device's upload of those seeded rows, stamped at the epoch, goes through the
same conflict path as any upload and never overwrites a newer iCloud copy.

Clocks are trusted. A device whose clock is wrong can win when it should not;
single user, accepted.

## The bridge

All provisional until #2361 reports.

```rust
pub enum SyncOperation {           // Effect::Sync, fire and forget
    Upload(Vec<SyncRecord>),
    Park(Vec<SyncRecord>),
    Unpark(Vec<RecordKey>),
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
  `deleted_at` exactly as given (the ordinary saves clear it).

## The flow

**A local save.** `SaveItem`, `SaveItems`, `DeleteItem`, `SaveVariations` and
`SaveSession` ask the shell to upload the record only once the store
acknowledges the write. A refused write uploads nothing. A parked id is never
uploaded, and nothing uploads while sync is paused.

**Records arrive.** Newer-app records go straight to `Park`. The core loads the
stored copies of the rest, decides each, writes the winners in one
`ApplyMerged`, and reloads the lists it changed. A failed merge write shows the
storage error, like any refused write. A body that will not decode at a version
the app knows is parked too, so it is never uploaded over, and so is an arrival
whose stored row this app cannot read, so the merge never writes over it.

One merge runs at a time. A batch arriving mid-merge waits in the core and
starts when the merge's write lands, acknowledged or refused, or when the merge
ends with nothing to write. If a local save on a list the batch touches is sent
after the stored copies were asked for, the core asks for them again rather
than decide on a copy the save may have changed.

A parked id is unparked by an arrival as late as the parked copy or later:
taken, once the merge write is acknowledged; kept, at once, with the local copy
uploaded, since it was held back while parked. An earlier arrival is decided
as usual and the id stays parked.

**The change token (#2355).** The shell must not advance its iCloud change
token past a batch until that batch's merge write comes back acknowledged. The
core does not retry a refused merge; fetching the batch again is how it is
retried.

**First upload.** `UploadEverything` uploads every record, tombstones
included, for #2355's first sync on a device.

**An account change** pauses uploads. What happens next is #2356's.

**Parked records** live in the shell's parked table (#2355). At launch the
shell sends them back as `RecordsArrived` before anything else, so the core
knows which ids not to upload over, and an updated app takes them.

## Android

Android gets no sync for now. Its bindings gain the new shapes and the Kotlin
shell ignores `Effect::Sync`; the store operations already run there because
they live in `intrada-store`.

## Tests

- A table of two-device cases against `decide`: edits on both while offline,
  a delete against a stale edit, a stale delete against a newer edit, an exact
  tie each way, a delete tying an edit, a record from a newer app, a record
  with nothing here.
- Convergence: every pair from the table, merged in both orders, ends the same.
- Each kind's body pinned against a fixture; an older body still decodes.
- The core flow: an acknowledged save uploads, a refused one does not; an
  arrived record loads, writes and reloads; a parked id is not uploaded.
- The store: `ApplyMerged` keeps a tombstone, and an arrived tombstone for a
  piece this device never had is still stored; a winning piece deletes the
  sections and links only the losing device had; `LoadRecords` names the rows
  it could not read.
- Races: a save sent while the stored copy loads makes the merge load again; a
  batch arriving mid-merge waits and merges after.
- `LiveBridge` for the new event and effect shapes (#846), and Android's
  `BridgeRoundTripTest`.

## Out of scope

The CloudKit engine, outbox, parked table and Settings switch (#2355);
switching Apple ID beyond pausing (#2356); singletons (#2357); photos (#2358).
