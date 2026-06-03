# Feature Specification: Remove CLI Shell

**Feature Branch**: `014-remove-cli`
**Created**: 2026-02-15
**Status**: Draft
**Input**: User description: "Remove the CLI as I no longer need it and want to focus on the web and core. Remove references and update documentation"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Remove CLI Crate from Workspace (Priority: P1)

As a developer, I want the CLI shell crate (`intrada-cli`) completely removed from the workspace so that the project only contains the core library and web shell, reducing maintenance overhead and build times.

**Why this priority**: The CLI crate is the primary artefact being removed. Nothing else can proceed until it's gone, and its removal delivers the core value of a leaner, focused codebase.

**Independent Test**: Can be verified by confirming the `crates/intrada-cli/` directory no longer exists, `cargo build` succeeds without it, and all remaining tests pass.

**Acceptance Scenarios**:

1. **Given** the workspace contains `intrada-cli`, **When** the CLI crate is removed, **Then** `crates/intrada-cli/` no longer exists and `cargo build` compiles successfully with only `intrada-core` and `intrada-web` as workspace members.
2. **Given** the CLI crate is removed, **When** `cargo test` is run, **Then** all remaining tests (core + web) pass without errors.
3. **Given** the CLI crate is removed, **When** `cargo clippy` is run, **Then** no warnings are produced.

---

### User Story 2 - Remove CLI-Only Workspace Dependencies (Priority: P2)

As a developer, I want workspace-level dependencies that were only used by the CLI removed from `Cargo.toml` so that the dependency tree stays clean and minimal.

**Why this priority**: Stale workspace dependencies create confusion about what the project actually needs. This should happen immediately after the crate removal to keep the workspace consistent.

**Independent Test**: Can be verified by checking the root `Cargo.toml` no longer declares dependencies unused by `intrada-core` or `intrada-web`, and `cargo build` still succeeds.

**Acceptance Scenarios**:

1. **Given** the CLI crate has been removed, **When** I inspect `Cargo.toml`, **Then** workspace dependencies only used by the CLI (such as `clap`, `anyhow`, `dirs`) are no longer listed.
2. **Given** CLI-only dependencies are removed, **When** `cargo build` is run, **Then** all remaining crates compile successfully.

---

### User Story 3 - Update Project Documentation (Priority: P3)

As a developer (or contributor), I want the README and development guidelines to accurately reflect the current project scope — core library and web app only — so that there is no confusion about a non-existent CLI.

**Why this priority**: Outdated documentation misleads new contributors and wastes time. Updating it completes the removal and ensures project artefacts are internally consistent.

**Independent Test**: Can be verified by reading the README and CLAUDE.md and confirming there are no mentions of CLI commands, CLI shell architecture, CLI-specific storage, or instructions for running the CLI binary.

**Acceptance Scenarios**:

1. **Given** the CLI has been removed, **When** I read the project README, **Then** there are no references to CLI usage, CLI commands, or the `intrada-cli` crate.
2. **Given** the CLI has been removed, **When** I read CLAUDE.md, **Then** the project structure, commands, and active technologies sections reflect only `intrada-core` and `intrada-web`.
3. **Given** the CLI has been removed, **When** I read the README, **Then** the architecture section describes only the core library and web shell.
4. **Given** the CLI has been removed, **When** I read the README, **Then** the data storage section describes only the web app's localStorage persistence (no references to SQLite or JSON file storage).

---

### Edge Cases

- What happens if any core module references CLI-specific types or patterns? The core should remain unchanged since it has no dependency on the CLI.
- What happens to CI jobs that implicitly run CLI tests? `cargo test` and `cargo clippy` will automatically skip CLI tests once the crate is removed, as they operate on workspace members.
- What if historical spec documents reference the CLI? Historical specs (001, 011, 012, etc.) are design artefacts and should be preserved as-is — they document what was built at the time. Only living documentation (README, CLAUDE.md) needs updating.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `crates/intrada-cli/` directory and all its contents MUST be removed from the repository.
- **FR-002**: The root `Cargo.toml` MUST continue to use `members = ["crates/*"]` and compile successfully with only `intrada-core` and `intrada-web` as workspace members.
- **FR-003**: Workspace dependencies in the root `Cargo.toml` that are exclusively used by `intrada-cli` MUST be removed. Specifically: `clap`, `anyhow`, and `dirs`.
- **FR-004**: The project README MUST be updated to remove all CLI-related content: CLI usage section, CLI references in the project structure, CLI architecture description, and CLI data storage description.
- **FR-005**: The README MUST accurately describe the current architecture (core + web shell only) and current data storage approach (localStorage).
- **FR-006**: The development guidelines file (CLAUDE.md) MUST be updated to remove CLI references from the project structure section and any CLI-specific technology or command references.
- **FR-007**: The CI/CD pipeline MUST continue to pass after all changes, with `cargo test`, `cargo clippy`, `cargo fmt`, WASM build, WASM tests, and E2E tests all succeeding.
- **FR-008**: Historical specification documents (in `specs/`) MUST NOT be modified, as they are point-in-time design records.
- **FR-009**: The `intrada-core` crate MUST NOT be modified, as it has no dependency on the CLI.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The project compiles and all tests pass with zero CLI-related code present — no files remain in `crates/intrada-cli/`.
- **SC-002**: The root `Cargo.toml` contains no dependencies unused by the remaining workspace members.
- **SC-003**: The README contains zero references to CLI usage, CLI commands, `intrada-cli`, or CLI-specific storage mechanisms.
- **SC-004**: CLAUDE.md accurately reflects the two-crate workspace (`intrada-core`, `intrada-web`) with no CLI references in the project structure or commands sections.
- **SC-005**: The full CI pipeline (test, clippy, fmt, wasm-build, wasm-test, e2e) passes without modification to the workflow file.

### Assumptions

- The `crates/*` glob pattern in the workspace `members` field will automatically exclude the removed CLI crate directory without requiring any Cargo.toml changes beyond dependency cleanup.
- No other crate in the workspace depends on `intrada-cli`.
- The `thiserror` workspace dependency is used by `intrada-core` and should be retained.
- The `serde`, `serde_json`, `ulid`, and `chrono` workspace dependencies are shared between core and web and should be retained.
