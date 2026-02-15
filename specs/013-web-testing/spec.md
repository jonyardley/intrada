# Feature Specification: Web UI Testing & E2E Test Infrastructure

**Feature Branch**: `013-web-testing`
**Created**: 2026-02-15
**Status**: Draft
**Input**: User description: "Add useful and meaningful rust tests to the UI and also do some research into the best tool for end-2-end tests I can run in CICD"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Rust Unit Tests for Web Shell Logic (Priority: P1)

As a developer, I want the web UI crate (`intrada-web`) to have meaningful Rust-level tests that verify core bridge effect processing, localStorage persistence logic, and helper functions, so that regressions in the web shell are caught before they reach a browser.

Currently `intrada-web` has 0 tests. The core crate has 87 tests and the CLI shell has 25 tests, but the web shell — which contains 16 components, 6 views, a localStorage persistence layer, and a Crux core bridge — has no automated test coverage at all.

**Why this priority**: The web shell is the primary user-facing interface but has zero test coverage. Adding Rust-level tests is the highest-value, lowest-friction improvement — it uses existing tooling (`cargo test`, `wasm-bindgen-test`) and catches logic errors without requiring browser infrastructure.

**Independent Test**: Run `cargo test -p intrada-web` and verify that all new tests pass, covering core bridge logic, localStorage persistence, and helper functions.

**Acceptance Scenarios**:

1. **Given** the web crate has no tests, **When** `cargo test -p intrada-web` is run, **Then** meaningful tests execute and pass covering the core bridge, persistence logic, and helper functions.
2. **Given** a developer changes the effect-handling logic in `core_bridge.rs`, **When** they run `cargo test`, **Then** any breaking change is caught by at least one failing test.
3. **Given** a developer modifies a component's event dispatch logic, **When** they run `cargo test`, **Then** incorrect event construction or effect handling is detected.

---

### User Story 2 - WASM Integration Tests in Headless Browser (Priority: P2)

As a developer, I want integration tests that run inside a real browser WASM environment, so that browser-specific behaviour (localStorage access, DOM APIs) is verified automatically.

Some web shell logic depends on browser APIs that are unavailable in a standard `cargo test` environment — localStorage read/write and `web_sys::window()`. These require `wasm-bindgen-test` running in a headless browser.

**Why this priority**: Browser-dependent code paths (localStorage persistence, window API access) cannot be tested with standard `cargo test`. WASM integration tests fill this gap using `wasm-pack test --headless`, catching issues that only manifest in a browser environment.

**Independent Test**: Run `wasm-pack test --headless --chrome` against the web crate and verify that localStorage round-trip tests pass in a headless browser.

**Acceptance Scenarios**:

1. **Given** the app writes session data to localStorage, **When** `wasm-pack test --headless --chrome` runs, **Then** the data round-trips correctly through serialisation and deserialisation.
2. **Given** the app loads library data from localStorage on first run, **When** no data exists in localStorage, **Then** stub data is seeded and persisted correctly.
3. **Given** a WASM integration test exists, **When** it runs in CI/CD, **Then** it completes in a headless browser without manual intervention.

---

### User Story 3 - E2E Testing Recommendation and Proof of Concept (Priority: P3)

As a developer, I want a researched recommendation for the best end-to-end testing tool that works with a Leptos CSR/WASM app in GitHub Actions CI/CD, along with a minimal proof-of-concept setup, so that full user-journey tests can be added incrementally.

**Why this priority**: E2E tests verify the complete user experience across the built application, but they require additional infrastructure (browser automation, build pipeline, test server). A researched recommendation with a working proof-of-concept establishes the foundation for future E2E coverage without blocking the higher-priority unit and integration tests.

**Independent Test**: A documented recommendation is produced comparing viable E2E tools, and at least one proof-of-concept E2E test runs successfully against the trunk-built app, both locally and in GitHub Actions.

**Acceptance Scenarios**:

1. **Given** the research is complete, **When** a developer reads the recommendation document, **Then** they understand which E2E tool to use, why it was chosen, and how to set it up.
2. **Given** a proof-of-concept E2E test exists, **When** `trunk build` produces the app and the test runs against it, **Then** it navigates to the home page and verifies that the library list renders.
3. **Given** the E2E proof-of-concept is integrated into CI/CD, **When** a pull request is opened, **Then** the E2E test runs automatically in GitHub Actions alongside existing checks.

---

### Edge Cases

- What happens when `wasm-pack test` cannot find a browser driver in CI? Tests should fail with a clear error message rather than silently skipping.
- How does the test suite handle localStorage being unavailable (e.g., in private browsing or restricted environments)? Tests should detect this and report it rather than producing false passes.
- What happens when trunk build output changes structure? E2E tests should not be coupled to specific file paths in the `dist/` directory.
- How are WASM integration tests isolated from each other when they share localStorage state? Each test should start with a clean state.
- What happens when the E2E test server port is already in use? The test setup should handle port conflicts gracefully.

## Requirements *(mandatory)*

