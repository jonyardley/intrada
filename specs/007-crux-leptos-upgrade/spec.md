# Feature Specification: Dependency Upgrade — Crux & Leptos

**Feature Branch**: `007-crux-leptos-upgrade`
**Created**: 2026-02-14
**Status**: Draft
**Input**: User description: "Upgrade to latest Crux and Leptos making sure to use the correct established patterns in the upgraded dependencies"

## Clarifications

### Session 2026-02-14

- Q: If upgraded dependencies change an API signature used in tests, should tests remain untouched (revert upgrade) or be updated to use new APIs while verifying the same behaviour? → A: Tests may be updated to accommodate API changes, but must verify the same behaviour
- Q: NFR-003 "startup time MUST NOT regress noticeably" is vague — drop it and rely on the 20% WASM size cap as a proxy, replace with a concrete threshold, or keep as-is? → A: Drop NFR-003; the 20% binary size cap is sufficient as a startup performance proxy

## User Scenarios & Testing *(mandatory)*

### User Story 1 — All Existing Functionality Continues Working After Upgrade (Priority: P1)

A developer upgrades the application's core framework dependencies to their latest stable versions. After the upgrade, every feature that worked before — browsing the library, viewing item details, adding/editing/deleting pieces and exercises, tagging, and searching — continues to work identically. No user-facing behaviour changes occur.

**Why this priority**: This is the fundamental guarantee of a dependency upgrade. If existing functionality breaks, the upgrade is a regression, not an improvement. Every other story depends on this one.

**Independent Test**: Run the full automated test suite (unit tests, clippy, build, WASM build) and manually verify the web app renders and responds to all user actions exactly as before.

**Acceptance Scenarios**:

1. **Given** the upgraded dependencies are in place, **When** the full test suite runs, **Then** all tests pass (tests may be updated to accommodate API changes but must verify the same behaviour as before)
2. **Given** the upgraded dependencies are in place, **When** the workspace is built with clippy in strict mode, **Then** zero warnings are reported
3. **Given** the upgraded web app is served locally, **When** a user navigates through all views (list, add, edit, detail, delete, search), **Then** every view renders correctly and all interactions work identically to the pre-upgrade version
4. **Given** the upgraded web app is built for production, **When** the WASM binary is compiled, **Then** the build succeeds without errors

---

### User Story 2 — Codebase Adopts Current Idiomatic Patterns (Priority: P2)

Where the upgraded dependencies introduce new recommended patterns, preferred APIs, or deprecate old approaches, the codebase is updated to follow the current idiomatic conventions. This ensures the project stays maintainable, avoids deprecation warnings in future releases, and aligns with official documentation and community examples.

**Why this priority**: Adopting idiomatic patterns during the upgrade prevents accumulation of technical debt. It is more efficient to migrate patterns during the version bump than to defer them.

**Independent Test**: Review all updated code against the latest official documentation for each dependency. Verify no deprecated APIs are used and patterns match current recommendations.

**Acceptance Scenarios**:

1. **Given** the upgraded codebase, **When** checked for deprecated API usage, **Then** no deprecated methods, traits, or types are in use
2. **Given** the upgraded codebase, **When** compared against current official examples and documentation, **Then** core patterns (component definitions, signal usage, effect handling, shell integration) follow the recommended approach
3. **Given** the upgraded codebase, **When** all compiler warnings (including deprecation warnings) are checked, **Then** zero warnings are emitted

---

### User Story 3 — Dependency Versions Are Current and Compatible (Priority: P3)

All workspace dependency version specifiers are updated to target the latest stable releases. Transitive dependencies resolve cleanly without conflicts, duplicate crate versions, or yanked versions.

**Why this priority**: Keeping version specifiers current ensures the project benefits from bug fixes, performance improvements, and security patches. It also reduces friction for future upgrades.

**Independent Test**: Inspect the resolved dependency tree for conflicts, duplicates, and outdated versions. Verify all version specifiers in manifests target current stable releases.

**Acceptance Scenarios**:

