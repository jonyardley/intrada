# Implementation Plan: Improve CI/CD Pipeline

**Branch**: `019-improve-cicd` | **Date**: 2026-02-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/019-improve-cicd/spec.md`

## Summary

The CI (`ci.yml`) and deploy (`deploy.yml`) workflows currently run independently on push to main — a failing test suite does not prevent deployment. The deploy workflow also duplicates the entire WASM build. This plan merges both into a single workflow where deployment is gated on all CI checks passing, and the WASM artifact is built once and reused for E2E tests and deployment.

## Technical Context

**Language/Version**: GitHub Actions YAML (no Rust code changes)
**Primary Dependencies**: GitHub Actions (checkout@v4, upload-artifact@v4, download-artifact@v4, rust-toolchain, rust-cache@v2, trunk-action, wrangler-action@v3)
**Storage**: N/A
**Testing**: Verified by pipeline behaviour (no unit tests for CI config)
**Target Platform**: GitHub Actions runners (ubuntu-latest)
**Project Type**: CI/CD infrastructure
**Performance Goals**: Eliminate duplicate WASM build; deploy job runs in <60 seconds (artifact download + wrangler deploy only)
**Constraints**: Must not break existing CI checks or deployment
**Scale/Scope**: 2 workflow files modified (1 edited, 1 deleted)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --------- | ------ | ----- |
| I. Code Quality | ✅ Pass | No Rust code changes. YAML follows consistent style. |
| II. Testing Standards | ✅ Pass | All existing checks preserved. No test coverage regression. |
| III. UX Consistency | ✅ N/A | No user-facing changes. |
| IV. Performance | ✅ Pass | Eliminates duplicate WASM build, reducing total compute time. |
| V. Architecture Integrity | ✅ N/A | No changes to core or shell code. |

No violations. No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/019-improve-cicd/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 research findings
├── quickstart.md        # Verification steps
└── checklists/
    └── requirements.md  # Spec quality checklist
```

### Source Code (repository root)

```text
.github/workflows/
├── ci.yml               # MODIFY — add deploy job, switch to release build
└── deploy.yml           # DELETE — merged into ci.yml
```

**Structure Decision**: No new files or directories. One file modified, one file deleted.

## Design

### Current State (2 separate workflows)

```
ci.yml (push to main + PRs):
  test ──────────┐
  clippy ────────┤
  fmt ───────────┤ (all parallel, no deploy)
  wasm-build ────┤──→ e2e
  wasm-test ─────┘

deploy.yml (push to main only):
  deploy (full WASM rebuild from scratch)
```

**Problems**: deploy.yml runs independently (no gating), duplicates WASM build, debug build in CI vs release in deploy.

### Target State (single workflow)

```
ci.yml (push to main + PRs):
  test ──────────┐
  clippy ────────┤
  fmt ───────────┤
  wasm-build ────┤──→ e2e ──→ deploy (main only)
  wasm-test ─────┘
```

**Changes**:
1. `wasm-build` switches from `trunk build` to `trunk build --release` (one line)
2. New `deploy` job added to `ci.yml` with `needs: [test, clippy, fmt, wasm-build, wasm-test, e2e]` and `if: github.event_name == 'push' && github.ref == 'refs/heads/main'`
3. Deploy job: checkout (for wrangler.toml + worker.js) → download artifact to `crates/intrada-web/dist` → wrangler-action. No Rust toolchain, no trunk, no Tailwind.
4. `deploy.yml` deleted entirely.

### Key Design Decisions

- **Single workflow** over `workflow_run` — simpler, artifact sharing works natively, one UI view. See [research.md](research.md) R1.
- **Always release build** — eliminates duplicate builds, catches release-mode-only issues on PRs, modest time impact (~30-60s). See [research.md](research.md) R2.
- **Artifact to wrangler.toml path** — download to `crates/intrada-web/dist/` matches the existing `wrangler.toml` config, zero config changes. See [research.md](research.md) R4.
