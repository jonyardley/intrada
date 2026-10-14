---
paths:
  - "crates/intrada-core/src/**"
  - "ios/Intrada/Core/**"
  - "crates/intrada-store/**"
  - "ios/IntradaTests/SharedItemStore*.swift"
---

# Offline-first invariants

On-device SQLite is the source of truth: the app works with no network and no
account, and sync is free through the user's own iCloud (#2353). Without iCloud
the device is the only copy. Break one of these and the app silently stops
being offline.

1. **No network on the local-first path.** Reads and writes go through the
   persistence `Effect`.
2. **Every persisted entity is sync-ready**: `updated_at` and a soft-delete
   `deleted_at` tombstone, no hard deletes. Test-enforced for the schema.
3. **Client-owned ids.** New entities mint their ulid locally as the canonical
   id.
4. **Reconciliation lives in the core.** Sync, LWW and merge logic is Rust; the
   shell executes typed storage ops.
5. **A failed local write is never a silent success.** Storage ops resolve
   `PersistenceOutput::Failed` and the core surfaces it; never fake an `Ack`
   (#816).
6. **A single write path**, not two branches to keep in sync.
7. **No account gate on core functionality.** Only sync may require auth.
8. **Relational data in the shared store (`crates/intrada-store`); only small
   singletons in UserDefaults** (library sort, profile, practice defaults,
   first-run flag, crash-recovery blob), written through an `AppEffect`. Each
   blob takes a versioned key and a Rust wire pin, so a shape change fails a
   test instead of silently failing to decode (#1952).

## Local data migrations (`crates/intrada-store/src/migrations.rs`)

A bad migration that ships is unrecoverable data loss.

- **Append-only, forward-only, ordered.** Append a new `("vN_…", …)` to
  `MIGRATIONS`; never edit or delete a shipped one. Users skip versions, so the
  chain runs cleanly from any past version.
- **Additive by default.** Nullable columns and new tables are safe. Drop,
  rename or retype through a copy-table migration, and defer destructive
  changes until backup exists.
- **Core type, schema and codec change together**: the Rust field (`Option` or
  `#[serde(default)]`), the migration with a default for existing rows, and the
  row-to-`Item` codec.
- **Test the upgrade path.** Every migration ships a test that a database at
  the previous version migrates with data intact.

## PR checklist for persistence, sync or a new entity

Invariants 1 to 5 hold, and any migration follows the rules above.
