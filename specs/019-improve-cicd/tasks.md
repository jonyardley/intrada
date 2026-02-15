# Tasks: Improve CI/CD Pipeline

**Input**: Design documents from `/specs/019-improve-cicd/`
**Prerequisites**: plan.md (required), spec.md (required), research.md

**Tests**: No test tasks — this feature modifies CI/CD configuration only and is verified by pipeline behaviour (see quickstart.md).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: No setup needed — this feature modifies existing files only.

*(No tasks in this phase)*

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Switch the WASM build to release mode — this change is shared by both user stories (US1 deploy job needs a release artifact, US2 needs a single build).

- [ ] T001 Change `trunk build` to `trunk build --release` in the `wasm-build` job's "Build WASM" step in `.github/workflows/ci.yml`

**Checkpoint**: CI still passes on PRs with the release build. E2E tests still work with the release artifact.

---

## Phase 3: User Story 1 - Safe Deployment Gating (Priority: P1)

**Goal**: Deployment only happens after all CI checks pass. No broken code reaches production.

**Independent Test**: Merge a PR to main — deploy job should appear, wait for all CI jobs, then run. On a CI failure, deploy should be skipped.

### Implementation for User Story 1

- [ ] T002 [US1] Add `deploy` job to `.github/workflows/ci.yml` with `needs: [test, clippy, fmt, wasm-build, wasm-test, e2e]` and `if: github.event_name == 'push' && github.ref == 'refs/heads/main'`. Job steps: (1) `actions/checkout@v4`, (2) `actions/download-artifact@v4` with `name: web-dist` and `path: crates/intrada-web/dist`, (3) `cloudflare/wrangler-action@v3` with `apiToken` and `accountId` secrets.
- [ ] T003 [US1] Delete `.github/workflows/deploy.yml` entirely — deploy is now handled by the new job in `ci.yml`.

**Checkpoint**: Push to main triggers CI → all jobs pass → deploy runs. Deploy is skipped if any CI job fails. PRs show no deploy job.

---

## Phase 4: User Story 2 - Efficient Pipeline (Priority: P2)

**Goal**: Eliminate redundant work — WASM is built once and reused for E2E tests and deployment.

**Independent Test**: In a successful main pipeline run, verify only one WASM build step exists across all jobs. The deploy job has no Rust toolchain, no trunk, no Tailwind CLI.

### Implementation for User Story 2

*(Already achieved by T001 + T002 + T003 — the release build in `wasm-build` is uploaded as an artifact and reused by both `e2e` and `deploy`. No additional tasks needed.)*

**Checkpoint**: The `web-dist` artifact is built once by `wasm-build`, downloaded by `e2e`, and downloaded by `deploy`. Total compute minutes decrease.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Verification and cleanup

- [ ] T004 Run `cargo test` to confirm all existing tests still pass (no code changes, sanity check)
- [ ] T005 Run `cargo clippy -- -D warnings` to confirm zero warnings
- [ ] T006 Run quickstart.md verification steps V1-V6 after merge to main

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 2)**: No dependencies — can start immediately
- **User Story 1 (Phase 3)**: Depends on T001 (release build must be in place before deploy job references the artifact)
- **User Story 2 (Phase 4)**: Achieved by T001 + T002 + T003 — no separate tasks
- **Polish (Phase 5)**: Depends on T001-T003 being complete

### Task Dependencies

```
T001 (release build) ──→ T002 (add deploy job) ──→ T003 (delete deploy.yml)
                                                         │
                                                         ▼
                                                   T004, T005, T006 (verification)
```

### Parallel Opportunities

- T004 and T005 can run in parallel (different tools, no file conflicts)
- T002 and T003 are sequential (T003 depends on T002 being correct before deleting the old file)

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001: Switch to release build
2. Complete T002: Add deploy job with gating
3. Complete T003: Delete old deploy.yml
4. **STOP and VALIDATE**: Merge PR, verify deploy is gated on CI

### Incremental Delivery

This is a small feature — all 3 implementation tasks should be done in a single PR. User Story 2 (efficiency) is a natural side effect of User Story 1 (gating), so both are delivered together.

---

## Notes

- Total implementation tasks: 3 (T001-T003)
- Total verification tasks: 3 (T004-T006)
- All changes are in `.github/workflows/` — no Rust code modified
- The feature is small enough to be a single commit or PR
- Verification step T006 (quickstart) can only be fully validated after merge to main with secrets configured
