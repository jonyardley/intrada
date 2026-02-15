# Quickstart: API Server

## Prerequisites

- Rust stable (1.78+) with `cargo`
- Turso CLI installed (`brew install tursodatabase/tap/turso` or `curl -sSfL https://get.tur.so/install.sh | bash`)
- Turso account with a database created (`turso db create intrada`)
- Fly.io CLI installed (`brew install flyctl`) and authenticated
- Turso database URL and auth token (see Setup below)

## Setup

### Turso Database

```bash
# Create the database (if not done)
turso db create intrada

# Get the connection URL
turso db show intrada --url
# Output: libsql://intrada-<org>.turso.io

# Create an auth token
turso db tokens create intrada
# Output: eyJhbGci...
```

### Local Environment

```bash
# Set environment variables for local development
export TURSO_DATABASE_URL="libsql://intrada-<org>.turso.io"
export TURSO_AUTH_TOKEN="eyJhbGci..."
export ALLOWED_ORIGIN="http://localhost:8080"
export RUST_LOG="info"
```

### Run Locally

```bash
cargo run -p intrada-api
# Server starts on http://localhost:8080
```

## Verification Steps

### V1: Health Check

```bash
curl -s http://localhost:8080/api/health | jq
# Expected: { "status": "ok", "database": "ok" }
```

### V2: Create a Piece

```bash
curl -s -X POST http://localhost:8080/api/pieces \
  -H "Content-Type: application/json" \
  -d '{"title": "Moonlight Sonata", "composer": "Beethoven", "tags": ["classical"]}' | jq
# Expected: 201 with id, created_at, updated_at
```

### V3: List Pieces

```bash
curl -s http://localhost:8080/api/pieces | jq
# Expected: Array containing the piece from V2
```

### V4: Get Piece by ID

```bash
# Use the id from V2
curl -s http://localhost:8080/api/pieces/<id> | jq
# Expected: Single piece object
```

### V5: Update Piece

```bash
curl -s -X PUT http://localhost:8080/api/pieces/<id> \
  -H "Content-Type: application/json" \
  -d '{"title": "Moonlight Sonata Op. 27 No. 2"}' | jq
# Expected: Updated piece with new title, updated_at changed, created_at preserved
```

### V6: Delete Piece

```bash
curl -s -X DELETE http://localhost:8080/api/pieces/<id> | jq
# Expected: { "message": "Piece deleted" }

curl -s http://localhost:8080/api/pieces/<id>
# Expected: 404
```

### V7: Validation Error

```bash
curl -s -X POST http://localhost:8080/api/pieces \
  -H "Content-Type: application/json" \
  -d '{"title": "", "composer": "Test"}' | jq
# Expected: 400 with error message about title
```

### V8: Create Exercise

```bash
curl -s -X POST http://localhost:8080/api/exercises \
  -H "Content-Type: application/json" \
  -d '{"title": "Hanon No. 1", "category": "Technique"}' | jq
# Expected: 201 (note: composer is optional for exercises)
```

### V9: Save Practice Session

```bash
curl -s -X POST http://localhost:8080/api/sessions \
  -H "Content-Type: application/json" \
  -d '{
    "entries": [{
      "id": "01TEST000000000000000001",
      "item_id": "01TEST000000000000000002",
      "item_title": "Moonlight Sonata",
      "item_type": "piece",
      "position": 0,
      "duration_secs": 600,
      "status": "Completed",
      "notes": null
    }],
    "session_notes": "Good practice",
    "started_at": "2026-02-15T09:00:00Z",
    "completed_at": "2026-02-15T09:10:00Z",
    "total_duration_secs": 600,
    "completion_status": "Completed"
  }' | jq
# Expected: 201 with server-generated session id
```

### V10: List Sessions

```bash
curl -s http://localhost:8080/api/sessions | jq
# Expected: Array with the session from V9, including entries
```

### V11: CORS Preflight

```bash
curl -s -X OPTIONS http://localhost:8080/api/pieces \
  -H "Origin: http://localhost:8080" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: Content-Type" \
  -D - -o /dev/null
# Expected: 200 with Access-Control-Allow-Origin header
```

### V12: Existing Tests Pass

```bash
cargo test
# All existing tests pass (intrada-core + intrada-web)

cargo clippy -- -D warnings
# Zero warnings

cargo fmt --all -- --check
# No formatting issues
```

### V13: Fly.io Deployment (after secrets configured)

```bash
# Set Fly.io secrets
fly secrets set TURSO_DATABASE_URL="libsql://intrada-<org>.turso.io"
fly secrets set TURSO_AUTH_TOKEN="eyJhbGci..."
fly secrets set ALLOWED_ORIGIN="https://intrada.jonyardley.workers.dev"

# Deploy
fly deploy

# Verify
curl -s https://intrada-api.fly.dev/api/health | jq
# Expected: { "status": "ok", "database": "ok" }
```

### V14: End-to-End from Frontend Origin

After deployment, verify from the Cloudflare Workers frontend domain:
1. Open browser DevTools Network tab
2. Navigate to `https://intrada.jonyardley.workers.dev`
3. (Future: when sync is wired) API calls should succeed without CORS errors
