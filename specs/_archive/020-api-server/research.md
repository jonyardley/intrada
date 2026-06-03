# Research: API Server

## R1: Database Client — libsql vs sqlx

**Decision**: Use `libsql` crate (v0.9) with `remote` feature flag for connecting to Turso.

**Rationale**:
- libsql is the official Rust SDK for Turso — first-class support, maintained by the Turso team
- The `remote` feature flag enables HTTP-only connections without compiling embedded SQLite C code, keeping the build fast and the Docker image small
- `db.connect()` is lightweight per-request — no separate connection pool crate needed
- Positional params (`?1`, `?2`) and named params (`:id`, `:title`) both supported
- `libsql::Error` provides structured error types for connection failures, SQL errors, etc.

**Alternatives considered**:
- `sqlx` with SQLite driver: sqlx doesn't support Turso's HTTP protocol — only local SQLite files. Would require running a local SQLite and losing Turso's managed database benefits (backups, replication, dashboard).
- `rusqlite`: Local-only, no remote database support. Same problem as sqlx.

## R2: Database Migrations — libsql_migration

**Decision**: Use `libsql_migration` crate (v0.2.2) with the `content` feature for embedded migrations.

**Rationale**:
- Content-based migrations embed SQL directly in the Rust binary — no filesystem dependency at runtime, which is ideal for Docker/Fly.io deployment
- Migrations are defined as `const` arrays of `(id, sql)` tuples, called sequentially on startup
- The crate automatically creates and manages a `libsql_migrations` tracking table
- Each migration is idempotent — already-executed migrations are skipped
- Simple API: `migrate(conn, id, sql)` returns `Executed` or `AlreadyExecuted`

**Alternatives considered**:
- `libsql_migration` with `dir` feature: File-based migrations require the migration directory to exist in the Docker image. Adds complexity for no benefit when the migration count is small.
- Manual `CREATE TABLE IF NOT EXISTS`: Works but doesn't track which migrations have run, making schema evolution fragile.

## R3: JSON Columns for Tags

**Decision**: Store tags as JSON text columns (`TEXT NOT NULL DEFAULT '[]'`) and use serde for serialisation/deserialisation in Rust.

**Rationale**:
- SQLite doesn't support array types (unlike Postgres `TEXT[]` used in the old Shuttle-based API)
- JSON text is the standard SQLite pattern for structured data — SQLite has built-in `json_extract()` and `json_each()` functions if server-side querying is needed later
- serde_json already in the workspace — `serde_json::to_string(&tags)` for writes, `serde_json::from_str(&json)` for reads
- Tags are always read/written as a complete list (no partial updates), so JSON serialisation is efficient

**Alternatives considered**:
- Separate `tags` join table: More normalised but adds complexity (joins, separate CRUD) for a field that's always loaded with its parent entity. Overkill for the expected dataset size.
- Comma-separated string: Fragile — tags containing commas break parsing. JSON is safer.

## R4: Tempo Storage

**Decision**: Store tempo as two separate columns (`tempo_marking TEXT`, `tempo_bpm INTEGER`) rather than a JSON object.

**Rationale**:
- Matches the existing pattern from the Shuttle-based API (proven to work)
- Individual columns are queryable without JSON functions (e.g., `WHERE tempo_bpm > 120`)
- Null handling maps naturally: both null = no tempo, one null = partial tempo
- Simpler SQL than parsing a JSON object for every read

**Alternatives considered**:
- Single JSON column for tempo: Would require `json_extract()` for every read. Adds complexity when two simple columns suffice.

## R5: Deployment Platform — Fly.io

**Decision**: Deploy on Fly.io with a multi-stage Docker build using `cargo-chef` for layer caching.

**Rationale**:
- Fly.io supports Rust natively via Docker — no runtime adapter or vendor SDK needed
- Multi-stage build with `cargo-chef` caches dependency compilation, making subsequent deploys fast (only recompiles changed source)
- `auto_stop_machines` / `auto_start_machines` in fly.toml provides scale-to-zero for cost efficiency on the free/hobby tier
- Built-in health checks via `[[http_service.checks]]` in fly.toml
- Secrets management via `fly secrets set` for Turso credentials
- London (`lhr`) region available — close to the user

**Alternatives considered**:
- Cloudflare Workers: Would require rewriting the API in JavaScript/TypeScript (Workers doesn't run Rust natively as a server). Loses all existing Axum route patterns and intrada-core validation reuse.
- Shuttle.rs: Shutting down — login broken, projects being deleted.

## R6: CORS Configuration

**Decision**: Use `tower-http` CorsLayer with a configurable allowed origin read from an environment variable.

**Rationale**:
- `tower-http` v0.6 already in the workspace from the Shuttle-based API
- CorsLayer handles OPTIONS preflight automatically — no manual route needed
- Origin from env var (`ALLOWED_ORIGIN`) allows different values for local dev vs production without code changes
- Specific origin (not wildcard `*`) is more secure and required if credentials are added later

**Alternatives considered**:
- Hardcoded origin: Would require code changes to switch between dev and production.
- Allow all origins (`*`): Less secure. Works for now (no auth) but would need changing when auth is added. Better to start with specific origins.

## R7: API Crate Structure — New vs Reuse

**Decision**: Create a new `crates/intrada-api/` crate from scratch, reusing route handler patterns from the Shuttle-based API but with a completely new database layer.

**Rationale**:
- The old API crate is on a different branch (`017-shuttle-api-deploy`) with Shuttle and sqlx dependencies that are being replaced entirely
- Route handler signatures and patterns (Axum extractors, validation flow, error mapping) are directly reusable
- The DB layer must be completely rewritten (sqlx → libsql, Postgres → SQLite)
- Starting fresh avoids carrying over Shuttle-specific code (shuttle_runtime, shuttle_shared_db)
- The error handling pattern (ApiError with IntoResponse) can be copied and adapted

**Alternatives considered**:
- Cherry-pick from 017 branch: Would bring in Shuttle dependencies that need immediate removal. More work than copying the patterns manually.

## R8: Workspace MSRV

**Decision**: Keep workspace MSRV at 1.75 but note that axum 0.8 requires 1.78+. The CI already uses 1.89.0.

**Rationale**:
- The workspace `Cargo.toml` specifies `rust-version = "1.75"` but CI uses 1.89.0
- Axum 0.8 requires 1.78+ — this is already satisfied by CI
- No need to bump the workspace MSRV just for the API crate — the core and web crates don't need it
- The API crate can specify its own `rust-version = "1.78"` if needed

## R9: CI Pipeline for API Server

**Decision**: Add `api-test` and `api-clippy` jobs to the existing CI workflow, conditional on API crate changes.

**Rationale**:
- The API server has different dependencies (libsql, tokio) than the WASM web shell
- Running API tests on every PR (even web-only changes) wastes CI minutes
- Path filters (`paths: ['crates/intrada-api/**', 'crates/intrada-core/**']`) can scope the jobs
- The deploy job for the API will be a separate concern (Fly.io, not Cloudflare)

**Alternatives considered**:
- Separate workflow file: Could work but makes the pipeline harder to reason about (multiple workflow files). The existing ci.yml already handles conditional jobs well.
- Always run API tests: Wasteful for web-only PRs but simpler. Acceptable for now given the small test suite.
