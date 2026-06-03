# Research: Remove CLI Shell

**Feature**: 014-remove-cli
**Date**: 2026-02-15

## Overview

This is a removal/cleanup feature with no unknowns or technology choices. Research confirms the safe removal path.

## R1: CLI-Only Workspace Dependencies

**Decision**: Remove `clap`, `anyhow`, and `dirs` from root `Cargo.toml` workspace dependencies.

**Rationale**: Cross-referencing the workspace `[workspace.dependencies]` with each remaining crate's `Cargo.toml`:

| Workspace Dep | intrada-core | intrada-web | Action |
|---------------|-------------|-------------|--------|
| crux_core     | ✅ used     | ✅ used     | KEEP   |
| serde         | ✅ used     | —           | KEEP   |
| serde_json    | ✅ used     | ✅ used     | KEEP   |
| ulid          | ✅ used     | ✅ used     | KEEP   |
| chrono        | ✅ used     | ✅ used     | KEEP   |
| thiserror     | ✅ used     | —           | KEEP   |
| clap          | —           | —           | REMOVE |
| anyhow        | —           | —           | REMOVE |
| dirs          | —           | —           | REMOVE |

**Alternatives considered**: Keeping dependencies "in case we need them later" — rejected per Constitution principle I (No Dead Code).

## R2: Workspace Member Pattern

**Decision**: No change needed to `members = ["crates/*"]`.

**Rationale**: The glob pattern `crates/*` automatically discovers workspace members from subdirectories. Deleting `crates/intrada-cli/` is sufficient — Cargo will no longer find it as a member. Verified by Cargo documentation: workspace members are resolved at `cargo` invocation time from the glob pattern.

**Alternatives considered**: Switching to explicit member list — rejected as it adds maintenance burden with no benefit.

## R3: CI Pipeline Impact

**Decision**: No CI workflow changes needed.

**Rationale**: The CI pipeline uses workspace-level commands (`cargo test`, `cargo clippy`, `cargo fmt --all`). These operate on all workspace members, which will automatically exclude the deleted CLI crate. The WASM and E2E jobs are web-specific and unaffected. Verified by reviewing `.github/workflows/ci.yml` — no direct references to `intrada-cli`.

## R4: Documentation Scope

**Decision**: Update README.md and CLAUDE.md only. Do not modify historical spec documents.

**Rationale**: README.md and CLAUDE.md are living documentation that must reflect current project state. Historical specs (001 through 013) are point-in-time design records that document what was built when. Modifying them would destroy historical accuracy. The README currently contains outdated information (references SQLite, which was replaced by JSON in feature 011, and then the CLI is being fully removed now).

**Alternatives considered**: Updating all specs to remove CLI mentions — rejected per FR-008 and good archival practice.
