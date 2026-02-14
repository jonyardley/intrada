# Quickstart: Crux & Leptos Upgrade

**Feature**: 007-crux-leptos-upgrade
**Date**: 2026-02-14

## What This Upgrade Does

- Bumps Leptos from 0.7 to 0.8 (latest stable: 0.8.15) in the web shell
- Adopts the new `bind:value` directive for form two-way binding (replaces verbose `prop:value` + `on:input` pattern)
- Keeps crux_core at 0.17.0-rc2 (already the latest published version)
- Preserves all existing functionality — zero user-facing changes

## Pre-Upgrade Baseline

Before starting, capture the current state:

```bash
# Record test count
cargo test --workspace 2>&1 | tail -5

# Record WASM binary size (for NFR-002 comparison)
trunk build --release
ls -la dist/*.wasm

# Verify clean state
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## Upgrade Steps

### Step 1: Version Bump

In `crates/intrada-web/Cargo.toml`, change:

```toml
# Before
leptos = { version = "0.7", features = ["csr"] }

# After
leptos = { version = "0.8", features = ["csr"] }
```

Then update the lockfile:

```bash
cargo update -p leptos
```

### Step 2: Build & Fix

```bash
cargo build --workspace
```

Expected: compiles cleanly with no errors. If any errors appear, they will be API changes documented in the [research](research.md).

### Step 3: Adopt `bind:value` Pattern

The key idiomatic change in Leptos 0.8 is the `bind:value` directive for two-way form binding.

**Before (0.7 pattern)**:
```rust
<input
    prop:value=move || value.get()
    on:input=move |ev| { value.set(event_target_value(&ev)); }
/>
```

**After (0.8 idiomatic)**:
```rust
<input bind:value=value />
```

Apply this to:
- `text_field.rs` — the `<input>` element
- `text_area.rs` — the `<textarea>` element
- Any view files with inline `prop:value` + `on:input` patterns

**Important**: Preserve all other attributes (`aria-describedby`, `aria-invalid`, `required`, `placeholder`, `id`, `type`, `class`) — only replace the value binding pair.

### Step 4: Verify

```bash
# Full CI gate
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
trunk build

# Binary size check (must be <120% of baseline)
trunk build --release
ls -la dist/*.wasm

# Manual smoke test
trunk serve
# Visit http://localhost:8080 and test all views
```

## Files Changed

| File | Change |
|------|--------|
| `crates/intrada-web/Cargo.toml` | Leptos version 0.7 → 0.8 |
| `crates/intrada-web/src/components/text_field.rs` | `bind:value` migration |
| `crates/intrada-web/src/components/text_area.rs` | `bind:value` migration |
| View files (add_piece, add_exercise, edit_piece, edit_exercise) | Verify / migrate any inline binding patterns |
| `crates/intrada-web/src/views/library_list.rs` | Verify search input binding |

## Files NOT Changed

| File | Reason |
|------|--------|
| `Cargo.toml` (workspace root) | crux_core stays at 0.17.0-rc2 |
| All `intrada-core/` files | Pure core, no Leptos dependency |
| All `intrada-cli/` files | CLI shell, no Leptos dependency |
| `crates/intrada-web/src/types.rs` | SendWrapper usage unaffected |
| `crates/intrada-web/src/core_bridge.rs` | Crux bridge patterns unchanged |

## Rollback

If the upgrade causes unforeseen issues:

```bash
# Revert Cargo.toml change
git checkout -- crates/intrada-web/Cargo.toml
cargo update -p leptos
```

The upgrade is scoped to a single version specifier change plus optional pattern adoption, making rollback trivial.
