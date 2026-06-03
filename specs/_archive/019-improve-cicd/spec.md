# Feature Specification: Improve CI/CD Pipeline

**Feature Branch**: `019-improve-cicd`
**Created**: 2026-02-15
**Status**: Draft
**Input**: User description: "Improve CICD - Make sure deployment happens after checks have passed and ensure that it is as efficient as possible"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Safe Deployment Gating (Priority: P1)

As a developer, when I merge a pull request to main, I want the production deployment to only happen after all quality checks (tests, linting, formatting, build, end-to-end tests) have passed, so that broken code is never deployed to the live site.

**Why this priority**: Deploying untested code to production is the highest-risk problem. Currently the deploy workflow runs independently of CI — a failing test suite does not prevent deployment.

**Independent Test**: Can be verified by introducing a deliberate test failure on main and confirming that deployment is blocked until all checks pass.

**Acceptance Scenarios**:

1. **Given** a merge to main where all CI checks pass, **When** the CI pipeline completes successfully, **Then** the deployment to the hosting provider starts automatically.
2. **Given** a merge to main where any CI check fails (tests, lint, formatting, build, or E2E), **When** the CI pipeline reports a failure, **Then** deployment does not occur and the developer is notified of the failure.
3. **Given** a pull request targeting main, **When** CI checks are running, **Then** no deployment is triggered (deployment only happens on push to main after merge).

---

### User Story 2 - Efficient Pipeline (Priority: P2)

As a developer, I want the CI/CD pipeline to complete as quickly as possible by eliminating redundant work, so that I get fast feedback on pull requests and deployments happen promptly after merge.

**Why this priority**: The current pipeline duplicates significant work — the deploy workflow rebuilds the entire WASM application from scratch even though CI already builds it. Reducing total build time saves compute costs and speeds up the feedback loop.

**Independent Test**: Can be verified by comparing total pipeline duration and total compute minutes before and after the improvement.

**Acceptance Scenarios**:

1. **Given** a push to main, **When** the full CI and deploy pipeline runs, **Then** the WASM application is built only once (not separately in CI and deploy).
2. **Given** a pull request, **When** CI runs, **Then** all independent checks (tests, lint, formatting, WASM tests) start simultaneously without waiting for each other.
3. **Given** repeated pipeline runs, **When** dependencies have not changed, **Then** build caches are used effectively to reduce redundant compilation time.

---

### Edge Cases

- What happens when the deploy step fails after CI passes (e.g., hosting provider outage)? The pipeline should report the deployment failure clearly without re-running CI checks.
- What happens when a CI check is flaky and passes on retry? The deployment should proceed once all checks show a passing status.
- What happens if the hosting provider credentials are missing or expired? The deployment step should fail with a clear error message, not a cryptic timeout.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Deployment MUST only begin after all quality checks have completed successfully (unit tests, lint checks, formatting checks, WASM build, WASM tests, and end-to-end tests).
- **FR-002**: The WASM application MUST be built only once per pipeline run and the resulting build artifact MUST be reused for both end-to-end testing and deployment.
- **FR-003**: All independent CI checks (unit tests, lint, formatting, WASM tests) MUST run in parallel to minimise wall-clock time.
- **FR-004**: The deployment step MUST use the release-optimised WASM build (smaller binary size) rather than a debug build.
- **FR-005**: The pipeline MUST continue to run all CI checks on pull requests without triggering deployment.
- **FR-006**: If any CI check fails, the deployment MUST be skipped entirely for that pipeline run.
- **FR-007**: The pipeline MUST use build caching to avoid redundant compilation of unchanged dependencies.
- **FR-008**: The pipeline MUST report clear status for each stage (checks, build, deploy) so developers can quickly identify where failures occur.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Deployment never occurs when any quality check has failed (zero broken deploys).
- **SC-002**: The WASM application is built exactly once per pipeline run, eliminating the current duplicate build.
- **SC-003**: Total compute time (sum of all job minutes) for a full main-branch pipeline run decreases compared to the current setup.
- **SC-004**: Pull request feedback time remains the same or improves (no regression from pipeline changes).
- **SC-005**: All existing CI checks continue to pass with no changes to test coverage or quality gates.

### Assumptions

- The hosting provider (Cloudflare Workers) supports deployment from a pre-built artifact rather than requiring a fresh build.
- Build caching provided by the CI platform is sufficient — no external caching infrastructure is needed.
- The current set of quality checks (unit tests, clippy, fmt, WASM build, WASM tests, E2E tests) is the correct and complete set — no new checks are being added in this feature.
