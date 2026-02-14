# Quickstart: CI/CD Quality Gates

**Feature Branch**: `002-ci-cd`
**Date**: 2026-02-14

## What This Feature Does

Adds a GitHub Actions workflow that automatically runs quality checks (tests, linting, formatting) on every pull request against `main` and every push to `main`. Each check reports independently so developers can see exactly what passed or failed.

## Verification Steps

After implementing, verify the pipeline works by:

1. **Push to a branch and open a PR against main**
   - Verify three checks appear on the PR: test, clippy, fmt
   - All three should pass (assuming the codebase is clean)

2. **Introduce a failing test and push**
   - The `test` check should fail; `clippy` and `fmt` should still pass independently

3. **Introduce a clippy warning and push**
   - The `clippy` check should fail; `test` and `fmt` should still pass independently

4. **Introduce a formatting issue and push**
   - The `fmt` check should fail; `test` and `clippy` should still pass independently

5. **Push a second commit to the same PR**
   - The second run should be faster than the first due to dependency caching

## Files Changed

- `.github/workflows/ci.yml` — New file. The single workflow definition containing three parallel jobs.

## Commands Used by CI

```bash
# Tests (run across full workspace)
cargo test

# Linting (all targets including tests and examples)
cargo clippy -- -D warnings

# Formatting check (check mode, no modifications)
cargo fmt --all -- --check
```
