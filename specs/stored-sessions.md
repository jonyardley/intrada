# Saved sessions are read and written by the core

> Tier 3 spec (bridge surface). Issue: #2234, under the Android epic #2220.
> Plan: the issue's plan comment.

## Problem

The iOS database code decides what an old session means. A row written before
plays existed (#1739) holds one score, one rep count and one tempo on the entry
itself; the Swift codec folds those into a single play. A status, item kind or
rep action it cannot read becomes a conservative default (#949). The JSON field
names of the entries column live only in Swift.

Android is about to store sessions. Copying those rules into Kotlin would give
two versions that drift, and a session saved on one would read differently on
the other.

## Approach

The core owns a session's stored form, as it already owns a piece's stored key
(`key_from_stored`, #2106). Two plain FFI calls, outside the Crux effect loop:

- `session_from_stored(row)`: the row's columns in, the session out, plus the
  list of values it could not read and replaced.
- `session_to_stored(session)`: the session in, the columns out, the entries as
  JSON.

The shell copies columns in and out of SQL and logs what the core lists. It
keeps the SQL, the migrations and `updated_at`, which are storage, not meaning.

## Rules the core takes over

| Stored value | Read as |
|---|---|
| An entry with no `plays`, completed or carrying a mark or banked reps | One play from the entry's fields, started at the session start, id `<entry>-play` |
| An entry with no `plays`, otherwise | No play |
| An entry with `plays` and stale entry-level marks | The plays only |
| A rep tap stored as a bare action string | That action at the session start |
| An unknown entry status | Not attempted, listed |
| An unknown rep action | Missed, listed |
| An unknown item kind | Piece, listed |
| An unknown completion status | Completed, listed |
| An unknown key modality | No mode, listed |
| A time inside the entries that does not parse | The session start, listed |
| An entries column that does not parse | No entries, listed |
| An out-of-range score or capture version | Clamped, as Swift did |
| A negative duration | 0, listed (Swift crashed) |
| A start or end time that does not parse | The row is refused; the shell skips and reports it |

The last row is new. Before, the string crossed the bridge and the core's
deserialiser failed the whole load.

## Decisions

- **The whole row, not only the entries column.** The completion status and
  the clamps are rules too; taking the row is what leaves the shell mapping
  columns only. Rejected: the entries column alone.
- **Writing moves as well as reading**, so the field names have one owner.
- **Plain calls, not a new persistence output.** A raw-row variant on
  `PersistenceOutput` changes the Crux effect contract and still leaves each
  shell parsing the JSON.
- **Unknown values are listed, not hidden** (#949): the shell logs each, as the
  Swift codec did.
- **Nothing on device is rewritten** (#1739 decision 8). A row is upgraded only
  when the session is saved again.
- **The JSON shape does not change.** Same camelCase names, absent rather than
  null for an empty optional, so a build from before this change still reads a
  row written after it.

## Releases

1. Core PR: `crates/intrada-core/src/stored_session.rs`, the two calls in
   `crates/intrada-ffi/src/ffi.rs`, this spec. The iOS app does not call them yet.
2. iOS PR, the same session: `ios/Intrada/Core/SessionCodec.swift` maps columns
   only; the Swift fold, field names and status defaults are deleted.

## Verification

- A Rust table test over every row in the table above: the legacy and
  unknown-value rows are the ones the iOS tests pin as shipped (#1256); the
  bad times and clamps are new cases. A current entry is written and read back.
- Deleting the fold, or the not-attempted default, fails a named case.
- iOS: `LegacyEntryPlaysTests` and the store tests pass unchanged in PR 2.

## Out of scope

Item columns, the crash-recovery blob, rewriting rows on device, and Android
storing sessions (later in #2220).
