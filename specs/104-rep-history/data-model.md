# Data Model: Rep History Tracking

**Feature**: 104-rep-history
**Date**: 2026-02-21

## Entity Changes

### New Type: RepAction

A single action in the rep history sequence, stored as a signed integer representing the delta applied to the count. Decoupled from UI labels.

| Variant  | Numeric value | Delta meaning | Description                              |
|----------|--------------|---------------|-------------------------------------------|
| Missed   | `-1`         | count − 1     | Failed rep (UI may label "Missed", "Drop", etc.) |
| Success  | `1`          | count + 1     | Successful rep (UI may label "Got it", "Hit", etc.) |

- Defined in `intrada-core/src/domain/session.rs`
- Derives: `Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq`
- Uses `serde_repr` for integer serialisation (`#[repr(i8)]`)
- Values are deltas: sum the array for net progress, running total for sparkline charts
- Extensible: future values (e.g. `0` = neutral/partial) can be added without breaking existing data

### Modified: SetlistEntry

Add one new field:

| Field        | Type                     | Default | Description                                   |
|--------------|--------------------------|---------|-----------------------------------------------|
| rep_history  | `Option<Vec<RepAction>>` | `None`  | Ordered sequence of Got it / Missed actions    |

- `None` = no counter was active (same pattern as existing rep fields)
- `Some(vec![])` = counter was enabled but no actions taken
- `Some(vec![Success, Missed, Success, ...])` = full action sequence

Existing rep fields unchanged:
- `rep_target: Option<u8>` — target count (3–10)
- `rep_count: Option<u8>` — current consecutive count
- `rep_target_reached: Option<bool>` — whether count ≥ target

### Modified: SetlistEntryView

Add one new field to mirror the domain type:

| Field        | Type                     | Description                         |
|--------------|--------------------------|-------------------------------------|
| rep_history  | `Option<Vec<RepAction>>` | Same as SetlistEntry, passed to UI  |

### Modified: ActiveSessionView

Add one new field for the current item's history:

| Field                    | Type                     | Description                         |
|--------------------------|--------------------------|-------------------------------------|
| current_rep_history      | `Option<Vec<RepAction>>` | History for the current active item |

### Modified: SaveSessionEntry (API)

Add one new field:

| Field        | Type                     | Serde     | Description                         |
|--------------|--------------------------|-----------|-------------------------------------|
| rep_history  | `Option<Vec<RepAction>>` | `default` | JSON-serialised action sequence     |

## Database Changes

### Migration: Add rep_history column

```sql
ALTER TABLE setlist_entries ADD COLUMN rep_history TEXT;
```

- Stored as JSON TEXT: `null` or `[1,1,-1,1,1,1,1,1]`
- Nullable — null means no counter was active
- No index needed (never queried within the column)

### Updated SQL

**INSERT** adds `rep_history` as a new positional parameter:
```sql
INSERT INTO setlist_entries (id, session_id, item_id, item_title, item_type, position, duration_secs, status, notes, score, intention, rep_target, rep_count, rep_target_reached, rep_history)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
```

**SELECT** adds `rep_history` to the column list:
```sql
SELECT id, session_id, item_id, item_title, item_type, position, duration_secs, status, notes, score, intention, rep_target, rep_count, rep_target_reached, rep_history
FROM setlist_entries WHERE session_id = ?1 ORDER BY position ASC
```

### Row Parsing

Column index 14: `rep_history` — read as `Option<String>`, deserialised via `serde_json::from_str::<Vec<RepAction>>()` to `Option<Vec<RepAction>>`.

## State Transitions

### RepGotIt Event

```
Pre:  rep_history = Some([...existing...])
Post: rep_history = Some([...existing..., Success])
      rep_count += 1 (capped at target)
      rep_target_reached = true (if count >= target)
```

### RepMissed Event

```
Pre:  rep_history = Some([...existing...])
Post: rep_history = Some([...existing..., Missed])
      rep_count = (count - 1).max(0)
```

### EnableRepCounter → InitRepCounter (renamed)

Only fires when `rep_target.is_none()` (first enable):
```
Pre:  rep_target = None, rep_count = None, rep_history = None
Post: rep_target = Some(5), rep_count = Some(0), rep_target_reached = Some(false), rep_history = Some([])
```

When `rep_target.is_some()` (re-enable after hide): No domain event needed. Leptos signal toggles visibility.

### DisableRepCounter → Removed

No domain event. Leptos signal hides the counter UI. All rep state preserved on the entry.

### freeze_rep_state (updated)

No change needed — `freeze_rep_state` only sets `rep_target_reached`. The history is already frozen by virtue of not being modified (no more RepGotIt/RepMissed events fire after transition). No special handling required.

## Validation Rules (API)

- If `rep_target` is `None`, `rep_history` MUST be `None`
- If `rep_target` is `Some`, `rep_history` MAY be `None` (backward compat) or `Some(Vec)`
- No maximum length enforced on history (naturally bounded by session duration)
