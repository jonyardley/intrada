# Research: CI/CD Quality Gates

**Feature Branch**: `002-ci-cd`
**Date**: 2026-02-14

## R1: CI Toolchain Setup

**Decision**: Use `dtolnay/rust-toolchain@stable` GitHub Action for Rust installation.

**Rationale**: This is the modern standard for Rust CI. The older `actions-rs/toolchain` was deprecated in October 2023. `dtolnay/rust-toolchain` is concise (one-line setup), supports pinned versions via `@stable`, `@nightly`, or `@1.89.0`, and handles `rustup` under the hood with sensible defaults.

**Alternatives considered**:
- `actions-rs/toolchain` — deprecated, no longer maintained
- Manual `rustup` install via shell commands — works but verbose and harder to maintain

## R2: Dependency Caching Strategy

**Decision**: Use `Swatinem/rust-cache@v2` for automatic Cargo dependency caching.

**Rationale**: This action provides smart caching with zero configuration. It automatically keys on `Cargo.toml` and `Cargo.lock` across workspace members, uses prefix matching on lock changes (so adding a single crate doesn't invalidate the entire cache), and saves cache via post-action (preventing broken-build cache pollution). No manual cache key management needed.

**Alternatives considered**:
- `actions/cache` with manual key configuration — works but requires manual path and key setup, easy to get wrong, no prefix matching by default
- No caching — functional but slow; cold Rust builds take 20-30+ seconds even for small projects

## R3: Native Build Dependencies (rusqlite bundled)

**Decision**: Use `ubuntu-latest` runner with no additional setup for C compilation.

**Rationale**: Ubuntu GitHub Actions runners include `build-essential` (gcc, make, etc.) by default. The `rusqlite` `bundled` feature uses the `cc` crate to compile SQLite from C source, which works out of the box on `ubuntu-latest`. No `apt-get install` step is needed.

**Alternatives considered**:
- Explicit `apt-get install build-essential` step — unnecessary on ubuntu-latest, adds ~5s to pipeline for no benefit
- macOS/Windows runners — not needed unless cross-platform builds become a requirement (currently out of scope)

## R4: Job Organisation

**Decision**: Three parallel jobs — `test`, `clippy`, `fmt` — each running independently.

**Rationale**: Running jobs in parallel gives faster overall feedback. Each job reports its status independently on the PR (FR-010), so a developer can immediately see which specific check failed. The three jobs share the same runner setup (Rust stable + rust-cache) but run different commands. Since there are no dependencies between checks, parallelism is free.

**Alternatives considered**:
- Single job with sequential steps — simpler YAML but slower total time, and a failing `fmt` step wouldn't run if `test` fails first (unless using `continue-on-error`). Also reports as a single pass/fail rather than three independent checks.
- Separate workflows per check — overcomplicates the repository structure with three workflow files for the same trigger conditions

## R5: Existing Project State

**Decision**: No changes to existing project configuration needed. Add workflow file only.

**Findings**:
- Cargo.lock is committed (required for reproducible CI builds) ✓
- No `rust-toolchain.toml` exists — the workflow will pin `stable` explicitly
- No `.rustfmt.toml` or `clippy.toml` — default Rust formatting and lint rules apply
- No `.cargo/config.toml` — no custom build flags
- 82 tests currently pass across both crates (64 core + 18 CLI)
- 0 clippy warnings
- Workspace uses `resolver = "2"` ✓
