# Quickstart: Remove CLI Shell

**Feature**: 014-remove-cli
**Date**: 2026-02-15

## Verification Steps

After implementing this feature, verify the removal is complete:

### 1. CLI Crate Removed

```bash
# Confirm directory no longer exists
ls crates/intrada-cli/
# Expected: No such file or directory

# Confirm only two crates remain
ls crates/
# Expected: intrada-core  intrada-web
```

### 2. Workspace Builds Successfully

```bash
# Full build
cargo build

# All tests pass (core + web)
cargo test

# No clippy warnings
cargo clippy -- -D warnings

# Formatting check
cargo fmt --all -- --check
```

### 3. CLI-Only Dependencies Removed

```bash
# Verify clap, anyhow, dirs are not in workspace deps
grep -E "^(clap|anyhow|dirs)" Cargo.toml
# Expected: no output
```

### 4. Documentation Updated

```bash
# README should have no CLI references
grep -i "intrada-cli\|CLI shell\|CLI usage\|cargo run --bin intrada" README.md
# Expected: no matches

# CLAUDE.md should have no CLI in project structure
grep "intrada-cli" CLAUDE.md
# Expected: no matches
```

### 5. CI Pipeline

```bash
# Run full CI validation locally
cargo test && cargo clippy -- -D warnings && cargo fmt --all -- --check
```

The CI pipeline (test, clippy, fmt, wasm-build, wasm-test, e2e) should pass without any workflow file changes.

## What Changes

| Item | Before | After |
|------|--------|-------|
| Workspace crates | 3 (core, cli, web) | 2 (core, web) |
| Workspace deps | 9 | 6 |
| CLI source files | 4 (main, shell, storage, display) | 0 |
| README sections | Includes CLI usage, CLI architecture, CLI storage | Web-only architecture and storage |

## What Does NOT Change

- `intrada-core` — no modifications
- `intrada-web` — no modifications
- `.github/workflows/ci.yml` — no modifications
- Historical specs (`specs/001-*` through `specs/013-*`) — preserved as-is
- E2E tests — no modifications
