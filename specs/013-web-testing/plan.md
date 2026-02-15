# Implementation Plan: Web UI Testing & E2E Test Infrastructure

**Branch**: `013-web-testing` | **Date**: 2026-02-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/013-web-testing/spec.md`

## Summary

Add three layers of test coverage to the `intrada-web` crate, which currently has zero tests. Layer 1: Rust unit tests for pure functions in `helpers.rs` and `validation.rs` (standard `cargo test`). Layer 2: WASM integration tests for localStorage persistence logic in `core_bridge.rs` (via `wasm-bindgen-test` + `wasm-pack test --headless --chrome`). Layer 3: A Playwright-based E2E proof-of-concept that verifies the built app renders correctly, with CI/CD integration. See [research.md](research.md) for tool evaluation and setup details.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition) + TypeScript (Playwright E2E tests)
**Primary Dependencies**: `wasm-bindgen-test` 0.3 (WASM tests), Playwright (E2E tests), existing workspace deps (crux_core, leptos, web-sys)
**Storage**: N/A (tests exercise existing localStorage persistence — no new storage)
**Testing**: `cargo test` (unit), `wasm-pack test --headless --chrome` (WASM integration), `npx playwright test` (E2E)
**Target Platform**: wasm32-unknown-unknown (browser), GitHub Actions CI/CD
**Project Type**: Single workspace — tests added to existing `intrada-web` crate + new `e2e/` directory
**Performance Goals**: All new CI jobs complete within 5 minutes total (FR-017)
**Constraints**: No changes to application behaviour for testability (FR-015); structural changes (adding `lib.rs`, making save functions `pub`) are acceptable; existing tests must not break (SC-007)
**Scale/Scope**: ~15+ unit tests (SC-001), 3+ WASM tests (SC-002), 1 E2E smoke test (SC-004)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Code Quality

- **Clarity over cleverness**: PASS — Test code will be straightforward and self-documenting
- **Single Responsibility**: PASS — Each test file covers one concern (helpers, validation, persistence, E2E)
- **Consistent Style**: PASS — Follows existing `cargo fmt` + `cargo clippy` standards
- **No Dead Code**: PASS — All test code is actively executed
- **Explicit over Implicit**: PASS — Test setup/teardown is visible in each test
- **Type Safety**: PASS — Standard Rust type safety

### II. Testing Standards

- **Test Coverage**: PASS — This feature *adds* coverage to the only untested crate
- **Test Independence**: PASS — Unit tests are pure; WASM tests clear localStorage; E2E tests use fresh page loads
- **Meaningful Assertions**: PASS — Tests verify behaviour (function outputs, persistence round-trips, rendered content)
- **Fast Feedback**: PASS — Unit tests: seconds; WASM tests: ~1-2 min; E2E tests: ~1-2.5 min
- **Failure Clarity**: PASS — Standard assertion messages + Playwright trace on failure
- **Contract Tests**: N/A — No new API boundaries introduced

### III. User Experience Consistency

- N/A — This feature adds test infrastructure only; no UI changes

### IV. Performance Requirements

- **CI Time Budget**: PASS — Total new CI time ~2-4.5 minutes, within 5-minute budget (FR-017)
- **Resource Limits**: PASS — Tests run in ephemeral CI environments

**Gate Status**: PASS — No violations

## Project Structure

### Documentation (this feature)

```text
specs/013-web-testing/
├── plan.md              # This file
├── research.md          # E2E tool evaluation, testability analysis
├── data-model.md        # Test layer definitions
├── quickstart.md        # Developer guide for running tests
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/intrada-web/
├── src/
│   ├── lib.rs               # NEW: re-exports modules for integration tests
│   ├── main.rs              # MODIFIED: uses intrada_web:: imports for shared modules
│   ├── helpers.rs           # Pure functions (existing — add #[cfg(test)] mod tests)
│   ├── validation.rs        # Form validation (existing — add #[cfg(test)] mod tests)
│   └── core_bridge.rs       # localStorage bridge (existing — add #[cfg(test)] mod tests, make save fns pub)
├── tests/
│   └── wasm.rs              # NEW: WASM integration tests (wasm-bindgen-test)
└── Cargo.toml               # MODIFIED: add wasm-bindgen-test dev-dependency

e2e/                          # NEW: Playwright E2E test directory
├── package.json
├── playwright.config.ts
└── tests/
    └── smoke.spec.ts

.github/workflows/ci.yml     # MODIFIED: add wasm-test + e2e jobs
```

**Structure Decision**: Tests are added inline (`#[cfg(test)]` modules) for unit tests and in `tests/wasm.rs` for WASM integration tests, following Rust conventions. A `lib.rs` is added to expose modules for integration test imports (the crate was previously binary-only with `main.rs`). E2E tests live in a separate `e2e/` directory at the repo root since they use a different language (TypeScript) and toolchain (Node.js + Playwright).

## Complexity Tracking

> No Constitution Check violations — this section is intentionally empty.
