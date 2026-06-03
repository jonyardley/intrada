# Tasks: CI/CD Quality Gates

**Input**: Design documents from `/specs/002-ci-cd/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, quickstart.md

**Tests**: Not explicitly requested. Verification is manual (open PRs and check results per quickstart.md).

**Organization**: Tasks are grouped by user story. Since all work targets a single file (`.github/workflows/ci.yml`), phases are sequential rather than parallel.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Create directory structure and workflow scaffold

- [x] T001 Create `.github/workflows/` directory at repository root
- [x] T002 Create workflow scaffold in `.github/workflows/ci.yml` with workflow name `CI` and empty jobs map

**Checkpoint**: Empty workflow file exists at correct path

---

## Phase 2: User Story 1 & 2 - Quality Checks on PRs and Main (Priority: P1) 🎯 MVP

**Goal**: Pipeline runs tests, clippy, and formatting checks automatically on PRs against main and pushes to main, with each check reporting independently.

**Independent Test**: Open a PR against main — three checks (test, clippy, fmt) should appear and all pass. Push directly to main — same three checks should run.

**Note**: US1 (PR triggers) and US2 (push triggers) are combined because they share the same workflow triggers and job definitions. The `on:` block satisfies both stories simultaneously.

### Implementation

- [x] T003 [US1] Add trigger configuration for pull requests against main and pushes to main in `.github/workflows/ci.yml`
- [x] T004 [US1] Add `test` job: checkout, `dtolnay/rust-toolchain@stable`, `cargo test` in `.github/workflows/ci.yml`
- [x] T005 [US1] Add `clippy` job: checkout, `dtolnay/rust-toolchain@stable` with clippy component, `cargo clippy -- -D warnings` in `.github/workflows/ci.yml`
- [x] T006 [US1] Add `fmt` job: checkout, `dtolnay/rust-toolchain@stable` with rustfmt component, `cargo fmt --all -- --check` in `.github/workflows/ci.yml`

**Checkpoint**: Push branch and open PR against main. Three independent checks should appear: test ✅, clippy ✅, fmt ✅. Verify push to main also triggers the same checks.

---

## Phase 3: User Story 3 - Dependency Caching (Priority: P2)

**Goal**: Add dependency caching to compilation jobs so repeat runs complete faster.

**Independent Test**: Run pipeline twice on the same branch. Second run should show cache hits and complete faster than the first.

### Implementation

- [x] T007 [US3] Add `Swatinem/rust-cache@v2` step to `test` job after toolchain setup in `.github/workflows/ci.yml`
- [x] T008 [US3] Add `Swatinem/rust-cache@v2` step to `clippy` job after toolchain setup in `.github/workflows/ci.yml`

**Checkpoint**: Push two commits to the same PR branch. Second pipeline run should show "Restored cache" in rust-cache step output.

---

## Phase 4: Polish & Verification

**Purpose**: Final validation and documentation

- [x] T009 Verify all three jobs run on `ubuntu-latest` and `cargo test` succeeds (confirms native build tools for rusqlite bundled) in `.github/workflows/ci.yml`
- [x] T010 Run quickstart.md validation: push branch, open PR, verify three independent checks pass
- [x] T011 Commit and push `.github/workflows/ci.yml` to feature branch

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **US1 & US2 (Phase 2)**: Depends on Phase 1 (directory and scaffold exist)
- **US3 (Phase 3)**: Depends on Phase 2 (jobs must exist before adding cache steps)
- **Polish (Phase 4)**: Depends on Phase 3

### Within Each Phase

All tasks are sequential — they modify the same file (`.github/workflows/ci.yml`).

### Parallel Opportunities

Limited due to single-file feature:
- T004, T005, T006 could conceptually be parallel (separate job definitions), but since they're all in the same YAML file, sequential is safer
- Phases themselves are strictly sequential

---

## Implementation Strategy

### MVP First (Phase 1 + Phase 2)

1. Complete Phase 1: Create workflow directory and scaffold
2. Complete Phase 2: Add triggers + three quality gate jobs
3. **STOP and VALIDATE**: Push and open PR — verify all three checks run independently
4. This alone delivers full value for US1 and US2

### Full Delivery

1. Complete MVP (Phases 1-2)
2. Add caching (Phase 3) — improves speed, no functional change
3. Verify and commit (Phase 4)

---

## Notes

- All tasks target a single file: `.github/workflows/ci.yml`
- No caching on the `fmt` job — it doesn't compile, so caching adds overhead with no benefit
- `clippy` uses `-D warnings` to treat warnings as errors, preventing warning debt
- `cargo fmt --all -- --check` checks all workspace crates without modifying files
- The `test` and `clippy` jobs need `Swatinem/rust-cache@v2`; `fmt` does not
