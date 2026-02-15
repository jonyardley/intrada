# Data Model: Practice Sessions

**Feature**: 012-practice-sessions | **Date**: 2026-02-15

## Entities

### Session

A single record of practice activity against a library item.

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `id` | `String` (ULID) | Yes | Unique, generated | Unique session identifier |
| `item_id` | `String` (ULID) | Yes | References Piece or Exercise | The library item practised |
| `duration_minutes` | `u32` | Yes | 1–1440 | Duration in whole minutes |
| `started_at` | `DateTime<Utc>` | Yes | Must be ≤ `logged_at` | When practice began |
| `logged_at` | `DateTime<Utc>` | Yes | Auto-set on creation | When the session was saved |
| `notes` | `Option<String>` | No | Max 5000 chars | Free-text notes about the session |

**Identity**: ULID (globally unique, time-sortable). Same crate (`ulid`) as Piece/Exercise IDs.

**Lifecycle**:
- Created via `SessionEvent::Log` (manual entry) or timer stop (web only)
- For manual logs, `started_at` defaults to `logged_at - duration_minutes`
- Editable: both `duration_minutes` and `notes` can be updated after creation via `SessionEvent::Update`
- Deletable via `SessionEvent::Delete`
- Survives deletion of the linked library item (orphaned sessions retained)

### SessionsData

Top-level serialisation unit for `sessions.json` / `intrada:sessions`.

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `sessions` | `Vec<Session>` | Yes | `#[serde(default)]` | All practice sessions |

**Serialisation**:
- CLI: `sessions.json` in platform data directory via `dirs::data_local_dir()` — e.g. `~/Library/Application Support/intrada/sessions.json` (macOS), `~/.local/share/intrada/sessions.json` (Linux). Pretty-printed, atomic writes
- Web: `localStorage` key `intrada:sessions` (compact JSON)
- Follows same `#[serde(default)]` pattern as `LibraryData`

## View Types

### SessionView

Flattened representation of a session for display. Added to `ViewModel` as `sessions: Vec<SessionView>`.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Session ULID |
| `item_id` | `String` | Linked library item ID |
| `item_title` | `String` | Item title, or "Deleted item" if orphaned |
| `item_type` | `String` | "piece" or "exercise", or "unknown" if orphaned |
| `duration_minutes` | `u32` | Duration in whole minutes |
| `started_at` | `String` | RFC 3339 formatted timestamp |
| `logged_at` | `String` | RFC 3339 formatted timestamp |
| `notes` | `Option<String>` | Session notes |

### ItemPracticeSummary

Added to `LibraryItemView` to show practice stats per item.

| Field | Type | Description |
|-------|------|-------------|
| `session_count` | `u32` | Total sessions for this item |
| `total_minutes` | `u32` | Sum of all session durations |

## Event Types

### SessionEvent

Added as `Event::Session(SessionEvent)` following the per-domain pattern.

| Variant | Fields | Description |
|---------|--------|-------------|
| `Log` | `{ item_id: String, duration_minutes: u32, notes: Option<String> }` | Log a new session |
| `Update` | `{ id: String, duration_minutes: Option<u32>, notes: Option<Option<String>> }` | Edit an existing session |
| `Delete` | `{ id: String }` | Delete a session |

### Root Event additions

| Variant | Fields | Description |
|---------|--------|-------------|
| `SessionsLoaded` | `{ sessions: Vec<Session> }` | Shell reports loaded sessions |

## Storage Effects

Added to the existing `StorageEffect` enum:

| Variant | Fields | Description |
|---------|--------|-------------|
| `SaveSession` | `Session` | Persist a new session |
| `UpdateSession` | `Session` | Persist an updated session |
| `DeleteSession` | `{ id: String }` | Remove a session from storage |
| `LoadSessions` | (none) | Request sessions from shell |

## Validation Rules

| Field | Rule | Error message |
|-------|------|---------------|
| `duration_minutes` | 1 ≤ value ≤ 1440 | "Duration must be between 1 and 1440 minutes" |
| `item_id` | Must be non-empty | "Item ID is required" |
| `notes` | ≤ 5000 chars (reuses `MAX_NOTES`) | "Notes must not exceed 5000 characters" |

New constants:
- `MIN_DURATION: u32 = 1`
- `MAX_DURATION: u32 = 1440`

## Relationships

```text
Session ──────── Piece/Exercise
  item_id ────→ id (soft reference, not enforced)
```

- One session links to exactly one library item
- One library item can have many sessions
- The reference is a soft link (ULID string) — no foreign key enforcement
- Deleting a library item does NOT cascade to its sessions
- Orphaned sessions display with placeholder title "Deleted item"

## JSON Schema

### sessions.json (CLI)

```json
{
  "sessions": [
    {
      "id": "01HXYZ...",
      "item_id": "01HABC...",
      "duration_minutes": 30,
      "started_at": "2026-02-15T10:00:00Z",
      "logged_at": "2026-02-15T10:30:00Z",
      "notes": "Worked on the coda section"
    }
  ]
}
```

### intrada:sessions (Web localStorage)

Same structure, compact JSON (no pretty-printing).
