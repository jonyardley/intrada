# Quickstart Verification: UI Primitive Components

**Feature**: 006-ui-primitives
**Date**: 2026-02-14
**Purpose**: Define verification scenarios to confirm the refactoring is correct after implementation

## Prerequisites

- Branch `006-ui-primitives` checked out
- Rust toolchain installed (stable 1.75+)
- `trunk` installed for WASM builds

## Verification Scenarios

### V1: Workspace Compilation

```bash
cargo build --workspace
```

**Expected**: Clean compilation with zero errors. All new component files resolve imports correctly.

### V2: Clippy Clean

```bash
cargo clippy --workspace -- -D warnings
```

**Expected**: Zero clippy warnings across all crates. No unused imports, dead code, or style violations.

### V3: Existing Test Suite

```bash
cargo test --workspace
```

**Expected**: All 82 existing tests pass. No test modifications were needed (this is a pure UI refactoring with no logic changes).

### V4: WASM Build

```bash
cd crates/intrada-web && trunk build
```

**Expected**: WASM binary builds successfully. No missing imports or type errors in the web crate.

### V5: Visual Smoke Test (Manual)

1. Run `trunk serve` from `crates/intrada-web/`
2. Open browser to the served URL
3. Verify the following pages look identical to pre-refactoring:

| Page | Elements to Check |
|------|-------------------|
| Library list | Primary "Add Piece" / "Add Exercise" buttons, card containers, type badges |
| Detail view | Back link, page heading, card container, type badge, field labels, Edit/Delete buttons |
| Add Piece form | Back link, page heading, form card, all text fields with labels, Save/Cancel buttons |
| Add Exercise form | Same as Add Piece but with Category and Difficulty fields |
| Edit Piece form | Same as Add Piece but pre-populated with existing data |
| Edit Exercise form | Same as Add Exercise but pre-populated |
| Delete confirmation | Danger button styling in the confirmation banner |

**Expected**: Pixel-identical appearance on all pages. No missing styles, broken layouts, or visual regressions.

### V6: Form Validation Smoke Test (Manual)

1. Navigate to Add Piece
2. Submit the empty form
3. Verify error messages appear below Title and Composer fields
4. Enter valid data and submit
5. Verify the piece appears in the library list

**Expected**: Validation behaviour is identical to pre-refactoring. Errors display correctly below the appropriate fields.

### V7: File Size Check

```bash
wc -l crates/intrada-web/src/**/*.rs crates/intrada-web/src/**/**/*.rs
```

**Expected**: No individual file exceeds 300 lines (SC-005).

### V8: Duplication Reduction Check

Verify that view files no longer contain inline CSS class strings for:
- Button styling (`rounded-lg bg-indigo-600`, `rounded-lg bg-white ... border-slate-300`)
- Form label styling (`block text-sm font-medium text-slate-700 mb-1`)
- Form input styling (`w-full rounded-lg border border-slate-300 px-3 py-2`)
- Card styling (`bg-white rounded-xl shadow-sm border border-slate-200 p-6`)
- Page heading styling (`text-2xl font-bold text-slate-900 mb-6`)
- Back link styling (`mb-6 inline-flex items-center gap-1 text-sm text-slate-500`)

**Expected**: These class strings appear only in their respective component files, not in view files (SC-001, SC-002, SC-007).

## Quick Verification Script

```bash
# Run all automated checks in sequence
cargo build --workspace && \
cargo clippy --workspace -- -D warnings && \
cargo test --workspace && \
(cd crates/intrada-web && trunk build)
```

If all four commands succeed, the automated verification is complete. Manual visual verification (V5, V6) should follow.
