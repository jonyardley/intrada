# Native iOS practice player

> Status: Phase A in progress. Last updated 2026-06-04.

## Problem

The native iOS builder (#936) assembles a setlist but "Start session" dead-ends.
The player closes the **Plan → Practise → Track** loop: build → practise each
item → finish → a session lands in the Practice history. The core's
`SessionStatus` machine (`Building → Active → Summary → Idle`) already drives all
of this; the gap is (1) completed sessions don't persist on-device, and (2) there
are no Active/Summary screens.

The app is offline-first: a finished session **must** survive a restart with no
network. Today it doesn't — `SaveSession` only POSTs to the API, and `StartApp`
in local-first mode loads items only (`app.rs` comment: "Sessions/sets aren't on
the native shell yet"). So the player needs a persistence foundation *before* the
UI is worth shipping.

## Approach — phased

- **Phase A — session persistence (this spec / #948).** Make completed sessions
  persist locally and reload on launch. No UI; pure core + GRDB. The foundation
  everything else sits on.
- **Phase B — player spine (#932).** Pre-start review → focus screen (current
  item, next/skip) → finish → Summary, wired to Phase A. In-memory state is the
  core's; finishing persists. Flow designed in Pencil first.
- **Phase C — layers.** Timer, rep counter / scoring, the interstitial reflection
  between items, resume/discard of a crash-recovered session.

Phases B/C carry the interaction design (the "spend friction deliberately" split:
a calm focus screen, with scoring/reps in the between-item interstitial rather
than on top of the timer). Those are settled in a Pencil pass before Phase B.

## Phase A — design

Mirror the **Items** local-first pipeline exactly (`domain/item.rs` +
`persistence.rs` + `LibraryStore.swift`), so sessions follow the proven path.

**Core (`intrada-core`):**
- `PersistenceOperation` gains `LoadSessions` and `SaveSession(PracticeSession)`;
  `PersistenceOutput` gains `Sessions(Vec<PracticeSession>)`.
- `persistence::load_sessions()` / `save_session()` helpers (→ `Event::Sessions…`).
- `SaveSession` handler branches on `model.local_first`: local → `save_session`
  (+ `record_success`, keep the `ClearSessionInProgress` notify); online →
  `http::create_session` (unchanged). The optimistic `model.sessions.push` stays
  in both.
- `StartApp { local_first: true }` issues `load_items()` **and** `load_sessions()`.
- A `SessionsStoreLoaded` handler sets `model.sessions` from the output (mirrors
  `StoreLoaded` for items); rebuilds `practice_summaries`.

**iOS (GRDB):**
- New `v3_session` migration: a `session` table keyed by `id`, top-level fields as
  columns (`started_at`, `completed_at`, `total_duration_secs`,
  `completion_status`, `session_notes`, `session_intention`), the nested
  `entries` as a JSON `TEXT` column, plus sync columns `updated_at` +
  `deleted_at` (invariant 2). `updated_at` is stamped to `completed_at` on save
  (sessions are immutable once completed).
- `entries` JSON via a Swift `Codable` storage DTO mirroring `SetlistEntry`
  (+ `RepAction`). **Not** a bincode blob: bincode is positional, so any future
  field change would fail to decode old rows — unacceptable when the device is
  the only copy of the data. JSON + optional fields evolves additively.
- Store gains `loadSessions()` / `saveSession(_:)`; `Store.persistenceOutput(for:)`
  handles the new ops.

### Key decisions

- **JSON column for entries, not a junction table or bincode blob.** Sessions are
  read/written whole (history display), so a relational `session_entry` table
  (with its own nested `rep_history`) is over-normalisation; the Items table's
  `tags`-as-JSON sets the denormalise-collections-as-JSON precedent. JSON over
  bincode for evolution safety.
- **No `updated_at` on the `PracticeSession` domain type.** The sync column lives
  on the table only; a completed session is immutable, so `updated_at` =
  `completed_at`. Avoids a domain-type/ViewModel/API change for no behavioural gain.
- **`DeleteSession` persistence deferred.** No native delete-session UI yet; the
  table ships `deleted_at` so it's sync-ready, but the soft-delete op lands with
  the UI that needs it (tracked separately).

### Offline-first invariant check (Phase A)

1. No-network local path — `SaveSession`/load go through the persistence Effect, not HTTP. ✓
2. Sync-ready schema — `updated_at` + `deleted_at`; no hard delete. ✓
3. Client-owned id — the session's ulid is canonical (no temp-id). ✓
4. Reconciliation in the core — the `local_first` branch + load handler are Rust. ✓
5. Failed write surfaces — reuse `StoreWritten`/`Failed` handling (no fake Ack). ✓
6. Both modes — `SaveSession` + `StartApp` tested local-first **and** online. ✓
7. No account gate. ✓
8. Relational data in GRDB (not `crux_kv`). ✓

## Testing (Phase A)

- Core (TDD): `SaveSession` in `local_first` emits the persistence save (not HTTP)
  and keeps the optimistic `model.sessions`; `StartApp { local_first }` loads
  sessions; online mode still POSTs. Both modes.
- GRDB: a migration **upgrade-path** test (populate at v2, migrate to v3, items
  intact) and a session **round-trip** (save → load → equal, entries preserved).
