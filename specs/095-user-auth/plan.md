# Implementation Plan: User Authentication

**Branch**: `095-user-auth` | **Date**: 2026-02-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/095-user-auth/spec.md`

## Summary

Add user authentication to Intrada using Clerk (managed auth) with Google OAuth. The API server validates JWTs via JWKS and scopes all data queries by `user_id`. The web shell loads Clerk's JS SDK, gates the app behind a sign-in screen, and attaches Bearer tokens to all API requests. Auth is optional when `CLERK_ISSUER_URL` is not set, preserving existing test and dev workflows.

## Technical Context

**Language/Version**: Rust stable (1.89.0 in CI; workspace MSRV 1.75+, 2021 edition)
**Primary Dependencies**: axum 0.8, leptos 0.8.x (CSR), crux_core 0.17.0-rc2, gloo-net 0.6, jsonwebtoken 9 (NEW), reqwest 0.12 (NEW), @clerk/clerk-js v5 (CDN)
**Storage**: Turso (managed libsql/SQLite) — adding `user_id` column to items, sessions, routines
**Testing**: cargo test (API integration), wasm-bindgen-test (WASM), Playwright (E2E)
**Target Platform**: WASM (web shell), Linux server (API on Fly.io)
**Project Type**: Three-crate workspace (core, web, api)
**Performance Goals**: Sign-in flow < 10 seconds, sign-out < 2 seconds, silent token refresh (no user interruption)
**Constraints**: No user profile management, Google-only login, pre-launch (no data migration needed)
**Scale/Scope**: Single-user-per-account data isolation, no sharing features

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | PASS | All new code follows existing patterns: cargo clippy, cargo fmt, explicit types, no dead code |
| II. Testing Standards | PASS | API integration tests preserved via auth-optional mode. E2E tests pass with Clerk mock. New auth-specific tests added. |
| III. UX Consistency | PASS | Sign-in screen uses existing glassmorphism design language. Sign-out button follows header nav pattern. WCAG 2.1 AA: sign-in button has accessible label, auth gate shows loading state. |
| IV. Performance | PASS | Clerk JS loads async from CDN (non-blocking). JWKS fetched once at server startup. Token refresh is silent. No additional API calls beyond existing fetch pattern. |
| V. Architecture Integrity | PASS | Core crate untouched (no I/O). Auth lives in web shell (Clerk JS interop) and API server (JWT validation). No cross-crate leakage. |
| VI. Inclusive Design | PASS | Single "Sign in with Google" button — minimal decisions. Predictable: same button, same position, every time. No sudden animations or sounds. |

**Post-design re-check**: PASS — no violations introduced by data model or API contract changes.

## Project Structure

### Documentation (this feature)

```text
specs/095-user-auth/
├── plan.md              # This file
├── research.md          # Phase 0: technology decisions
├── data-model.md        # Phase 1: migration details
├── quickstart.md        # Phase 1: verification guide
├── contracts/           # Phase 1: API contract changes
│   └── auth-changes.md
└── tasks.md             # Phase 2 (created by /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
  intrada-core/           # No changes (pure core, no I/O)
  intrada-api/
    src/
      auth.rs             # NEW — AuthConfig, AuthUser extractor, JWKS fetch
      state.rs            # MODIFIED — add Option<AuthConfig>
      error.rs            # MODIFIED — add Unauthorized variant
      main.rs             # MODIFIED — optional JWKS fetch on startup
      routes/
        mod.rs            # MODIFIED — CORS allow Authorization header
        items.rs          # MODIFIED — add AuthUser extractor to handlers
        sessions.rs       # MODIFIED — add AuthUser extractor to handlers
        routines.rs       # MODIFIED — add AuthUser extractor to handlers
      db/
        items.rs          # MODIFIED — add user_id param to all functions
        sessions.rs       # MODIFIED — add user_id param to all functions
        routines.rs       # MODIFIED — add user_id param to all functions
      migrations.rs       # MODIFIED — add migrations 0013–0018
    Cargo.toml            # MODIFIED — add jsonwebtoken, reqwest
  intrada-web/
    index.html            # MODIFIED — add Clerk JS SDK script + auth helpers
    src/
      clerk_bindings.rs   # NEW — wasm_bindgen interop for Clerk JS
      api_client.rs       # MODIFIED — attach Bearer token to all requests
      app.rs              # MODIFIED — wrap in auth gate
      components/
        app_header.rs     # MODIFIED — add sign-out button
      lib.rs              # MODIFIED — add pub mod clerk_bindings
    Cargo.toml            # MODIFIED — add js-sys
e2e/
  fixtures/
    api-mock.ts           # MODIFIED — add Clerk mock via page.addInitScript()
```

**Structure Decision**: All changes map to existing three-crate structure. No new crates or structural changes needed.
