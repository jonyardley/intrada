# Quickstart: Web App Component Architecture

**Feature**: 005-component-architecture
**Date**: 2026-02-14

## Verification Scenarios

This feature is a pure refactoring. Verification focuses on proving zero regressions across compilation, tests, linting, and runtime behaviour.

### Scenario 1: Workspace Compilation

**Goal**: Confirm the refactored web crate compiles within the workspace.

```bash
# From repository root
cargo build
cargo build --target wasm32-unknown-unknown -p intrada-web
```

**Expected**: Both commands succeed with zero errors. The WASM target build confirms the web crate compiles to WASM as before.

### Scenario 2: All Workspace Tests Pass

**Goal**: Confirm all 82+ existing tests pass without modification.

```bash
cargo test
```

**Expected**: All tests pass. No test file should have been modified by this refactoring — the test suite validates that the Crux core and CLI shell remain unaffected.

### Scenario 3: Zero Clippy Warnings

**Goal**: Confirm code quality after restructuring.

```bash
cargo clippy -- -D warnings
```

**Expected**: Zero warnings across the entire workspace, including the refactored web crate.

### Scenario 4: Trunk Build (WASM Serving)

**Goal**: Confirm the web app builds and serves via Trunk.

```bash
cd crates/intrada-web
trunk build
```

**Expected**: Trunk builds successfully, producing WASM and JS artifacts. No Trunk configuration changes should be needed.

### Scenario 5: File Size Compliance

**Goal**: Verify no single file exceeds 300 lines (SC-001).

```bash
find crates/intrada-web/src -name '*.rs' -exec wc -l {} + | sort -rn
```

**Expected**: Every `.rs` file is under 300 lines. The largest file should be `edit_exercise.rs` at approximately 275 lines.

### Scenario 6: Line Count Overhead

**Goal**: Verify total line count overhead stays under 10% (SC-008).

```bash
# Current baseline: 1,906 lines in main.rs
# Maximum allowed: 1,906 * 1.10 = 2,096 lines
find crates/intrada-web/src -name '*.rs' -exec cat {} + | wc -l
```

**Expected**: Total line count across all files is under 2,097 lines.

### Scenario 7: Module Structure Verification

**Goal**: Verify the expected file structure exists.

```bash
find crates/intrada-web/src -name '*.rs' | sort
```

**Expected output** (approximately):
```
crates/intrada-web/src/app.rs
crates/intrada-web/src/components/form_field_error.rs
crates/intrada-web/src/components/library_item_card.rs
crates/intrada-web/src/components/mod.rs
crates/intrada-web/src/core_bridge.rs
crates/intrada-web/src/data.rs
crates/intrada-web/src/helpers.rs
crates/intrada-web/src/main.rs
crates/intrada-web/src/types.rs
crates/intrada-web/src/validation.rs
crates/intrada-web/src/views/add_exercise.rs
crates/intrada-web/src/views/add_piece.rs
crates/intrada-web/src/views/detail.rs
crates/intrada-web/src/views/edit_exercise.rs
crates/intrada-web/src/views/edit_piece.rs
crates/intrada-web/src/views/library_list.rs
crates/intrada-web/src/views/mod.rs
```

### Scenario 8: Runtime Smoke Test

**Goal**: Verify all user flows work in the browser.

```bash
cd crates/intrada-web
trunk serve
```

Then manually verify in browser:
1. Library list view loads with stub data
2. Click a piece → detail view renders correctly
3. Click "Edit" → edit form pre-fills with correct data
4. Click "Back" → returns to list
5. Click "Add Piece" → add piece form renders
6. Click "Add Exercise" → add exercise form renders
7. Submit forms with valid data → items appear in list
8. Submit forms with invalid data → validation errors display
9. Delete an item with confirmation → item removed from list

**Expected**: All flows behave identically to pre-refactoring state.

## Integration Notes

- **Crux core crate**: Completely unaffected. No changes to `intrada-core`.
- **CLI crate**: Completely unaffected. No changes to `intrada-cli`.
- **Build tooling**: `Trunk.toml`, `index.html`, `Cargo.toml` — no changes expected.
- **Tailwind CSS**: Utility classes remain inline in components. No CSS changes.
