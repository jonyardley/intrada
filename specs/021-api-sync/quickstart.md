# Quickstart: API Sync

**Feature**: 021-api-sync
**Date**: 2026-02-15

## Prerequisites

1. **API server running** — either locally or on Fly.io:
   - Local: `cd crates/intrada-api && cargo run` (with Turso env vars set)
   - Production: `https://intrada-api.fly.dev` (already deployed)

2. **CORS configured** on API server to accept requests from:
   - Local dev: `http://localhost:8080` (Trunk dev server)
   - Production: `https://intrada.<account>.workers.dev`

## Local Development

### 1. Start the API server locally

```bash
export TURSO_DATABASE_URL="libsql://your-db.turso.io"
export TURSO_AUTH_TOKEN="your-token"
export ALLOWED_ORIGIN="http://localhost:8080"
cd crates/intrada-api && cargo run
```

### 2. Start the web app pointing to local API

```bash
cd crates/intrada-web
INTRADA_API_URL=http://localhost:8080 trunk serve
```

### 3. Open the app

Navigate to `http://localhost:8080` in your browser.

## Verification Steps

### V1: Library loads from API

1. Open the app in a browser
2. The library view should show a loading indicator briefly
3. If the API has no data, the library should show an empty state (no stub/seed data)
4. Check browser DevTools Network tab: should see `GET /api/pieces` and `GET /api/exercises`

### V2: Add a piece via API

1. Click "Add" and fill in piece details (title: "Test Piece", composer: "Test Composer")
2. Submit the form
3. The piece should appear in the library
4. Check DevTools: should see `POST /api/pieces` followed by `GET /api/pieces`
5. Open a second browser/incognito window — the piece should appear there too

### V3: Edit a piece via API

1. Click on the piece created in V2
2. Edit the title to "Updated Piece"
3. Save changes
4. The updated title should appear in the library
5. Check DevTools: should see `PUT /api/pieces/{id}`

### V4: Delete a piece via API

1. Click on the piece, then delete it
2. Confirm deletion
3. The piece should disappear from the library
4. Check DevTools: should see `DELETE /api/pieces/{id}`
5. Refresh the other browser window — piece should be gone there too

### V5: Sessions load from API

1. Navigate to the Sessions view
2. Should show loading indicator briefly, then display session history (or empty state)
3. Check DevTools: should see `GET /api/sessions`

### V6: Complete a practice session

1. Create a piece (if library is empty)
2. Start a new practice session, add the piece to the setlist
3. Start and complete the session
4. The completed session should appear in the sessions list
5. Check DevTools: should see `POST /api/sessions`

### V7: Error handling — network failure

1. Stop the API server (or disconnect from network)
2. Try to add a piece
3. Should see a user-friendly error message (not a blank screen or silent failure)
4. The app should not crash

### V8: Loading state — Fly.io cold start

1. Wait for the Fly.io machine to suspend (after 1 minute of inactivity)
2. Open the app
3. Should see a loading indicator while the machine resumes (1-3 seconds)
4. Data should load successfully after the delay

### V9: Session-in-progress crash recovery

1. Start a practice session
2. Close the browser tab
3. Reopen the app
4. The in-progress session should be recovered from localStorage (not from the API)

### V10: No stub data on first load

1. Clear all localStorage
2. Ensure the API database is empty (or use a fresh database)
3. Open the app
4. Should show an empty library — no stub/seed data

## Automated Tests

```bash
# Core tests (should still pass — no core changes)
cd crates/intrada-core && cargo test

# Web shell WASM tests
cd crates/intrada-web && wasm-pack test --headless --chrome

# All workspace tests
cargo test

# Clippy
cargo clippy -- -D warnings

# E2E tests (with API server running)
cd e2e && npx playwright test
```

## CI/CD

The existing CI pipeline (`cargo test`, `cargo clippy`, WASM build) should continue to pass. The deploy workflow builds with `trunk build --release` which will compile with the default API URL.

To set a custom API URL for production builds, add to the deploy workflow:
```yaml
env:
  INTRADA_API_URL: https://intrada-api.fly.dev
```