### Functional Requirements

#### Rust Unit Tests (US1)

- **FR-001**: The web crate MUST have unit tests that verify core event processing produces the expected storage effects, by instantiating `Core<Intrada>`, sending events, and asserting that the returned `Vec<Effect>` contains the expected `StorageEffect` variants (SavePiece, SaveExercise, UpdatePiece, UpdateExercise, DeleteItem, SaveSession, UpdateSession, DeleteSession). These tests run with `cargo test` and do not require browser APIs.
- **FR-002**: The web crate MUST have unit tests that verify `Core<Intrada>::view()` produces correct `ViewModel` state after events are processed (items appear in list, error field set on validation failure).
- **FR-003**: The web crate MUST have unit tests for any standalone helper or validation functions in the web crate.
- **FR-004**: Tests MUST run with `cargo test` in the standard Rust test harness (not requiring a browser).
- **FR-005**: Tests MUST maintain isolation — one test's outcome must not depend on another test's side effects.

#### WASM Integration Tests (US2)

- **FR-006**: The web crate MUST have WASM integration tests using `wasm-bindgen-test` that verify localStorage read/write operations.
- **FR-007**: WASM integration tests MUST run in a headless browser via `wasm-pack test --headless --chrome`.
- **FR-008**: WASM integration tests MUST verify that data persisted to localStorage survives a read-back after write.
- **FR-009**: WASM integration tests MUST clean up localStorage state between tests to ensure isolation.
- **FR-010**: WASM integration tests MUST be runnable in the CI/CD pipeline.

#### E2E Testing (US3)

- **FR-011**: A research document MUST be produced comparing at least three E2E testing tools, evaluating each for compatibility with Leptos CSR/WASM apps, CI/CD support, community maturity, and setup complexity.
- **FR-012**: The research document MUST include a clear recommendation with justification.
- **FR-013**: At least one proof-of-concept E2E test MUST be implemented that verifies basic app rendering (page loads, primary content visible).
- **FR-014**: The proof-of-concept MUST include a CI/CD workflow configuration that runs the E2E test automatically on pull requests.
- **FR-015**: The E2E test setup MUST not require changes to the application behaviour — it tests the built output only. Structural changes for testability (e.g., adding `lib.rs` to expose modules for integration tests) are acceptable provided they do not alter runtime behaviour.

#### CI/CD Integration

- **FR-016**: All new test types MUST be integrated into the existing CI/CD pipeline.
- **FR-017**: New CI/CD jobs MUST not add more than 5 minutes to the total pipeline duration.
- **FR-018**: Test failures MUST produce clear, actionable output that helps developers diagnose the issue.

### Key Entities

- **Test Suite Layer**: A category of tests (unit, WASM integration, E2E) with distinct tooling, execution environment, and coverage scope.
- **Core Bridge**: The web shell's interface to the Crux core, handling effect dispatch, localStorage persistence, and view-model updates — the primary target for unit tests.
- **E2E Test Fixture**: A self-contained test that builds the application, serves it, drives a browser against it, and verifies user-visible outcomes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The web crate (`intrada-web`) has at least 15 meaningful passing tests covering core bridge logic, helper functions, and persistence behaviour.
- **SC-002**: At least 3 WASM integration tests pass in a headless browser, verifying localStorage round-trips for library and session data.
- **SC-003**: A research document evaluates at least 3 E2E testing tools with a clear recommendation and justification.
- **SC-004**: At least 1 proof-of-concept E2E test runs successfully both locally and in GitHub Actions CI/CD.
- **SC-005**: All new tests are integrated into the CI/CD pipeline and run automatically on every pull request.
- **SC-006**: The total CI/CD pipeline duration increases by no more than 5 minutes with all new test types enabled.
- **SC-007**: Zero existing tests are broken by the addition of new test infrastructure.

## Assumptions

- The project continues to use GitHub Actions for CI/CD.
- The `wasm-pack` toolchain is available or can be installed in CI/CD runners.
- Headless Chrome is available on GitHub Actions Ubuntu runners (it is pre-installed).
- The trunk build process does not require special secrets or environment variables.
- Leptos 0.8.x CSR does not provide built-in component test utilities — tests will focus on the shell logic (core bridge, persistence, helpers) rather than Leptos component rendering.
- E2E tests will use a static file server to serve the trunk-built output, not a live development server.

## Dependencies

- Existing CI/CD pipeline in `.github/workflows/ci.yml`
- `wasm-bindgen-test` crate for WASM integration tests
- `wasm-pack` CLI for running WASM tests in headless browsers
- External E2E tool (to be determined by research) for proof-of-concept
- Trunk build must succeed before E2E tests can run (existing CI job)

## Out of Scope

- Full E2E test coverage for all user journeys — only a proof-of-concept is included.
- Visual regression testing or screenshot comparison.
- Performance or load testing.
- Testing on mobile browsers or non-Chromium browsers (initial scope is headless Chrome only).
- Changes to the application source code for testability — tests adapt to the existing code.
