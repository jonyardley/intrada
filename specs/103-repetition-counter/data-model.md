# Data Model: Repetition Counter

**Branch**: `103-repetition-counter` | **Date**: 2026-02-21

## Entity Changes

### SetlistEntry (extended)

Three new optional fields added to the existing `SetlistEntry` struct:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `rep_target` | `Option<u8>` | `None` | Configured repetition target (3–10). `None` = counter not used for this entry. |
| `rep_count` | `Option<u8>` | `None` | Final consecutive correct count at save time. `None` = counter not used. |
| `rep_target_reached` | `Option<bool>` | `None` | Whether `rep_count >= rep_target` at save time. `None` = counter not used. |

All fields use `#[serde(default)]` for backward compatibility with existing serialised data.

### Validation Constants

| Constant | Type | Value | Description |
|----------|------|-------|-------------|
| `DEFAULT_REP_TARGET` | `u8` | `5` | Default repetition target when counter is enabled |
| `MIN_REP_TARGET` | `u8` | `3` | Minimum allowed repetition target |
| `MAX_REP_TARGET` | `u8` | `10` | Maximum allowed repetition target |

### Validation Rules

- `validate_rep_target(target: &Option<u8>) -> Result<(), LibraryError>`: If `Some(t)`, `t` must be in `MIN_REP_TARGET..=MAX_REP_TARGET`. Returns `LibraryError::Validation` on failure.

## State Transitions

### Counter Lifecycle Per Entry

```
None (no counter)
  │
  ├── [Add rep target in Building] ──→ Some(target=5, count=None)
  │
  ├── [Start Session] ──→ Some(target=N, count=0, reached=false)
  │
  ├── [Got it] ──→ count += 1 (capped at target)
  │     └── [count == target] ──→ reached=true, buttons hidden
  │
  ├── [Missed] ──→ count -= 1 (clamped to 0)
  │
  ├── [Disable mid-item] ──→ target=None, count=None, reached=None
  │
  ├── [Skip / Next / Finish] ──→ frozen (count/target/reached saved as-is)
  │
  └── [Save Session] ──→ persisted to DB
```

### Counter State by Session Phase

| Phase | `rep_target` | `rep_count` | `rep_target_reached` | Mutable? |
|-------|-------------|------------|---------------------|----------|
| Building | Set by user (3–10) or None | N/A | N/A | Target only |
| Active (pre-start) | Carried from building | 0 | false | Count via got-it/missed |
| Active (target reached) | Unchanged | = target | true | Frozen (buttons hidden) |
| Summary | Unchanged | Frozen | Frozen | Read-only |
| Saved | Unchanged | Frozen | Frozen | Read-only (persisted) |

## Database Schema

### Migrations

**Migration 0021**: `ALTER TABLE setlist_entries ADD COLUMN rep_target INTEGER;`
**Migration 0022**: `ALTER TABLE setlist_entries ADD COLUMN rep_count INTEGER;`
**Migration 0023**: `ALTER TABLE setlist_entries ADD COLUMN rep_target_reached INTEGER;`

### Updated Column List (setlist_entries)

After migrations, the full column list for SELECT/INSERT:

```
id, session_id, item_id, item_title, item_type, position, duration_secs, status, notes, score, intention, rep_target, rep_count, rep_target_reached
```

Positional indices for `row_to_entry()`:
- Index 11: `rep_target` (`Option<i64>` → `Option<u8>`)
- Index 12: `rep_count` (`Option<i64>` → `Option<u8>`)
- Index 13: `rep_target_reached` (`Option<i64>` → `Option<bool>`, where 1 = true)

## ViewModel Extensions

### SetlistEntryView (extended)

| Field | Type | Description |
|-------|------|-------------|
| `rep_target` | `Option<u8>` | Target for display ("Reps: 3/5") |
| `rep_count` | `Option<u8>` | Count for display |
| `rep_target_reached` | `Option<bool>` | For styling (✓ vs plain) |

### ActiveSessionView (extended)

| Field | Type | Description |
|-------|------|-------------|
| `current_rep_target` | `Option<u8>` | Current item's target (for RepCounter display) |
| `current_rep_count` | `Option<u8>` | Current item's count (for RepCounter display) |
| `current_rep_target_reached` | `Option<bool>` | For achievement state rendering |

## New Session Events

| Event | Fields | Phase | Description |
|-------|--------|-------|-------------|
| `RepGotIt` | (none) | Active | Increment current entry's `rep_count` by 1 (capped at target) |
| `RepMissed` | (none) | Active | Decrement current entry's `rep_count` by 1 (clamped to 0) |
| `SetRepTarget` | `entry_id: String, target: Option<u8>` | Building | Set or clear rep target for an entry |
| `EnableRepCounter` | (none) | Active | Enable counter on current item (set count=0 if target exists) |
| `DisableRepCounter` | (none) | Active | Disable counter on current item (clear count and reached) |

## API Contract Changes

### SaveSessionEntry (extended)

Three new optional fields added:

```
rep_target: Option<u8>     // #[serde(default)]
rep_count: Option<u8>      // #[serde(default)]
rep_target_reached: Option<bool>  // #[serde(default)]
```

No changes to API endpoints — the existing `POST /api/sessions` and `GET /api/sessions` endpoints carry the new fields transparently via the extended structs.
