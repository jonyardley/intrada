# Data Model: API Sync

**Feature**: 021-api-sync
**Date**: 2026-02-15

## Overview

This feature does not introduce new entities. It changes the persistence layer for existing entities from localStorage to HTTP API calls. The data model is defined by `intrada-core` types and the REST API from feature 020-api-server.

## Entities (unchanged — defined in intrada-core)

### Piece

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| id | String (ULID) | Yes (server-generated) | Unique identifier |
| title | String | Yes | 1-500 characters |
| composer | String | Yes | 1-200 characters |
| key | Option<String> | No | Max 50 characters |
| tempo | Option<Tempo> | No | See Tempo below |
| notes | Option<String> | No | Max 5000 characters |
| tags | Vec<String> | No (defaults to []) | Each tag 1-100 chars |
| created_at | DateTime<Utc> | Yes (server-generated) | ISO 8601 |
| updated_at | DateTime<Utc> | Yes (server-generated) | ISO 8601 |

### Exercise

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| id | String (ULID) | Yes (server-generated) | Unique identifier |
| title | String | Yes | 1-500 characters |
| composer | Option<String> | No | Max 200 characters |
| category | Option<String> | No | Max 100 characters |
| key | Option<String> | No | Max 50 characters |
| tempo | Option<Tempo> | No | See Tempo below |
| notes | Option<String> | No | Max 5000 characters |
| tags | Vec<String> | No (defaults to []) | Each tag 1-100 chars |
| created_at | DateTime<Utc> | Yes (server-generated) | ISO 8601 |
| updated_at | DateTime<Utc> | Yes (server-generated) | ISO 8601 |

### Tempo

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| marking | Option<String> | No | Max 100 characters |
| bpm | Option<u16> | No | 1-400 |

### PracticeSession

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| id | String (ULID) | Yes (server-generated) | Unique identifier |
| entries | Vec<SetlistEntry> | Yes | At least 1 entry |
| session_notes | Option<String> | No | Max 5000 characters |
| started_at | DateTime<Utc> | Yes | ISO 8601 |
| completed_at | DateTime<Utc> | Yes | ISO 8601 |
| total_duration_secs | u64 | Yes | Sum of entry durations |
| completion_status | CompletionStatus | Yes | "Completed" or "EndedEarly" |

### SetlistEntry

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| id | String (ULID) | Yes | Client-generated |
| item_id | String | Yes | References piece/exercise |
| item_title | String | Yes | Denormalised snapshot |
| item_type | String | Yes | "piece" or "exercise" |
| position | usize | Yes | 0-based |
| duration_secs | u64 | Yes | Time spent |
| status | EntryStatus | Yes | "Completed", "Skipped", "NotAttempted" |
| notes | Option<String> | No | Max 5000 characters |

### ActiveSession (unchanged — localStorage only)

The in-progress session remains in localStorage for crash recovery (FR-008). It is not sent to the API until completed.

## Data Flow Changes

### Before (localStorage)

```
User action → Core emits StorageEffect → Shell writes/reads localStorage → Core receives Event
```

### After (API sync)

```
User action → Core emits StorageEffect → Shell sends HTTP request to API → Core receives Event
```

### Hybrid (session-in-progress)

```
Session tick → Core emits SaveSessionInProgress → Shell writes localStorage (unchanged)
Session complete → Core emits SavePracticeSession → Shell sends POST /api/sessions
```

## StorageEffect → API Mapping

| StorageEffect | HTTP Method | Endpoint | Response Event |
|---------------|-------------|----------|----------------|
| LoadAll | GET | /api/pieces + /api/exercises | DataLoaded { pieces, exercises } |
| SavePiece(piece) | POST | /api/pieces | (fire-and-forget, error → LoadFailed) |
| SaveExercise(exercise) | POST | /api/exercises | (fire-and-forget, error → LoadFailed) |
| UpdatePiece(piece) | PUT | /api/pieces/{id} | (fire-and-forget, error → LoadFailed) |
| UpdateExercise(exercise) | PUT | /api/exercises/{id} | (fire-and-forget, error → LoadFailed) |
| DeleteItem { id } | DELETE | /api/pieces/{id} or /api/exercises/{id} | (fire-and-forget, error → LoadFailed) |
| LoadSessions | GET | /api/sessions | SessionsLoaded { sessions } |
| SavePracticeSession(session) | POST | /api/sessions | (fire-and-forget, error → LoadFailed) |
| DeletePracticeSession { id } | DELETE | /api/sessions/{id} | (fire-and-forget, error → LoadFailed) |
| SaveSessionInProgress(session) | N/A | localStorage (unchanged) | — |
| ClearSessionInProgress | N/A | localStorage (unchanged) | — |

## ID Generation

Currently the core generates ULIDs for new pieces/exercises/sessions. With the API, the server generates IDs. Two approaches:

1. **Optimistic**: Core generates ID, shell sends it to server, server accepts it — requires API changes.
2. **Server-generated**: Shell sends data without ID, server returns the created entity with ID, shell feeds it back to core.

The API server already generates IDs (020-api-server). The core currently generates IDs too. Since the server response includes the full entity with server-generated ID, the shell should update the core's model with the server response. This means:

- For `SavePiece`: POST to API, receive response with server-generated ID, dispatch `Event::DataLoaded` with refreshed data (or a new event to replace the item).
- Simplest approach: After any mutation, re-fetch the full list from the API and dispatch `DataLoaded`/`SessionsLoaded`. This is simple and ensures consistency.

### Decision: Refresh-after-mutate pattern

After any write operation (create/update/delete), re-fetch the full list from the API. This eliminates ID synchronisation issues and ensures the UI always reflects the server state.
