# Implementation Plan: CI/CD Quality Gates

**Branch**: `002-ci-cd` | **Date**: 2026-02-14 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/002-ci-cd/spec.md`

## Summary

Add a GitHub Actions workflow that runs three quality gates — tests, clippy linting, and formatting checks — on every pull request targeting main and every push to main. Jobs run in parallel with independent status reporting and Cargo dependency caching for fast feedback.

## Technical Context

**Language/Version**: Rust stable (currently 1.89.0, project minimum 1.75+, 2021 edition)
**Primary Dependencies**: GitHub Actions, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`
**Storage**: N/A
**Testing**: `cargo test` (82 tests across 2 workspace crates)
**Target Platform**: `ubuntu-latest` GitHub Actions runner
**Project Type**: Cargo workspace with 2 member crates (`crates/*`)
**Performance Goals**: Cached pipeline run < 5 minutes (SC-003)
**Constraints**: rusqlite uses `bundled` feature requiring C compiler (available by default on ubuntu-latest)
**Scale/Scope**: Single workflow file, 3 parallel jobs

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Research Gate

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | Workflow file follows YAML best practices; single-responsibility jobs; no dead code |
| II. Testing Standards | ✅ Pass | This feature *is* the testing infrastructure — enforces test execution on every PR and push |
| III. User Experience Consistency | ✅ Pass | N/A for CI config (no user-facing UI). Developer UX: consistent check names, independent status reporting |
| IV. Performance Requirements | ✅ Pass | Dependency caching addresses performance; SC-003 sets 5-minute target |

### Post-Design Gate

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | Minimal YAML, well-structured with named jobs and clear step names |
| II. Testing Standards | ✅ Pass | Pipeline itself is verified by opening PRs with passing/failing code |
| III. User Experience Consistency | ✅ Pass | Three independent checks visible on PR, consistent naming |
| IV. Performance Requirements | ✅ Pass | Swatinem/rust-cache provides automatic cache keying on Cargo.lock |

No violations. Complexity Tracking table not needed.

## Project Structure

### Documentation (this feature)

```text
specs/002-ci-cd/
├── plan.md              # This file
├── research.md          # Phase 0: tool/strategy research
├── quickstart.md        # Verification steps
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
.github/
└── workflows/
    └── ci.yml           # Single workflow: 3 parallel jobs (test, clippy, fmt)
```

**Structure Decision**: A single workflow file with three parallel jobs. This is the simplest structure that satisfies FR-010 (independent status per check) without overcomplicating the repo with multiple workflow files.

## Design

### Workflow Architecture

```
ci.yml
├── trigger: push to main, PR against main
├── job: test
│   ├── checkout
│   ├── rust-toolchain (stable)
│   ├── rust-cache
│   └── cargo test
├── job: clippy
│   ├── checkout
│   ├── rust-toolchain (stable, components: clippy)
│   ├── rust-cache
│   └── cargo clippy -- -D warnings
└── job: fmt
    ├── checkout
    ├── rust-toolchain (stable, components: rustfmt)
    └── cargo fmt --all -- --check
```

### Key Design Decisions

1. **Three parallel jobs** (not sequential steps): Each job reports independently on the PR. If formatting fails, the developer doesn't have to wait for tests to finish to see it. Each check shows as a separate ✅ or ❌ in the PR checks list.

2. **`-D warnings` for clippy**: Treats all clippy warnings as errors. This prevents warning accumulation — any new warning breaks CI, forcing immediate resolution.

3. **No cache for fmt job**: The formatting check (`cargo fmt --check`) only parses source files; it doesn't compile anything. Caching would add overhead for no benefit. Only `test` and `clippy` jobs (which compile) benefit from rust-cache.

4. **`ubuntu-latest` only**: The project is CLI-only targeting local developer machines. Cross-platform CI (macOS, Windows) is out of scope per the spec. Ubuntu runners have C compiler toolchain pre-installed, satisfying FR-009.

5. **Stable Rust via `dtolnay/rust-toolchain`**: Pins to `stable` channel. No `rust-toolchain.toml` is added — the workflow manages its own toolchain. This avoids coupling local dev environment to CI config.

### Requirement Traceability

| Requirement | Satisfied By |
|-------------|-------------|
| FR-001: Run on PRs against main | `on: pull_request: branches: [main]` |
| FR-002: Run on push to main | `on: push: branches: [main]` |
| FR-003: Test suite execution | `test` job: `cargo test` |
| FR-004: Linting checks | `clippy` job: `cargo clippy -- -D warnings` |
| FR-005: Formatting checks | `fmt` job: `cargo fmt --all -- --check` |
| FR-006: Cache dependencies | `Swatinem/rust-cache@v2` in test + clippy jobs |
| FR-007: Graceful cache miss | rust-cache falls back to full build automatically |
| FR-008: Stable Rust toolchain | `dtolnay/rust-toolchain@stable` |
| FR-009: Native compilation support | ubuntu-latest includes build-essential (C compiler) |
| FR-010: Independent check status | Three separate jobs, each reports independently |
