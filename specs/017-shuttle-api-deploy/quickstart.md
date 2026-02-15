# Quickstart: Shuttle API Server & Database

**Feature**: 017-shuttle-api-deploy
**Date**: 2026-02-15
**Branch**: `017-shuttle-api-deploy`

## Prerequisites

- Rust stable (1.78+ for axum 0.8)
- Docker (for local Shuttle Postgres)
- `cargo-shuttle` CLI (`cargo install cargo-shuttle`)
- Trunk (`cargo install trunk`)
- `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)

## Local Development

### 1. Build the WASM frontend

```bash
cd crates/intrada-web
trunk build
```

This outputs to `crates/intrada-web/dist/`. Copy or symlink to where the API server expects it:

```bash
cp -r crates/intrada-web/dist crates/intrada-api/dist
```

### 2. Run the API server locally

```bash
cd crates/intrada-api
cargo shuttle run
```

This will:
- Start a local Postgres container via Docker
- Run SQL migrations
- Serve the API at `http://localhost:8000`
- Serve the WASM app at `http://localhost:8000/`

### 3. Verify the server is running

```bash
curl http://localhost:8000/api/health
# Expected: {"status":"ok"}
```

## Verification Checklist

Run these checks to confirm the feature is working correctly.

### V1: Server starts and serves WASM app
- [ ] `cargo shuttle run` starts without errors
- [ ] Navigate to `http://localhost:8000/` in a browser
- [ ] The Intrada web app loads and displays the home screen
- [ ] Refreshing on a deep link (e.g., `/library`) returns the app (SPA routing works)

### V2: Health endpoint
- [ ] `GET /api/health` returns `{"status":"ok"}` with status 200

### V3: Piece CRUD
- [ ] `POST /api/pieces` with valid data returns 201 and the created piece
- [ ] `GET /api/pieces` returns an array including the created piece
- [ ] `GET /api/pieces/{id}` returns the specific piece
- [ ] `PUT /api/pieces/{id}` updates the piece and returns 200
- [ ] `DELETE /api/pieces/{id}` deletes the piece and returns 200
- [ ] `POST /api/pieces` with empty title returns 400 with validation error
- [ ] `GET /api/pieces/{nonexistent}` returns 404

### V4: Exercise CRUD
- [ ] `POST /api/exercises` with valid data returns 201
- [ ] `GET /api/exercises` returns all exercises
- [ ] `PUT /api/exercises/{id}` updates successfully
- [ ] `DELETE /api/exercises/{id}` deletes successfully
- [ ] Validation errors return 400

### V5: Practice session persistence
- [ ] `POST /api/sessions` with valid session data returns 201
- [ ] `GET /api/sessions` returns all saved sessions with entries
- [ ] `GET /api/sessions/{id}` returns a specific session with entries
- [ ] Sessions with empty entries list are rejected with 400

### V6: Web app uses API
- [ ] Create a piece through the web UI → piece appears in library
- [ ] Clear browser localStorage → reload → piece still appears (fetched from server)
- [ ] Edit a piece → changes persist across browser clears
- [ ] Delete a piece → piece is removed from server

### V7: Local cache behaviour
- [ ] On load, cached data appears before server response (inspect with slow connection or throttling)
- [ ] After server fetch, UI updates if data differs from cache

### V8: Existing tests pass
- [ ] `cargo test` in workspace root — all tests pass (142+ unit tests)
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] E2E tests pass against the API-backed app (may require test infrastructure updates)

### V9: CI/CD pipeline
- [ ] CI workflow builds and tests the `intrada-api` crate
- [ ] Deploy job uses `shuttle-hq/deploy-action@v2`
- [ ] After merge to main, app is accessible at the Shuttle URL

## Quick API Test Script

```bash
BASE="http://localhost:8000/api"

# Health
curl -s "$BASE/health" | jq .

# Create a piece
curl -s -X POST "$BASE/pieces" \
  -H "Content-Type: application/json" \
  -d '{"title":"Clair de Lune","composer":"Debussy","tags":["impressionist"]}' | jq .

# List pieces
curl -s "$BASE/pieces" | jq .

# Create an exercise
curl -s -X POST "$BASE/exercises" \
  -H "Content-Type: application/json" \
  -d '{"title":"Hanon No. 1","category":"Technique","tags":["warmup"]}' | jq .

# List exercises
curl -s "$BASE/exercises" | jq .
```
