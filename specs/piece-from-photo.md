# A photo on a piece

> Tier 3 spec. Issue [#1355]. Touches the FFI bridge contract and the on-device
> schema, so Tier 3 by the domain-sensitivity override rather than by size.
>
> **Phase A** — the photo itself: capture or pick one image, keep it on the
> device, show it on the item. No text recognition, no titles read off the page.
> Ships in two PRs: **core first** (this spec, `Item.photo_id`, the events, the
> `ViewModel` projection, migration v16, the Swift codec, tests), **screens
> second** (capture/pick on create and edit, the photo on the detail screen,
> snapshots + VoiceOver), with Claude Design between them.
>
> **Phase B** — reading the photo (extracting a title, composer, key from a
> score page) is out of scope here and is not designed yet. The spec is named
> for the arc, not for Phase A.

[#1355]: https://github.com/jonyardley/intrada/issues/1355

## Problem

You add a piece to the library while the music is in front of you. The thing
that identifies it, for you, is the page: the title block on the first system,
your teacher's notes in the margin, the photocopy in the folder. Typing
"Chopin — Nocturne in E flat, Op. 9 No. 2" reproduces some of that and loses the
rest, and the rest is what you actually recognise the piece by three weeks later.

So: let one photo ride along with the item as an aide-memoire. Not a document
library, not a score reader. One picture you took, on the item you took it for.

Photos existed once before in this codebase and were rolled back with the
lessons vertical (`docs/roadmap.md` §6): server-side, in R2, coupled to a
feature nobody kept. This is the opposite shape — on-device, no account, no
network, attached to something that already earns its keep.

## Scope

**In (Phase A):**

- Exactly one photo per item, piece or exercise.
- Attached at create, or attached/replaced/removed at edit.
- Stored on the device. Works with no network and no account.
- Shown on the item detail screen.

**Out (later iterations, named here only so the data shape does not fight them):**

- Multiple photos per item, ordering, captions.
- A photo management surface (a gallery, bulk delete, reuse across items).
- Versions/history of a photo.
- Text recognition and the create-from-photo flow (Phase B).
- Sync. The photo file is device-local; the sync engine does not exist yet and
  this does not build any part of it.

**Item, not piece.** The issue title says "create item", the body says "one
photo per item", and nothing about an aide-memoire is piece-shaped — an exercise
photographed from a technique book is the same need. No `ItemKind` restriction.
(The spec filename says "piece" because Phase B is piece-shaped.)

## Approach

### The core stores an id; the bytes never cross the bridge

`Item` gains one field:

```rust
/// The item's aide-memoire photo, if it has one. An opaque ULID the shell
/// resolves to a file on disk; the core never sees image bytes.
#[serde(default)]
pub photo_id: Option<String>,
```

The image itself is a file in the app container, written and read by the shell
at a path derived from the id. The core holds only the id.

This is the one genuinely load-bearing decision, so the reasoning, in full:

- **Bytes on the bincode bridge would be ruinous.** A phone photo is 2–5 MB.
  `Item` crosses the FFI bridge on every hydrate and every `ViewModel` render.
  Put a `Vec<u8>` in it and the whole library's photos are copied, encoded and
  decoded on every render pass.
- **Bytes in SQLite would be nearly as bad.** `loadItems()` does
  `SELECT * FROM item` and builds every `Item` eagerly. A BLOB column turns
  library hydration into a multi-megabyte read.
- **SwiftUI wants the file.** `AsyncImage`/`Image(contentsOfFile:)` load and
  downsample off the main thread from a URL. Handing SwiftUI a `[UInt8]` that
  arrived through the bridge throws that away.

So the shell owns the bytes and the filesystem layout; the core owns the id and
every decision about **when a photo starts and stops belonging to an item**.
That split keeps the dumb-pipe rule honest: the shell is doing I/O, not deciding
anything.

### Lifecycle, and who decides it

The shell mints a ULID, writes the image, and only then tells the core. The core
never has a half-attached photo, and a failed write simply never becomes an
event.

| Moment | Core does | Effect emitted |
|--------|-----------|----------------|
| Create with a photo | Stamps `photo_id` on the new item | `SaveItem` |
| Attach to an item with no photo | Sets `photo_id`, bumps `updated_at` | `SaveItem` |
| Attach over an existing photo | Replaces `photo_id` | `SaveItem` + `DeletePhoto(old)` |
| Remove | Clears `photo_id`, bumps `updated_at` | `SaveItem` + `DeletePhoto(old)` |
| Delete the item | Tombstones the item as today | `DeleteItem` + `DeletePhoto` |

The **core** emits the file deletions. The shell never decides a photo is
garbage, which is exactly the decision that would rot if it lived in Swift.

`DeletePhoto` is a new `PersistenceOperation` alongside the item and session
ops. It resolves `Ack`/`Failed` like the rest, so a store failure surfaces
rather than being assumed (invariant 5, #816).

**Item delete does remove the file** even though the row is only tombstoned.
There is no undelete surface, and the alternative is that every deleted piece
leaks a few megabytes forever. The tombstoned row keeps its `photo_id` so a
future sync still sees the last known state.

### Events

```rust
ItemEvent::AttachPhoto { item_id: String, photo_id: String }
ItemEvent::RemovePhoto { item_id: String }
```

plus `CreateItem.photo_id: Option<String>`, so a photo taken on the create form
lands in the same event as the rest of the form. It has to: the core mints the
item's ULID, so the shell has no id to send an `AttachPhoto` against — the same
reason `AddLinkedExercise` exists (#1431).

Photo changes are **not** folded into `UpdateItem`. Two paths to the same field
means two places the old-file deletion has to be right, and `UpdateItem`'s
three-state `Option<Option<T>>` encoding is the fiddliest thing on the bridge.
Tags already set the precedent: `AddTags`/`RemoveTags` sit beside `Update`.

### Validating the id

`photo_id` must parse as a ULID. This is not tidiness — the id becomes a path
component in the shell, so `../../…` in that string is a path traversal out of
the app container. The core is the only place that can refuse it before it is
written to the item, and validation is the single source of truth for rules of
this kind (`validation.rs`).

Local-first only. `AttachPhoto`/`RemovePhoto` against the online path set
`last_error` and change nothing, like `AddLinkedExercise` — photos are
device-local by construction and there is no server surface to write them to.

### View projection

`LibraryItemView.photo_id: Option<String>`, so the screens PR reads the
`ViewModel` rather than reaching back into core types. The core PR ships the
projection the screens will consume; a core PR that does not is a core PR the
screens PR has to reopen.

### Storage: schema

Migration **v16**, additive and nullable:

```sql
ALTER TABLE item ADD COLUMN photo_id TEXT
```

Existing rows get `NULL`, which is "no photo". Nothing is dropped, renamed or
retyped, so the migration is safe on the free offline tier where the device is
the only copy of the data.

The file lives at
`<Application Support>/photos/<photo_id>.jpg`, written by a `PhotoStore` the
Store resolves `DeletePhoto` against — a protocol, like `ItemStore`, so a test
can inject a failing fake and assert the core surfaces `Failed`.

## Key decisions

**1. A column on `item`, not a `photo` child table.** Iteration 1 is exactly one
photo, and at one-per-item the photo is an *attribute* of the item, not an
entity of its own — the same category as `composer` or `key`, which carry no
tombstone either. Offline-first invariant 2 is satisfied by the item's own
`updated_at`/`deleted_at`: removing a photo moves `item.updated_at`, and
item-granular LWW resolves it correctly when exactly one photo can exist.

The trade-off, stated plainly: this shape cannot express a per-photo tombstone,
so **multiple photos is a migration, not a column** — a `photo` child table
keyed by `item_id` with its own `updated_at`/`deleted_at`, exactly as `variant`
did for step ladders in #1083. That migration is cheap (additive table, backfill
one row per item that has a `photo_id`) and it is the right moment to pay for
the sync machinery, rather than carrying a table's worth of ceremony now for a
feature that is explicitly out of scope.

**2. The shell mints the photo ULID, not the core.** The shell needs an id
*before* it writes the file; getting one from the core first would be an extra
round trip through the bridge for no gain. Invariant 3 asks for client-minted
ids, and the shell is the client. The core validates what it receives.

**3. JPEG, re-encoded by the shell.** The shell downsizes and re-encodes to JPEG
before writing. HEIC from the camera is smaller but the source may be a
screenshot, a PNG from Files, or a shared image, and one format on disk means
one decode path. Quality and max dimension are shell constants, tuned in the
screens PR against real photos of real music.

**4. No `AppEffect` for the write.** The shell writes the file itself and then
sends the event, rather than the core commanding the write. The alternative
(stage the file, event, core mints an id, core commands a move) is a round trip
and a staging directory to buy a marginally tidier ownership story, and it does
not remove the failure mode it appears to fix — a crash between any two steps
still orphans a file. See the open question below.

**5. `ActiveSession` is untouched.** The crash-recovery blob's transitive graph
is `SetlistEntry`, not `Item`, so adding a field to `Item` does not invalidate
the positional-bincode snapshots already sitting in UserDefaults on devices, and
the key does not need bumping. (Confirmed by reading the graph, not assumed —
this is the trap that bit #1223, #1244 and #1256.)

## Open questions

**Orphan reclamation.** A crash between the shell's file write and the core
receiving `AttachPhoto` leaves a file no item references. It is a leak, not
corruption: nothing reads it, nothing shows it, and it costs a few megabytes at
worst. The fix is a prune sweep after hydration — the core emits
`PrunePhotos { keep: [ids] }` and the shell deletes everything else in the
directory. Deliberately not in Phase A: it needs the hydration path to be
provably complete before it is safe to delete on that signal, and getting that
wrong deletes a user's photo. Tracked as a follow-up.

**iCloud backup.** Application Support is backed up by default, so photos ride
along in an iCloud device backup. That is probably what a user wants and
definitely what they would assume. If library size becomes a complaint, the
directory can be excluded — not a Phase A decision.

## Phases

- **Phase A, core PR** (this spec): `Item.photo_id`, `CreateItem.photo_id`,
  `LibraryItemView.photo_id`, `AttachPhoto`/`RemovePhoto`,
  `PersistenceOperation::DeletePhoto`, validation, migration v16, the Swift
  codec and `PhotoStore`, round-trip and upgrade-path tests.
- **Phase A, design**: Claude Design for capture/pick and display, against the
  existing kit. Where the photo sits on the detail screen, what the create form
  shows before and after a photo is taken, how removal is confirmed.
- **Phase A, screens PR**: the SwiftUI, its snapshots, VoiceOver labels, iPad.
- **Phase B**: recognition. Not designed. Separate issue when Phase A has shipped
  and there is a real photo on a real item to read.

## Testing

- Round-trip (`assert_round_trips`) on `AttachPhoto`, `RemovePhoto`, and an
  `Item` carrying a `photo_id` — a bridge-crossing write payload gets a real
  round-trip before it is wired to a screen (#846).
- Attach over an existing photo emits `DeletePhoto` for the **old** id, and
  deleting an item with a photo emits `DeletePhoto`. These are the assertions
  that fail if the lifecycle drifts into Swift.
- A non-ULID `photo_id` is rejected and stores nothing.
- Online mode refuses both events and leaves the item unchanged.
- Migration upgrade path: a database populated at v15 migrates to v16 with the
  items intact and `photo_id` NULL.
- A `PhotoStore` that throws resolves `Failed`, not `Ack`.
