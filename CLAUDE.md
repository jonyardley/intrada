# intrada Development Guidelines

> **Maintenance reminder**: Review this file for accuracy every 2 weeks or after any
> significant feature lands. Last reviewed: 2026-02-20.

## Project Overview

intrada is a music practice companion app. Users sign in with Google (via Clerk),
manage a library of pieces and exercises, run timed practice sessions with scoring,
build reusable routines, and view analytics.

## Project Structure

```text
crates/
  intrada-core/   # Pure Crux core — business logic, no I/O, no side effects
  intrada-web/    # Web shell — Leptos 0.8 CSR + WASM, Clerk auth UI
  intrada-api/    # REST API — Axum 0.8 + Turso (libsql), JWT validation
e2e/              # Playwright E2E tests
specs/            # SpecKit design artifacts
```

## Tech Stack

- **Language**: Rust stable (1.89.0 in CI; workspace MSRV 1.75+, intrada-api requires 1.78+ for axum 0.8)
- **Core**: crux_core 0.17.0-rc2, serde 1, ulid 1, chrono 0.4, thiserror 1
- **API**: axum 0.8, tokio 1, libsql 0.9 (remote), tower-http 0.6 (CORS), reqwest 0.12, jsonwebtoken 10 (rust_crypto feature), tracing 0.1
- **Web**: leptos 0.8.x (CSR), leptos_router 0.8.x, gloo-net 0.6, web-sys 0.3, wasm-bindgen 0.2, send_wrapper 0.6, Tailwind CSS v4 (standalone CLI), trunk 0.21.x
- **Auth**: Clerk (managed auth, Google OAuth only), @clerk/clerk-js v5 (loaded via CDN)
- **Database**: Turso (managed libsql/SQLite) via HTTP protocol
- **Testing**: cargo test (unit/integration), wasm-bindgen-test 0.3, Playwright (E2E)
- **CI/CD**: GitHub Actions, deploy to Cloudflare Workers (web) + Fly.io (API)

## Authentication

- **Provider**: Clerk with Google OAuth sign-in
- **API auth**: JWT (RS256) validated against Clerk JWKS (`/.well-known/jwks.json`)
- **Key refresh**: Background tokio task refreshes JWKS keys every 60 minutes
- **User isolation**: All DB queries scope by `user_id` from JWT `sub` claim
- **Auth disabled mode**: When `CLERK_ISSUER_URL` is unset, `AuthUser("")` is returned — all data shares an empty user_id. This is for local dev/test only.
- **Frontend 401 retry**: API client retries once with a fresh Clerk token on 401

Key files: `intrada-api/src/auth.rs`, `intrada-web/src/clerk_bindings.rs`, `intrada-web/src/api_client.rs`

## Environment Variables

### API server (intrada-api)
- `TURSO_DATABASE_URL` — required, Turso database URL
- `TURSO_AUTH_TOKEN` — required, Turso auth token
- `CLERK_ISSUER_URL` — required in production (e.g. `https://clerk.myintrada.com`), omit to disable auth
- `ALLOWED_ORIGIN` — CORS origin (default: `http://localhost:8080`)
- `PORT` — server port (default: `3001`)
- `RUST_LOG` — tracing filter (default: `info`)

### Web build (intrada-web, compile-time)
- `CLERK_PUBLISHABLE_KEY` — Clerk publishable key, baked into WASM at build time
- `INTRADA_API_URL` — API base URL (default: `https://intrada-api.fly.dev`)

## Storage

- **All persistent data** (items, sessions, routines) is stored via the REST API in Turso
- **localStorage** is used ONLY for `intrada:session-in-progress` crash recovery
- No other localStorage keys are used — the old `intrada:library` and `intrada:sessions` keys were removed when the API was introduced

## Commands

```bash
cargo fmt --check          # must pass before commit — CI enforces this
cargo test                 # run all workspace tests
cargo clippy               # lint check
cargo test -p intrada-api  # API tests only (includes auth tests)
```

## Architecture Patterns

- **Crux core/shell split**: `intrada-core` contains zero I/O. All side effects are represented as enum variants and executed by the web shell. The core must compile on any Rust target without WASM dependencies.
- **Effect enum**: Currently named `StorageEffect` (historical misnomer — it carries all side effects, not just storage)
- **API client**: `api_client.rs` has generic helpers (`get_json`, `post_json`, `put_json`, `delete`) with built-in 401 retry
- **Validation**: `intrada-core/src/validation.rs` is the single source of truth for all validation constants and rules
- **Database**: Positional column indexing (`row.get(0)`, etc.) with a `SELECT_COLUMNS` const to keep column order in one place
- **Migrations**: Sequential numbered migrations in `intrada-api/src/migrations.rs`, each must be a single SQL statement

## Code Style

- Rust stable, 2021 edition
- Follow standard Rust conventions
- `cargo fmt` and `cargo clippy -- -D warnings` must pass
- No `unwrap()` without justification

## Known Tech Debt

- `StorageEffect` should be renamed to `AppEffect` or `SideEffect`
- Sessions and routines SQL is inline in route handlers (items has a dedicated `db/items.rs` module)
- Legacy `pieces` and `exercises` tables from early migrations still exist in the schema
- `dependabot.yml` needs `package-ecosystem` set to `"cargo"`

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
