# Quickstart: Tempo Tracking

**Feature**: 105-tempo-tracking
**Date**: 2026-02-24

## Prerequisites

- Rust stable toolchain (1.89+)
- `trunk` for web builds
- Running API server with Turso database
- Feature branch `105-tempo-tracking` checked out

## Build & Test

```bash
# Run all core tests (includes new tempo validation and history tests)
cargo test -p intrada-core

# Run API tests (includes achieved_tempo persistence tests)
cargo test -p intrada-api

# Run full workspace
cargo test

# Lint and format
cargo clippy -- -D warnings
cargo fmt --check
```

## Verification Steps

### V1: Achieved tempo field in session summary (FR-001, FR-002, FR-007, FR-008)

1. Start a practice session with at least one item
2. Complete the session (or end early)
3. On the summary screen, verify:
   - Each **completed** entry shows an "Achieved tempo (BPM)" input field below the confidence score buttons
   - **Skipped** and **Not Attempted** entries do NOT show the tempo field
4. Enter a BPM value (e.g., 108) for a completed entry
5. Leave another completed entry's tempo field empty
6. Save the session
7. Verify: Session saves successfully, no errors

### V2: Achieved tempo validation (FR-008)

1. In the session summary, try entering:
   - `0` → rejected (below minimum)
   - `501` → rejected (above maximum)
   - `abc` → rejected (non-numeric)
   - `1` → accepted (minimum valid)
   - `500` → accepted (maximum valid)
   - `120` → accepted (typical value)
2. Verify: Invalid values show a validation message, valid values are accepted

### V3: Achieved tempo in session history (FR-003)

1. After saving a session with achieved tempo values
2. Navigate to session history
3. View the session detail
4. Verify: Achieved tempo values are displayed for entries that had them
5. Verify: Entries without achieved tempo show no tempo value

### V4: Tempo history on item detail (FR-004, FR-005)

1. Log achieved tempos for the same item across 3+ sessions
2. Navigate to the item's detail view
3. Verify:
   - A "Tempo History" section appears (only if tempos have been recorded)
   - Each entry shows date and achieved BPM
   - Entries are ordered most recent first
   - If the item has a target BPM set, it's shown as a reference alongside the history

### V5: Latest tempo in library list (FR-006)

1. After logging achieved tempos for an item
2. Navigate to the library list
3. Verify:
   - The item shows its latest achieved BPM
   - If the item also has a target BPM, both are visible (e.g., "108 / 120 BPM")
   - Items without any achieved tempos show only the target BPM (if set)
   - Items with neither show no tempo information

### V6: Session deletion removes tempo data (FR-009)

1. Log an achieved tempo for an item
2. Verify the tempo appears in the item's tempo history
3. Delete the session
4. Verify: The tempo history no longer includes that data point

### V7: No data with empty sessions (edge case)

1. Navigate to an item that has never had a tempo recorded
2. Verify: No "Tempo History" section appears (or shows a graceful empty state)

## Performance Check

The `test_performance_10k_items` test in `app.rs` must continue to pass within the <200ms threshold after adding tempo tracking to the practice summary cache.
