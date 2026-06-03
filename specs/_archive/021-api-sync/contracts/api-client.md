# API Client Contract: Web Frontend → REST API

**Feature**: 021-api-sync
**Date**: 2026-02-15
**Consumer**: intrada-web (WASM shell)
**Provider**: intrada-api (Fly.io)

## Overview

This contract defines how the web frontend consumes the REST API defined in feature 020-api-server. The web shell uses `gloo-net` to make HTTP requests and maps responses back to core events.

## Base URL Configuration

```
Compile-time: option_env!("INTRADA_API_URL")
Default: "https://intrada-api.fly.dev"
Local dev: "http://localhost:8080"
```

## API Client Module

### Module: `api_client`

A thin module in `intrada-web` that provides typed functions for each API operation.

### Functions

#### Library Operations

```
fetch_pieces() -> Result<Vec<Piece>, ApiError>
  GET /api/pieces
  Response: JSON array of Piece objects

fetch_exercises() -> Result<Vec<Exercise>, ApiError>
  GET /api/exercises
  Response: JSON array of Exercise objects

create_piece(piece: &CreatePiece) -> Result<Piece, ApiError>
  POST /api/pieces
  Body: JSON { title, composer, key?, tempo?, notes?, tags? }
  Response: 201 Created with Piece JSON

update_piece(id: &str, piece: &UpdatePiece) -> Result<Piece, ApiError>
  PUT /api/pieces/{id}
  Body: JSON with changed fields (three-state: omit/null/value)
  Response: 200 OK with updated Piece JSON

delete_piece(id: &str) -> Result<(), ApiError>
  DELETE /api/pieces/{id}
  Response: 200 OK

create_exercise(exercise: &CreateExercise) -> Result<Exercise, ApiError>
  POST /api/exercises
  Body: JSON { title, composer?, category?, key?, tempo?, notes?, tags? }
  Response: 201 Created with Exercise JSON

update_exercise(id: &str, exercise: &UpdateExercise) -> Result<Exercise, ApiError>
  PUT /api/exercises/{id}
  Body: JSON with changed fields
  Response: 200 OK with updated Exercise JSON

delete_exercise(id: &str) -> Result<(), ApiError>
  DELETE /api/exercises/{id}
  Response: 200 OK
```

#### Session Operations

```
fetch_sessions() -> Result<Vec<PracticeSession>, ApiError>
  GET /api/sessions
  Response: JSON array of PracticeSession objects

create_session(session: &PracticeSession) -> Result<PracticeSession, ApiError>
  POST /api/sessions
  Body: JSON { entries, session_notes?, started_at, completed_at, total_duration_secs, completion_status }
  Response: 201 Created with PracticeSession JSON

delete_session(id: &str) -> Result<(), ApiError>
  DELETE /api/sessions/{id}
  Response: 200 OK
```

### Error Type

```
enum ApiError {
    Network(String),       // Connection failed, timeout
    Server(u16, String),   // HTTP status code + error message
    Deserialize(String),   // JSON parsing failed
}
```

Maps to user-facing messages via `Event::LoadFailed(String)`.

## Request Headers

All requests include:
- `Content-Type: application/json` (for POST/PUT)
- No authentication headers (auth is out of scope)

## Response Handling

| HTTP Status | Handling |
|-------------|----------|
| 200 OK | Parse JSON body, use data |
| 201 Created | Parse JSON body, use data |
| 400 Bad Request | Parse `{ "error": "..." }`, show to user |
| 404 Not Found | Show "Item not found" error |
| 500 Internal Server Error | Show "Server error, please try again" |
| Network failure | Show "Unable to connect to server" |

## Refresh-After-Mutate Pattern

After any successful write operation:
1. Re-fetch the full list (GET /api/pieces + GET /api/exercises, or GET /api/sessions)
2. Feed refreshed data to core via `Event::DataLoaded` or `Event::SessionsLoaded`

This ensures the UI always reflects server state and handles server-generated IDs correctly.

## Timeout

No explicit timeout configured. The browser's default fetch timeout applies. The Fly.io machine may take 1-3 seconds to resume from suspend — the loading indicator covers this delay.
