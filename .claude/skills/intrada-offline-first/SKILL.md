---
name: intrada-offline-first
description: "The eight non-negotiable offline-first invariants (no network on the local-first path, sync-ready entities with updated_at/deleted_at, client-owned ulids, reconciliation in the core, no silent failed writes, dual-mode handlers, no account gate, GRDB vs crux_kv), their PR checklist, and the append-only on-device SQLite migration rules. MUST read before any change touching persistence, sync, a new domain entity, or the local database schema."
---

### Offline-first invariants (non-negotiable)

The native app is **offline-first**: on-device SQLite is the source of truth, the
app works with no network and no account, and sync is a future paid tier. Break
one of these and the app silently stops being offline.

1. **No network on the local-first path.** A local-first feature must work in
   airplane mode. New reads/writes go through the persistence `Effect`, never
   HTTP. *(Test-enforced: local-first launch + mutations assert zero `Http`.)*
2. **Every persisted entity is sync-ready from day one** — carries `updated_at`
   and a soft-delete `deleted_at` tombstone; **no hard deletes**, so the deferred
   sync engine never needs a migration. *(Test-enforced for the schema.)*
3. **Client-owned ids.** New entities mint their ulid locally as the canonical
   id — no server-assigned-id round-trip, and no temp-id dance, in local-first.
   (The temp-id default under *Other patterns* governs server-backed writes.)
4. **Reconciliation lives in the core**, not the Swift shell. Sync / LWW / merge
   logic is Rust (shareable to Android); the shell only executes typed storage
   ops. (The dumb-pipe rule, applied to persistence.)
5. **A failed local write is never a silent success.** Storage ops resolve a real
   failure output (`PersistenceOutput::Failed`) and the core surfaces it — never
   fake an `Ack` (#816).
6. **Existing dual-mode handlers stay intact; new code is local-first only.**
   Legacy domain handlers still branch on `local_first` — when touching one, keep
   both branches passing, since core tests still exercise the online path. New
   domain code targets local-first only: **the build-and-test-both-modes
   requirement is retired for new work** (see
   [`docs/rebuild-review.md`](docs/rebuild-review.md) §3).
7. **No account gate on core functionality.** Only sync (the paid tier) may
   require auth. The free app works fully signed-out.
8. **Relational data in the GRDB store; only small singletons in `crux_kv`**
   (settings, `session-in-progress` crash-recovery).

**PR checklist — any change touching persistence, sync, or a new domain entity:**

- [ ] New reads/writes go through the persistence `Effect`, not HTTP (1)
- [ ] New persisted table/columns have `updated_at` + `deleted_at`; no hard delete (2)
- [ ] New entities use a client-minted ulid as the canonical id (3)
- [ ] Any merge/reconciliation logic is in the core, not the shell (4)
- [ ] Write handlers branch on `local_first` (or use `save_or_put`) and a local
      failure resolves `Failed`, not `Ack` (5)
- [ ] Changes to an existing dual-mode handler keep both `local_first` branches
      tested; new code is local-first only (6)
- [ ] Data-model change: new migration appended (never edits a shipped one),
      additive where possible; core type + migration + codec updated together;
      ships an upgrade-path test (see below)

### Local data migrations

The on-device SQLite schema (GRDB, in `LibraryStore`) evolves via
`DatabaseMigrator`. Treat these with **more** care than the server migrations:
**on the free offline tier the device is the only copy of the user's data** — no
server backup, no DBA. A destructive or buggy migration that ships is
**unrecoverable** data loss, and you can't un-ship it.

- **Append-only, forward-only, ordered.** Add a new `registerMigration("vN_…")`;
  **never edit or delete a shipped migration** — it has already run on real
  devices, and users skip versions, so the chain must run cleanly from *any*
  past version.
- **Additive by default.** New nullable columns and new tables are safe.
  Drop/rename/retype is dangerous — use a copy-table migration, and prefer to
  **defer destructive changes until sync/backup exists**.
- **Evolve the core type + schema + codec together.** A new `Item` field needs,
  in one change: the Rust field (`Option` / `#[serde(default)]`), a migration
  adding the column with a default for existing rows, and the row↔`Item` codec.
- **Test the upgrade path, not just the end state.** Every migration ships with a
  test that a DB *populated at the previous version* migrates with data intact.

