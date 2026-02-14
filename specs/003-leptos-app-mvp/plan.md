# Implementation Plan: Leptos Web App MVP

**Branch**: `003-leptos-app-mvp` | **Date**: 2026-02-14 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-leptos-app-mvp/spec.md`

## Summary

Add a Leptos CSR web application to the intrada workspace as a full Crux web shell, wiring `Core<Intrada>` with stub effect handlers (hardcoded data for LoadAll, no-op for writes). The app serves a styled landing page using Tailwind CSS v4 with a mini library view rendered from the Crux ViewModel. Trunk provides the dev server with auto-rebuild. CI is updated with a WASM build check.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition) — same as existing workspace
**Primary Dependencies**: leptos 0.8.x (csr), crux_core 0.17.0-rc2 (workspace), tailwindcss v4 (standalone CLI), trunk 0.21.x, console_error_panic_hook, wasm-bindgen
**Storage**: N/A (stub data in-memory; no browser persistence)
**Testing**: cargo test (existing core + CLI tests must continue passing); manual browser verification for web crate
**Target Platform**: WASM (wasm32-unknown-unknown) via trunk, served to modern browsers (Chrome, Firefox, Safari)
**Project Type**: Workspace crate (new `crates/intrada-web/` alongside existing `intrada-core` and `intrada-cli`)
**Performance Goals**: Source changes reflected in browser within 5 seconds (SC-002); dev server startup under 3 minutes from clean clone (SC-001)
**Constraints**: No Node.js dependency; CSR-only (no server runtime); must not break existing CLI build/tests (SC-003)
**Scale/Scope**: Single landing page with mini library view; 1 new crate; ~200-400 lines of new code

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Code Quality — PASS

| Rule | Status | Notes |
|------|--------|-------|
| Clarity over cleverness | PASS | Web shell mirrors CLI shell's proven pattern |
| Single Responsibility | PASS | Web crate is a shell only — no business logic |
| Consistent Style | PASS | Same workspace, same `cargo fmt`/`cargo clippy` |
| No Dead Code | PASS | Stub handlers are intentional MVPs, not dead code |
| Explicit over Implicit | PASS | Effect handling is explicit match arms |
| Type Safety | PASS | Rust + Leptos signal types, no `any` equivalent |

### II. Testing Standards — PASS (with notes)

| Rule | Status | Notes |
|------|--------|-------|
| Test Coverage | PASS | Core logic tested in `intrada-core`; web shell is thin UI layer. CI validates WASM compilation. |
| Test Independence | PASS | Existing 82 tests unaffected (SC-003) |
| Meaningful Assertions | PASS | Core behavior tests exist; web crate validates rendering not logic |
| Fast Feedback | PASS | `trunk serve` provides sub-5-second rebuild cycle |
| Failure Clarity | PASS | `console_error_panic_hook` for browser errors; trunk reports compile errors |
| Contract Tests | N/A | No new API boundaries — web shell uses existing Core<Intrada> interface |

### III. User Experience Consistency — PASS (with violation tracked)

| Rule | Status | Notes |
|------|--------|-------|
| Design System Adherence | PASS | Tailwind v4 utility classes provide consistent design tokens |
| Interaction Patterns | PASS | Single page, simple click interactions |
| Error Communication | PASS | `<noscript>` for JS-disabled; clear error in terminal for build failures |
| Loading States | PASS | Stub data loads synchronously — no async loading state needed for MVP |
| Accessibility | PASS | Target Lighthouse accessibility score 90+ (SC-004) |
| Progressive Enhancement | **VIOLATION** | CSR-only means no JS = no content. Tracked in Complexity Tracking. |

### IV. Performance Requirements — PASS

| Rule | Status | Notes |
|------|--------|-------|
| Response Time Budgets | N/A | No API endpoints — client-side only |
| Payload Efficiency | PASS | WASM binary + CSS only; no over-fetching |
| Resource Limits | PASS | Small app; WASM binary expected <2MB |
| Lazy Loading | N/A | Single page, all content loads together |
| Caching Strategy | N/A | No persistent data; stub data in-memory |
| Measurement | PASS | Lighthouse audit for SC-004; rebuild timing for SC-002 |

## Project Structure

### Documentation (this feature)

```text
specs/003-leptos-app-mvp/
├── plan.md              # This file
├── research.md          # Phase 0 output (complete)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── intrada-core/        # Existing: Crux pure core (Event, Model, ViewModel, Effect)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── app.rs       # Intrada App impl (Event → Model → Effect)
│       ├── model.rs     # Model, ViewModel, LibraryItemView
│       ├── error.rs
│       ├── validation.rs
│       └── domain/
│           ├── mod.rs
│           ├── piece.rs
│           ├── exercise.rs
│           └── types.rs
│
├── intrada-cli/         # Existing: CLI shell (SQLite + terminal I/O)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── shell.rs     # Reference pattern for web shell
│       ├── storage.rs
│       └── display.rs
│
└── intrada-web/         # NEW: Leptos CSR web shell
    ├── Cargo.toml       # leptos (csr), intrada-core, console_error_panic_hook
    ├── Trunk.toml       # trunk build/serve configuration
    ├── index.html       # Entry point with data-trunk asset links
    ├── input.css        # Tailwind v4 entry (@import 'tailwindcss'; @source)
    └── src/
        └── main.rs      # Leptos mount + App component + Crux web shell logic

.github/workflows/
└── ci.yml               # MODIFIED: Add wasm-build job
```

**Structure Decision**: New crate `crates/intrada-web/` follows the existing workspace convention where `members = ["crates/*"]` auto-discovers new crates. The web shell depends on `intrada-core` via workspace path reference, identical to how `intrada-cli` depends on it. No contracts/ directory needed as there are no new API boundaries — the web shell uses the existing `Core<Intrada>` public interface.

## Complexity Tracking

> **Filled because Constitution Check has one violation that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Progressive Enhancement: CSR-only (no content without JS) | Spec explicitly requires CSR-only mode. WASM apps inherently require JavaScript. A `<noscript>` element informs users. SSR would satisfy progressive enhancement but is out of scope. | SSR adds server runtime complexity (axum/actix), deployment infrastructure, and hydration logic — all inappropriate for an MVP that validates the Crux web shell architecture. CSR-only is the minimum viable approach. SSR is deferred to a future feature. |
