# Tasks: Remove CLI Shell

**Input**: Design documents from `/specs/014-remove-cli/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, quickstart.md

**Tests**: No test tasks — this is a removal feature. Validation is via `cargo build`, `cargo test`, `cargo clippy`, and grep checks (covered in the final validation phase).

**Organization**: Tasks are grouped by user story. US1 must complete before US2 (dependency cleanup requires the crate to be gone first). US3 (documentation) can run in parallel with US2.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: User Story 1 — Remove CLI Crate from Workspace (Priority: P1) 🎯 MVP

**Goal**: Delete the `intrada-cli` crate entirely so the workspace contains only `intrada-core` and `intrada-web`.

**Independent Test**: `crates/intrada-cli/` no longer exists; `cargo build`, `cargo test`, and `cargo clippy` all succeed.

### Implementation for User Story 1

- [x] T001 [US1] Delete the entire `crates/intrada-cli/` directory and all its contents (src/main.rs, src/shell.rs, src/storage.rs, src/display.rs, Cargo.toml)
- [x] T002 [US1] Run `cargo build` to verify the workspace compiles with only `intrada-core` and `intrada-web`
- [x] T003 [US1] Run `cargo test` to verify all remaining tests pass (expected: 87 core + 36 web)
- [x] T004 [US1] Run `cargo clippy -- -D warnings` to verify no warnings

**Checkpoint**: CLI crate removed. Workspace builds and tests pass with two crates.

---

## Phase 2: User Story 2 — Remove CLI-Only Workspace Dependencies (Priority: P2)

**Goal**: Remove `clap`, `anyhow`, and `dirs` from root `Cargo.toml` workspace dependencies since no remaining crate uses them.

**Independent Test**: Root `Cargo.toml` contains only 6 workspace dependencies (crux_core, serde, serde_json, ulid, chrono, thiserror); `cargo build` succeeds.

**Depends on**: Phase 1 (US1) — crate must be deleted first so Cargo doesn't complain about missing workspace deps.

### Implementation for User Story 2

- [x] T005 [US2] Remove `clap = { version = "4.5", features = ["derive"] }` from `[workspace.dependencies]` in `Cargo.toml`
- [x] T006 [US2] Remove `anyhow = "1"` from `[workspace.dependencies]` in `Cargo.toml`
- [x] T007 [US2] Remove `dirs = "5"` from `[workspace.dependencies]` in `Cargo.toml`
- [x] T008 [US2] Run `cargo build` to verify workspace compiles with cleaned-up dependencies

**Checkpoint**: Workspace dependencies are minimal — only deps used by remaining crates.

---

## Phase 3: User Story 3 — Update Project Documentation (Priority: P3)

**Goal**: Update README.md and CLAUDE.md to reflect the two-crate workspace with no CLI references.

**Independent Test**: `grep -i "intrada-cli\|CLI shell\|CLI usage\|cargo run --bin intrada" README.md` returns no matches; `grep "intrada-cli" CLAUDE.md` returns no matches.

**Can run in parallel with**: Phase 2 (US2) — documentation changes are in different files.

### Implementation for User Story 3

- [x] T009 [P] [US3] Update README.md: Remove the "CLI usage" section (lines 37-79) entirely
- [x] T010 [P] [US3] Update README.md: Rewrite the project description to describe only the web app (remove "Ships as a CLI" phrasing, remove SQLite reference)
- [x] T011 [US3] Update README.md: Rewrite the "Project structure" section to remove `intrada-cli/` and its file listing; keep only `intrada-core/` and `intrada-web/`
- [x] T012 [US3] Update README.md: Rewrite the "Architecture" section to describe only the core library and web shell (remove CLI shell paragraph)
- [x] T013 [US3] Update README.md: Rewrite the "Data storage" section to describe only web localStorage persistence (remove CLI SQLite/JSON file references)
- [x] T014 [P] [US3] Update CLAUDE.md: Remove `intrada-cli/` from the project structure section and update to show only `intrada-core/` and `intrada-web/`
- [x] T015 [P] [US3] Update CLAUDE.md: Remove CLI-specific entries from the "Active Technologies" section (entries referencing clap, CLI, SQLite, JSON file persistence for CLI)

**Checkpoint**: All living documentation accurately reflects the core + web only workspace.

---

## Phase 4: Validation & Polish

**Purpose**: Final verification that all removal criteria are met.

- [x] T016 Run `cargo fmt --all -- --check` to verify formatting
- [x] T017 Run quickstart.md verification steps: confirm `crates/intrada-cli/` does not exist, only 2 crates remain, no CLI-only deps in Cargo.toml, no CLI references in README.md or CLAUDE.md
- [x] T018 Verify historical specs (specs/001-* through specs/013-*) were NOT modified (FR-008)
- [x] T019 Verify `intrada-core` crate was NOT modified (FR-009)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (US1)**: No dependencies — start immediately
- **Phase 2 (US2)**: Depends on Phase 1 — must delete crate before removing its workspace deps
- **Phase 3 (US3)**: No dependency on Phase 2 — can run in parallel (different files)
- **Phase 4 (Validation)**: Depends on all previous phases

### User Story Dependencies

- **US1 (P1)**: No dependencies — this is the foundational removal
- **US2 (P2)**: Depends on US1 — Cargo will error if workspace deps are removed while a crate still references them
- **US3 (P3)**: Independent of US2 — documentation changes don't affect build

### Parallel Opportunities

- T005, T006, T007 can be combined into a single edit of `Cargo.toml` (same file, logically grouped)
- T009, T010 (README changes) can be done in parallel with T014, T015 (CLAUDE.md changes) — different files
- US2 and US3 can run in parallel after US1 completes

---

## Parallel Example: After US1 Completes

```text
# These can run in parallel:
Stream A (US2): T005-T008 — Clean up Cargo.toml dependencies
Stream B (US3): T009-T015 — Update README.md and CLAUDE.md
```

---

## Implementation Strategy

### Sequential Execution (Single Developer)

1. Complete Phase 1 (US1): Delete CLI crate → verify build
2. Complete Phase 2 (US2): Clean workspace deps → verify build
3. Complete Phase 3 (US3): Update README + CLAUDE.md
4. Complete Phase 4: Run all validation checks
5. Commit and push

### MVP First

Phase 1 alone delivers the core value — the CLI is gone and the workspace builds. Phases 2-3 are cleanup and polish.

---

## Notes

- This is a removal feature — all tasks are deletions or edits, no new code
- FR-008 explicitly prohibits modifying historical specs — verify this in Phase 4
- FR-009 explicitly prohibits modifying intrada-core — verify this in Phase 4
- Total: 19 tasks (4 US1 + 4 US2 + 7 US3 + 4 validation)
