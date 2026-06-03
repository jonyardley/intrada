# Quickstart: Session Item Scoring

**Feature**: 022-session-scoring
**Date**: 2026-02-17

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

### V1: Score Assignment (FR-001, FR-002, FR-003)

1. Open the app and navigate to the library
2. Start a new practice session with at least 2 items
3. Complete the session (or end early)
4. On the summary screen, verify:
   - Each **completed** entry shows 5 tappable score buttons (1–5)
   - Skipped or not-attempted entries do NOT show score buttons
5. Tap score "4" on the first completed entry → button highlights
6. Tap score "4" again → deselects (returns to no score)
7. Tap score "3" → button 3 highlights
8. Leave the second entry unscored
9. Save the session
10. **Expected**: Session saves successfully with no errors

### V2: Score Persistence & Display (FR-004, FR-005, FR-006)

1. After V1, navigate to the session history
2. Open the session you just saved
3. Verify:
   - First entry shows score "3"
   - Second entry shows no score indicator
   - Duration, status, and notes are unchanged
4. Open an older session (saved before this feature)
5. Verify: entries display normally with no score — no errors or visual artefacts

### V3: Progress Summary (FR-007, FR-010, FR-011)

1. Complete 2–3 sessions that include the same library item, scoring it differently each time (e.g., 2, then 3, then 4)
2. Navigate to that item's detail page
3. Verify:
   - The most recent confidence score is displayed prominently
   - Below it, a chronological list shows all past scores with session dates
   - Most recent score appears first in the list
4. Navigate back to the library list view
5. Verify: scores do NOT appear on the list cards (FR-011)

### V4: Edge Cases

1. Save a session with zero entries scored → session saves normally
2. Add the same item twice to a session, score each differently → both scores appear in that item's progress history
3. View an item that has been practised but never scored → "No scores available" message shown

### V5: Backward Compatibility (FR-006, SC-004)

1. Verify existing sessions in the database display correctly
2. Verify the API returns `null` for `score` on pre-existing entries
3. Run `cargo test` — all existing tests pass with no modification

## Smoke Test (CI)

```bash
cargo test && cargo clippy -- -D warnings
```

All existing tests must continue to pass. New tests for scoring must be added as part of implementation.
