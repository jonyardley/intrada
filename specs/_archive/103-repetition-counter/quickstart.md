# Quickstart: Repetition Counter

**Branch**: `103-repetition-counter` | **Date**: 2026-02-21

## Build & Test

```bash
cargo fmt --check          # Formatting
cargo clippy -- -D warnings # Linting
cargo test                 # Full workspace tests (core + web + api)
```

## Manual Verification Steps

### 1. Counter During Active Practice (US1)

1. Start the app, sign in, navigate to Practice
2. Add at least one item to the setlist
3. Start the session
4. On the active item, tap "Enable counter" — verify counter shows "0 / 5"
5. Tap "Got it" 3 times — verify counter shows "3 / 5"
6. Tap "Missed" once — verify counter shows "2 / 5"
7. Tap "Missed" 3 more times — verify counter stays at "0 / 5" (floor)
8. Tap "Got it" 5 times — verify achievement indicator appears at "5 / 5"
9. Dismiss the prompt — verify counter freezes, buttons hidden, timer continues
10. Tap "Next Item" or "Finish Session" — verify session progresses normally

### 2. Configure Target Per Item (US2)

1. In the building phase, add two items
2. On the first item, tap "Add rep target" — verify stepper appears at 5
3. Increase to 7 — verify stepper shows 7
4. On the second item, leave rep target off (no link tapped)
5. Start the session
6. First item: verify counter shows "0 / 7"
7. Second item: verify no counter visible

### 3. Summary and History Display (US3)

1. Complete a session using the counter on at least one item (reach target on one, skip another mid-count)
2. In session summary: verify achieved item shows "5/5 ✓", skipped item shows "2/5"
3. Save the session
4. Navigate to session history — verify rep data visible alongside score and duration

### 4. Crash Recovery (FR-007)

1. Start a session with the counter enabled
2. Tap "Got it" a few times (e.g., to 3/5)
3. Close the browser tab
4. Reopen the app — verify crash recovery prompt appears
5. Recover the session — verify counter shows "3 / 5" and target is correct

### 5. Backward Compatibility (FR-011)

1. View an existing session saved before this feature
2. Verify it displays correctly with no rep count information
3. Verify no errors in the browser console
