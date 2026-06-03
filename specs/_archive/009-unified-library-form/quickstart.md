# Quickstart: Unified Library Item Form

**Feature**: 009-unified-library-form
**Date**: 2026-02-14

## Prerequisites

- Rust stable toolchain (1.75+)
- trunk 0.21.x installed (`cargo install trunk`)
- All existing tests pass: `cargo test --workspace`

## Running the App

```bash
cd crates/intrada-web
trunk serve
```

Open `http://localhost:8080/` in a browser.

## Test Scenarios

### Scenario 1: Add Piece via Unified Form (US1)

1. Navigate to `http://localhost:8080/`
2. Click the "Add Item" button in the library list header
3. **Verify**: The URL changes to `/library/new`
4. **Verify**: A tabbed interface appears at the top with "Piece" and "Exercise" tabs
5. **Verify**: "Piece" tab is selected by default (visually distinct — active styling)
6. **Verify**: Form shows: Title (required), Composer (required), Key, Tempo Marking, BPM, Notes, Tags
7. Fill in: Title = "Moonlight Sonata", Composer = "Beethoven"
8. Click "Save"
9. **Verify**: Redirected to `/` (library list)
10. **Verify**: "Moonlight Sonata" appears in the library list as a "piece" type

**Expected result**: Piece created successfully via the unified form.

### Scenario 2: Add Exercise via Unified Form (US1)

1. Navigate to `/library/new`
2. **Verify**: "Piece" tab is active by default
3. Click the "Exercise" tab
4. **Verify**: The form updates — Composer label changes to non-required, Category field appears
5. Fill in: Title = "Scale Practice", Category = "Scales"
6. Click "Save"
7. **Verify**: Redirected to `/` (library list)
8. **Verify**: "Scale Practice" appears as an "exercise" type

**Expected result**: Exercise created successfully via the unified form.

### Scenario 3: Tab Switching Preserves Shared Fields (US1, FR-004)

1. Navigate to `/library/new`
2. On the "Piece" tab, fill in:
   - Title = "Test Title"
   - Composer = "Test Composer"
   - Key = "C Major"
   - Notes = "Some notes"
3. Click the "Exercise" tab
4. **Verify**: Title = "Test Title" (preserved)
5. **Verify**: Composer = "Test Composer" (preserved — shared field)
6. **Verify**: Key = "C Major" (preserved)
7. **Verify**: Notes = "Some notes" (preserved)
8. **Verify**: Category field appears (empty)
9. Fill in Category = "Etude"
10. Click the "Piece" tab
11. **Verify**: All shared fields still have their values
12. **Verify**: Category field is no longer visible
13. Click the "Exercise" tab
14. **Verify**: Category = "Etude" (preserved from step 9)

**Expected result**: Shared field values persist across tab switches. Category value is preserved when hidden.

### Scenario 4: Validation Rules Change Per Tab (US1, FR-006, FR-007)

1. Navigate to `/library/new`
2. On the "Piece" tab, leave all fields empty
3. Click "Save"
4. **Verify**: Error appears: "Title is required" and "Composer is required"
5. Click the "Exercise" tab
6. **Verify**: All validation errors are cleared
7. Leave all fields empty
8. Click "Save"
9. **Verify**: Only error: "Title is required" (composer is optional for exercises)
10. Type "My Exercise" in Title
11. Click "Save"
12. **Verify**: Exercise is created successfully (no composer required)

**Expected result**: Validation rules adapt to the active tab. Errors clear on tab switch.

### Scenario 5: Edit Form — Display-Only Tabs (US2, FR-015)

1. Navigate to `/` and click on a piece item to view its detail
2. Click "Edit"
3. **Verify**: URL is `/library/{id}/edit`
4. **Verify**: The "Piece" tab is selected and visually active
5. **Verify**: The "Exercise" tab is visible but disabled/greyed out (not clickable)
6. **Verify**: Form fields are pre-populated with the item's current data
7. Try clicking the "Exercise" tab
8. **Verify**: Nothing happens — the tab does not switch
9. Modify the title and click "Save"
10. **Verify**: Redirected to the detail view with the updated title
11. Navigate to `/` and click on an exercise item, then click "Edit"
12. **Verify**: The "Exercise" tab is selected, "Piece" tab is disabled
13. **Verify**: Category field is visible and pre-populated (if the exercise has a category)

**Expected result**: Edit form shows display-only tabs matching the item's type. Type cannot be changed.

### Scenario 6: Unified URL Structure (US3)

1. Navigate directly to `http://localhost:8080/library/new`
2. **Verify**: Unified tabbed form loads
3. Switch between Piece and Exercise tabs
4. **Verify**: URL does not change (stays `/library/new`)
5. Navigate to `http://localhost:8080/pieces/new`
6. **Verify**: Shows 404 / Not Found view (old route removed)
7. Navigate to `http://localhost:8080/exercises/new`
8. **Verify**: Shows 404 / Not Found view (old route removed)
9. Navigate to `/` and verify "Add Item" is a single button, not a dropdown
10. For an existing piece: navigate to `http://localhost:8080/library/{id}/edit`
11. **Verify**: Edit form loads correctly for that item

**Expected result**: Routes are consolidated. Old routes are removed. Single "Add Item" button.

### Scenario 7: Keyboard Accessibility (FR-013)

1. Navigate to `/library/new`
2. Tab (keyboard) to focus the first tab button ("Piece")
3. Press Right Arrow key
4. **Verify**: Focus moves to "Exercise" tab
5. Press Enter or Space
6. **Verify**: "Exercise" tab becomes active, form updates
7. Press Left Arrow key
8. **Verify**: Focus moves to "Piece" tab
9. Press Enter or Space
10. **Verify**: "Piece" tab becomes active, form updates
11. Navigate to an edit form
12. Tab to focus the tab bar
13. **Verify**: Tabs receive focus but arrow keys / Enter do not switch them (display-only)

**Expected result**: Tabs are keyboard-navigable on the add form. Tabs are non-interactive on the edit form.

### Scenario 8: Submission Uses Correct Type (FR-008)

1. Navigate to `/library/new`
2. On the "Exercise" tab, fill in Title = "Speed Drill" and Category = "Technique"
3. Switch to the "Piece" tab
4. **Verify**: Category field is hidden
5. Click "Save" (on the Piece tab — with Title and Composer filled in)
6. **Verify**: The created item is a **piece** (not an exercise), despite having filled Category earlier
7. Click on the new item in the list
8. **Verify**: Item type badge shows "Piece", no category is displayed

**Expected result**: The form submits using the currently active tab's type, ignoring type-specific fields from the other tab.

## CI Checks

After all changes:

```bash
cargo test --workspace         # All 82+ tests must pass
cargo clippy --workspace -- -D warnings  # Zero warnings
cargo fmt --all -- --check     # Formatting compliant
```
