# Research: Shuttle API Server & Database

**Feature**: 017-shuttle-api-deploy
**Date**: 2026-02-15
**Status**: Complete

## R1: Database Engine Selection

**Decision**: Shuttle-managed Postgres via `shuttle-shared-db`

**Rationale**: Zero-configuration provisioning — add a `#[shuttle_shared_db::Postgres]` parameter annotation and Shuttle auto-provisions a Postgres database on deploy. No external accounts, no token management, no secrets files for the DB connection. Returns a `sqlx::PgPool` directly. Local dev spins up a Docker Postgres container automatically via `cargo shuttle run`.

**Alternatives considered**:

| Option | Pros | Cons |
|--------|------|------|
| **shuttle-shared-db (Postgres)** ✅ | Zero-config, auto-provisioned, no external accounts, Docker local dev | Tied to Shuttle platform |
| shuttle-turso (LibSQL) | Edge-hosted SQLite, portable if leaving Shuttle | Requires external Turso account, token management via Secrets.toml, more setup |

**Free tier**: 0.5 GB shared database storage, 1 GB network egress — more than sufficient for a single-user music practice app.

## R2: Web Framework & Static File Serving

**Decision**: Axum 0.8 via `shuttle-axum`, with `tower-http` ServeDir for static WASM files

**Rationale**: Axum is the de facto standard for Rust web APIs and has first-class Shuttle support. The SPA pattern uses `ServeDir::new("dist").fallback(ServeFile::new("dist/index.html"))` as a `fallback_service` on the outer Router, so API routes match first and unmatched paths fall through to the WASM app's client-side router.

**Key versions**:
- axum 0.8.8 (minimum Rust 1.78)
- tower-http 0.6.x (compatible with axum 0.8; requires `fs` feature for ServeDir)
- shuttle-runtime 0.57, shuttle-axum 0.57

**SPA serving pattern**:
```rust
let spa = ServeDir::new("dist")
    .fallback(ServeFile::new("dist/index.html"));

let app = Router::new()
    .nest("/api", api_router())
    .fallback_service(spa);
```

**Key caveats**:
- axum 0.8 uses `{param}` path syntax (not `:param`)
- `.fallback()` preserves 200 status (correct for SPA); `.not_found_service()` forces 404 (wrong for SPA)
- CORS is not needed when WASM app and API are served from same origin

## R3: CI/CD Deployment Pipeline

**Decision**: `shuttle-hq/deploy-action@v2` GitHub Action, triggered on merge to main

**Rationale**: Official Shuttle GitHub Action with simple API key authentication. Integrates into existing CI workflow. Alternative is Shuttle's direct GitHub integration (auto-redeploy on push), but the Action approach gives explicit control over build/test gating.

**Configuration**:
```yaml
- uses: shuttle-hq/deploy-action@v2
  with:
    shuttle-api-key: ${{ secrets.SHUTTLE_API_KEY }}
    project-id: "proj_XXXXXXXXXXXX"
```

**Secrets required**: `SHUTTLE_API_KEY` stored in GitHub repository secrets.

## R4: Core Type Reusability

**Decision**: Server crate imports `intrada-core` directly — no refactoring needed

**Rationale**: All domain types, DTOs, validation functions, and error types in intrada-core are cleanly separated from Crux. They carry no I/O, no async, no browser dependencies. The server crate can import and use them directly.

**Reusable imports**:
- Domain types: `Piece`, `Exercise`, `PracticeSession`, `SetlistEntry`, `Tempo`
- Container types: `LibraryData`, `SessionsData`
- Request DTOs: `CreatePiece`, `CreateExercise`, `UpdatePiece`, `UpdateExercise`
- Error type: `LibraryError`
- Query type: `ListQuery`
- Validation: all `validate_*` functions and `MAX_*`/`MIN_*` constants

**Not imported** (Crux-coupled): `Event`, `Effect`, `StorageEffect`, `Intrada` app, session state machine types, view model types.

## R5: Error Handling Pattern

**Decision**: Custom `ApiError` enum implementing axum's `IntoResponse`

**Rationale**: Axum requires handlers to always produce a response. The pattern maps domain errors (`LibraryError`) to HTTP status codes and JSON error bodies. Handlers return `Result<impl IntoResponse, ApiError>`.

**Mapping**:
| Domain Error | HTTP Status | Response |
|---|---|---|
| `LibraryError::Validation(msg)` | 400 Bad Request | `{ "error": msg }` |
| `LibraryError::NotFound(msg)` | 404 Not Found | `{ "error": msg }` |
| Database/internal errors | 500 Internal Server Error | `{ "error": "Internal server error" }` |

## R6: Project Structure

**Decision**: New `crates/intrada-api/` server crate in the workspace

**Rationale**: Follows the existing workspace pattern (`crates/*`). The server crate depends on `intrada-core` for types and validation, plus Shuttle/Axum/SQLx for the server runtime.

**Dependencies**:
```toml
[dependencies]
intrada-core = { path = "../intrada-core" }
shuttle-runtime = "0.57"
shuttle-axum = "0.57"
shuttle-shared-db = { version = "0.57", features = ["postgres", "sqlx"] }
axum = "0.8"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "chrono"] }
tower-http = { version = "0.6", features = ["fs"] }
tokio = { version = "1", features = ["macros"] }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
ulid = { workspace = true }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

## R7: Shuttle.toml & Build Assets

**Decision**: Declare WASM build output as build assets in `Shuttle.toml`

**Rationale**: Shuttle's `[build]` section copies declared assets into the runtime image. The Trunk-built WASM app output (in `dist/`) needs to be available at runtime for `ServeDir` to serve.

**Configuration**:
```toml
name = "intrada"

[build]
assets = ["dist/*"]
```

**Build order**: Trunk builds the WASM app to `dist/`, then `cargo shuttle deploy` packages the server crate with `dist/` as build assets.
