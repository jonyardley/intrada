# Implementation Plan: Remove CLI Shell

**Branch**: `014-remove-cli` | **Date**: 2026-02-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/014-remove-cli/spec.md`

## Summary

Remove the `intrada-cli` crate from the workspace to focus development on the core library and web app. This involves deleting the CLI crate directory, cleaning up CLI-only workspace dependencies (`clap`, `anyhow`, `dirs`), and updating all living documentation (README.md, CLAUDE.md) to reflect the two-crate workspace. Historical spec documents are preserved as-is. No changes to `intrada-core` or `intrada-web`.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition) — no changes
**Primary Dependencies**: Removing `clap 4.5`, `anyhow 1`, `dirs 5` from workspace; retaining `crux_core`, `serde`, `serde_json`, `ulid`, `chrono`, `thiserror`
**Storage**: N/A — no storage changes (web uses localStorage, CLI's JSON file storage goes away with the crate)
**Testing**: `cargo test` (remaining: 87 core + 36 web unit/WASM), Playwright E2E (15 tests)
**Target Platform**: WASM (web only, after removal)
**Project Type**: Workspace with two crates (`intrada-core`, `intrada-web`)
**Performance Goals**: N/A — removal feature, no new functionality
**Constraints**: CI must pass unchanged; no modifications to core or web crates
**Scale/Scope**: ~1,440 lines of CLI code removed; 3 workspace deps removed; 2 documentation files updated

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality — No Dead Code | ✅ PASS | This feature directly fulfils this principle by removing an entire unused crate |
| I. Code Quality — Single Responsibility | ✅ PASS | Workspace becomes more focused (core + web only) |
| I. Code Quality — Consistent Style | ✅ PASS | No new code; formatting rules unchanged |
| II. Testing Standards — Test Coverage | ✅ PASS | CLI tests are removed along with the CLI code; remaining core + web tests unaffected |
| II. Testing Standards — Test Independence | ✅ PASS | No test coupling between CLI and other crates |
| III. UX Consistency | ✅ N/A | No user-facing changes |
| IV. Performance | ✅ N/A | No performance impact |

**Gate result**: PASS — all applicable principles satisfied. This feature is a direct implementation of the "No Dead Code" principle.

## Project Structure

### Documentation (this feature)

```text
specs/014-remove-cli/
├── plan.md              # This file
├── research.md          # Phase 0 output (minimal — removal feature)
├── quickstart.md        # Phase 1 output (verification steps)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
  intrada-core/       # Pure core library — UNCHANGED
    src/
      app.rs
      model.rs
      validation.rs
      error.rs
      domain/
        piece.rs
        exercise.rs
        types.rs
        mod.rs
  intrada-web/        # Web shell — UNCHANGED
    src/
      lib.rs
      main.rs
      app.rs
      core_bridge.rs
      data.rs
      helpers.rs
      validation.rs
      types.rs
      components/
      views/
    tests/
      wasm.rs
e2e/                  # Playwright E2E tests — UNCHANGED
```

**Structure Decision**: After removal, the workspace contains exactly two crates under `crates/`. The `members = ["crates/*"]` glob continues to work. No source code structure changes needed — only the CLI directory deletion and workspace dependency cleanup.

### Files Modified

| File | Action | Reason |
|------|--------|--------|
| `crates/intrada-cli/` | DELETE (entire directory) | FR-001: Remove CLI crate |
| `Cargo.toml` (root) | EDIT | FR-003: Remove `clap`, `anyhow`, `dirs` from workspace dependencies |
| `README.md` | REWRITE | FR-004, FR-005: Remove CLI content, update architecture and storage descriptions |
| `CLAUDE.md` | EDIT | FR-006: Remove CLI from project structure and technology references |

### Files NOT Modified

| File | Reason |
|------|--------|
| `crates/intrada-core/**` | FR-009: Core has no CLI dependency |
| `crates/intrada-web/**` | No CLI references in web shell |
| `.github/workflows/ci.yml` | FR-007: CI uses `cargo test`/`cargo clippy` which auto-discover workspace members |
| `specs/001-*` through `specs/013-*` | FR-008: Historical specs preserved as-is |

## Complexity Tracking

> No violations. This feature reduces complexity by removing a crate and its dependencies.