1. **Given** the updated manifests, **When** dependencies are resolved, **Then** no conflicting or duplicate major versions of the same crate appear in the dependency tree
2. **Given** the updated manifests, **When** a clean build is performed from scratch, **Then** all dependencies download and compile without errors
3. **Given** the updated manifests, **When** the core framework dependencies are inspected, **Then** they reference the latest stable versions available at the time of the upgrade

---

### Edge Cases

- What happens if a dependency introduces a subtle behavioural change that does not cause a compile error but changes runtime behaviour (e.g., signal timing, rendering order)?
- How does the system handle any changes to the reactive signal model that might affect two-way data binding in form inputs?
- What happens if the WASM target compilation has different requirements than the native target after the upgrade?
- How does the upgrade affect the core bridge layer that translates between the pure-core architecture and the UI framework?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The upgrade MUST update the UI framework dependency from version 0.7 to the latest stable 0.8.x release
- **FR-002**: The upgrade MUST verify and, if necessary, update the core architecture framework to the latest available release
- **FR-003**: All existing automated tests (currently 82) MUST pass after the upgrade. Tests may be updated to accommodate API changes in upgraded dependencies, but each updated test MUST verify the same behaviour as before the upgrade
- **FR-004**: The upgrade MUST produce zero compiler warnings across the entire workspace, including deprecation warnings
- **FR-005**: The upgrade MUST produce zero linting warnings in strict mode across the entire workspace
- **FR-006**: The WASM production build MUST succeed after the upgrade
- **FR-007**: All component definitions MUST follow the patterns recommended by the latest version of the UI framework
- **FR-008**: All reactive signal usage (read, write, derived) MUST follow the patterns recommended by the latest version of the UI framework
- **FR-009**: The shell/bridge integration between the pure core and the UI framework MUST follow the patterns recommended by the latest versions of both frameworks
- **FR-010**: Form handling (two-way binding, validation display, event handling) MUST continue to function identically after the upgrade
- **FR-011**: All accessibility attributes (ARIA labels, roles, keyboard navigation) MUST be preserved through the upgrade
- **FR-012**: Any secondary dependencies that need version bumps to maintain compatibility MUST be updated as part of this work

### Non-Functional Requirements

- **NFR-001**: The upgrade MUST NOT introduce any new runtime dependencies
- **NFR-002**: The WASM binary size MUST NOT increase by more than 20% compared to the pre-upgrade build

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of tests pass after the upgrade, verifying the same behaviours as before (test code may be updated to accommodate API changes)
- **SC-002**: Zero compiler warnings and zero linting warnings across the full workspace
- **SC-003**: WASM build completes successfully
- **SC-004**: Formatting check passes (matching CI gate)
- **SC-005**: All views render and respond to user interactions identically to the pre-upgrade version (verified via manual smoke test)
- **SC-006**: No deprecated API usage remains in the codebase after upgrade
- **SC-007**: Dependency tree resolves cleanly with no conflicting versions of core framework crates

## Scope

### In Scope

- Updating core framework dependency version specifiers
- Updating UI framework dependency from 0.7 to 0.8.x
- Updating any secondary dependencies required for compatibility
- Migrating code patterns to match current idiomatic usage
- Verifying all existing tests pass
- Verifying WASM build succeeds
- Manual smoke test of all web views

### Out of Scope

- Adding new features or functionality
- Changing the application architecture
- Adding new dependencies not required by the upgrade
- Performance optimisation beyond what the upgraded dependencies provide
- Upgrading build tooling (e.g., trunk) unless required for compatibility

## Assumptions

- The latest stable versions of both frameworks are API-compatible with the project's current architecture (pure core with UI shell)
- The CSR (client-side rendering) mode used by the web app is fully supported in the latest UI framework version
- The project's custom effect/command patterns remain compatible with the latest core framework
- The WASM compilation target continues to be supported by all upgraded dependencies
- The existing CI pipeline (formatting, clippy, tests, WASM build) serves as the primary automated validation gate
