# Quickstart: Reusable Routines

**Feature**: 025-reusable-routines
**Date**: 2026-02-18

## Prerequisites

- Rust stable toolchain (1.75+)
- Trunk installed (`cargo install trunk`)
- Tailwind CSS v4 standalone CLI available
- Running Turso database (or local libsql for development)
- API server running (`cargo run -p intrada-api`)

## Build & Run

```bash
# Run all tests (core + api + wasm)
cargo test

# Run clippy
cargo clippy -- -D warnings

# Start the API server (in one terminal)
cargo run -p intrada-api

# Start the web dev server (in another terminal)
cd crates/intrada-web && trunk serve
```

## Verification Steps

### V1: Save Routine from Building Phase (FR-001, FR-009, FR-010, FR-014)

1. Open the app and add 2–3 items to the library (pieces and/or exercises)
2. Navigate to `/sessions/new` to start building a new session
3. Add at least 2 items to the setlist
4. Verify: a "Save as Routine" option is visible below the setlist
5. Tap "Save as Routine" → an inline name input and Save/Cancel buttons appear
6. Try to save with an empty name → validation error shown
7. Enter the name "Morning Warm-up" and tap Save
8. **Expected**: Success feedback. The building state remains unchanged — setlist items are still there
9. With the setlist empty (remove all items), verify: "Save as Routine" option is hidden or disabled

### V2: Load Routine into Session (FR-003, FR-004)

1. Start a new session at `/sessions/new`
2. Verify: a "Saved Routines" section appears showing "Morning Warm-up" with entry count
3. Tap "Load" on "Morning Warm-up"
4. **Expected**: Routine entries appear in the setlist in saved order
5. Add one more item manually to the setlist
6. Tap "Load" on "Morning Warm-up" again
7. **Expected**: Routine entries are appended after existing items (additive, not replacing)
8. Verify: all entries have unique IDs (no duplicates from loading the same routine twice)

### V3: Save Routine from Session Summary (FR-002)

1. Start and complete a practice session (or end early)
2. On the summary screen, verify: a "Save as Routine" option is visible
3. Tap "Save as Routine", enter "Bach Recital Prep", and save
4. **Expected**: Routine is created from the session's setlist entries
5. Navigate to `/sessions/new` and verify both "Morning Warm-up" and "Bach Recital Prep" appear in the routine list

### V4: Routine Management Page (FR-005, FR-006, FR-015)

1. Navigate to `/routines`
2. Verify: both routines are listed with names and entry counts
3. Tap "Delete" on one routine and confirm
4. **Expected**: Routine is removed from the list
5. Refresh the page
6. **Expected**: Deleted routine does not reappear (persisted to server)

### V5: Routine Editing (FR-007, FR-008, FR-016)

1. Navigate to `/routines`
2. Tap "Edit" on a routine
3. Verify: the edit page at `/routines/:id/edit` shows the routine name and entry list
4. Change the name and tap Save
5. **Expected**: Name is updated on the routines list
6. Edit the routine again — remove an entry and reorder the remaining entries
7. Tap Save
8. **Expected**: Changes are persisted. Refresh the page to confirm.
9. Add a new entry from the library, save, and verify it appears

### V6: Edge Cases

1. Delete all library items, then try to load a routine that referenced them → routine loads with denormalized titles (items still appear)
2. Save a routine with a name exactly 200 characters → succeeds
3. Try to save a routine with 201 characters → validation error
4. Try to update a routine to have zero entries → validation error
5. Navigate directly to `/routines/:invalid-id/edit` → appropriate error or redirect
6. With no saved routines, visit `/sessions/new` → no routine section displayed (or empty state)

### V7: Persistence & Startup (FR-012, FR-013, SC-004)

1. Save a routine, then close and reopen the app
2. **Expected**: Routine appears on startup (fetched from server)
3. Load the routine into a session → entries match what was saved
4. Run `cargo test` — all existing and new tests pass

## Smoke Test (CI)

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

All existing tests must continue to pass. New tests for routine events, API CRUD, and validation must be added as part of implementation.
