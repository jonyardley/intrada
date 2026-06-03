# Implementation Plan: API Sync

**Branch**: `021-api-sync` | **Date**: 2026-02-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/021-api-sync/spec.md`

## Summary

Replace localStorage persistence in the web shell with HTTP calls to the deployed REST API (intrada-api on Fly.io). The core (`intrada-core`) remains unchanged — all modifications are in the shell layer (`intrada-web`). The shell's `process_effects()` function will use `gloo-net` and `spawn_local()` to make async HTTP requests instead of synchronous localStorage operations. Session-in-progress crash recovery stays in localStorage. A loading indicator and error messages provide user feedback.

## Technical Context

**Language/Version**: Rust stable (1.89.0 in CI; workspace MSRV 1.75+, 2021 edition)
**Primary Dependencies**: leptos 0.8.x (CSR), crux_core 0.17.0-rc2, gloo-net 0.6 (NEW), wasm-bindgen-futures 0.4 (NEW), serde/serde_json 1
**Storage**: REST API (Turso/libsql via intrada-api) for pieces, exercises, sessions; localStorage for session-in-progress crash recovery only
**Testing**: cargo test (core unit tests), wasm-bindgen-test (WASM shell tests), Playwright (E2E)
**Target Platform**: WASM (browser) targeting Cloudflare Workers (static assets)
**Project Type**: Single workspace with multiple crates (intrada-core, intrada-web, intrada-api)
**Performance Goals**: UI interactions feel instant; initial load completes within 5 seconds (including Fly.io cold start); form submissions show loading state within 100ms
**Constraints**: No offline fallback (operations fail with error if server unreachable); no authentication; WASM single-threaded (no Send/Sync requirements)
**Scale/Scope**: Single user app, ~100 pieces/exercises, ~50 sessions typical

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Code Quality ✅ PASS

- Shell HTTP handlers follow single responsibility (one function per API endpoint)
- `api_client` module encapsulates all HTTP logic with clear error types
- No dead code — localStorage persistence functions removed/replaced
- Error conditions explicit via `ApiError` enum

### II. Testing Standards ✅ PASS

- Core tests unchanged (core is not modified)
- WASM tests updated for new async HTTP patterns
- E2E tests validate full user journeys through the API
- Boundary tests: core emits StorageEffect → shell makes HTTP call → core receives Event

### III. User Experience Consistency ✅ PASS

- Loading indicators on initial load and form submissions (FR-007)
- User-friendly error messages on API failures (FR-006)
- Consistent interaction patterns — form submit flow unchanged from user's perspective
- Duplicate submission prevention (FR-010)

### IV. Performance Requirements ✅ PASS

- No unnecessary re-fetches — refresh-after-mutate only after write operations
- Bundle size impact minimal: gloo-net ~15KB, wasm-bindgen-futures already in dependency tree
- Startup requires 2 HTTP requests (pieces + exercises) which may take 1-3s on cold start — covered by loading indicator

### V. Architecture Integrity ✅ PASS (Critical)

- **Pure Core**: `intrada-core` has ZERO changes — no new events, no new effects, no I/O
- **Shell Isolation**: All HTTP code lives in `intrada-web` (api_client module + updated process_effects)
- **Effect-Driven Communication**: Same StorageEffect enum, shell just handles effects differently
- **Portable by Design**: `cargo test` in intrada-core requires no WASM or browser
- **Validation Sharing**: Core validation rules unchanged; server-side validation is a second layer

### Post-Phase 1 Re-check ✅ PASS

No constitution violations identified. The design strictly follows Approach A (shell-side HTTP) which preserves core purity.

## Project Structure

### Documentation (this feature)

```text
specs/021-api-sync/
├── plan.md              # This file
├── research.md          # Phase 0: approach decisions, async patterns, dependencies
├── data-model.md        # Phase 1: entity mapping and data flow changes
├── quickstart.md        # Phase 1: verification steps
├── contracts/
│   └── api-client.md    # Phase 1: web frontend → REST API contract
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── intrada-core/        # NO CHANGES — pure Crux core
│   └── src/
│       ├── app.rs       # Event, Effect, StorageEffect (unchanged)
│       └── domain/      # Piece, Exercise, Session logic (unchanged)
│
├── intrada-web/         # ALL CHANGES HERE
│   ├── Cargo.toml       # Add gloo-net, wasm-bindgen-futures, serde deps
│   └── src/
│       ├── api_client.rs    # NEW — typed HTTP functions for each API endpoint
│       ├── core_bridge.rs   # MODIFIED — process_effects() uses HTTP instead of localStorage
│       ├── app.rs           # MODIFIED — async init with loading state, remove stub data seeding
│       ├── data.rs          # REMOVED or emptied — stub data no longer needed
│       ├── lib.rs           # MODIFIED — export api_client module
│       ├── types.rs         # MODIFIED — add loading signal type
│       ├── components/      # MODIFIED — add loading indicator, error display
│       └── views/           # MODIFIED — add loading states to forms, disable buttons during submit
│
├── intrada-api/         # NO CHANGES — already deployed
│   └── src/
│       └── routes/      # Pieces, exercises, sessions endpoints
│
└── e2e/                 # MODIFIED — E2E tests may need API server running
```

**Structure Decision**: Existing workspace structure. All changes confined to `intrada-web` crate. No new crates or projects.

## Complexity Tracking

No constitution violations. No complexity justifications needed.
