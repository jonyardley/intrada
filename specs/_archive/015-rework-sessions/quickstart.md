# Quickstart: Rework Sessions (Setlist Model)

**Feature**: 015-rework-sessions
**Date**: 2026-02-15

---

## Verification Steps

These steps verify the feature meets the spec's success criteria and functional requirements after implementation.

### Prerequisites

```bash
# All core tests pass
cargo test

# No clippy warnings
cargo clippy -- -D warnings

# Web app builds
cd crates/intrada-web && trunk build --release
```

---

### 1. Build a Practice Setlist (US1)

**Verifies**: FR-001, FR-002, FR-003, FR-004, SC-001

1. Open the app in a browser
2. Navigate to start a new session
3. Browse the library and select 3+ items → items appear in the setlist in order added
4. Reorder items by dragging/moving → setlist reflects new order
5. Remove one item → item removed, remaining items keep relative order
6. Attempt to start with an empty setlist → system prevents starting with clear message
7. **Timing**: Building a setlist of 3 items should take under 30 seconds (SC-001)

### 2. Practice Through a Setlist with Timer (US2)

**Verifies**: FR-005, FR-006, FR-007, FR-008, SC-002

1. Start a session with a 3-item setlist
2. First item displayed with timer counting from zero → verify item name and timer visible
3. Progress indicator shows "1 of 3"
4. Wait ~10 seconds, press "Next" → time recorded for first item, second item becomes active with fresh timer
5. Progress shows "2 of 3", first item marked as done
6. Advance to last item, press "Finish" → time recorded, transitions to summary view
7. **Accuracy**: Recorded times should be within 1 second of real elapsed time (SC-002)

### 3. Skip Items and Add Items Mid-Session (US3)

**Verifies**: FR-009, FR-010, FR-011, SC-004

1. Start a session with 3 items
2. On the first item, press "Skip" → item marked as skipped with zero time, next item becomes active
3. Timer for the new current item is unaffected by the skip action (SC-004)
4. While practising the second item, add an existing library item → new item appears at end of setlist, total count increases
5. While practising, add a brand new item (not in library) → new item created in library and added to setlist end
6. Finish the session and verify summary shows: skipped item (zero time), practised items (with times), added items

### 4. End Session Early (US2, edge case)

**Verifies**: FR-023

1. Start a session with 5 items
2. Complete 2 items (press "Next" twice)
3. On the 3rd item, choose "End Session Early"
4. Verify summary shows:
   - Items 1–2: completed with recorded times
   - Item 3: completed with time up to the point of ending
   - Items 4–5: marked as "not attempted"
5. Session completion status shows "ended early"

### 5. End-of-Session Summary with Notes (US4)

**Verifies**: FR-012, FR-013, FR-014, FR-015

1. Complete a session (via step 2 above)
2. Summary view shows: total session duration, each item with individual time, skipped items marked
3. Add a note to one specific item → note saved with that entry
4. Add an overall session note → note saved with session
5. Save the session
6. Close and reopen the app → the saved session appears in history with all data intact (SC-003)

### 6. Session History Display (FR-024)

**Verifies**: FR-024

1. Complete 2–3 sessions (mix of fully completed and ended early)
2. Navigate to session history
3. Each session shows: date, total duration, number of items practised, completion status
4. Sessions appear in reverse chronological order (newest first)

### 7. Per-Item Practice Summary (FR-025)

**Verifies**: FR-025

1. Complete sessions that include the same library item in multiple sessions
2. Navigate to that item's detail view
3. Practice summary shows aggregated data: total sessions where item was practised, total minutes across all sessions
4. Verify skipped entries are NOT counted in practice summary (only completed entries)

### 8. Old Data Wipe and Code Removal (US5)

**Verifies**: FR-016, FR-017, SC-007

1. **Code removal**: Verify no references to old `Session` struct, `SessionEvent` enum, `LogSession`, `UpdateSession` (old), `SessionsData` (old schema) remain in codebase
   ```bash
   # Should return no results (excluding spec/plan docs and migration code)
   grep -r "LogSession\|UpdateSession\|SessionEvent::Log\|SessionEvent::Update\|SessionEvent::Delete" crates/ --include="*.rs" | grep -v "// REMOVED" | grep -v test
   ```
2. **Data wipe**: Set `intrada:sessions` in localStorage to old-format data, reload the app → old data is wiped, new empty session store initialised
3. **New schema**: After wipe, `intrada:sessions` contains `{"sessions":[]}` with new schema structure

### 9. Session Recovery After Browser Close (FR-020)

**Verifies**: FR-020, SC-005

1. Start a session with 3 items
2. Complete 1 item, advance to 2nd item
3. Close the browser tab (or press F5 to refresh)
4. Reopen the app → session-in-progress is recovered
5. Current item and accumulated times are intact
6. Continue and complete the session normally

### 10. Edge Cases

**Verifies**: Edge cases from spec

1. **All items skipped**: Start a session, skip every item → session saveable with zero total time
2. **Duplicate items**: Add the same library item twice to a setlist → both entries tracked independently with separate times and notes
3. **Deleted library item**: Complete a session, then delete a library item that was in the session → session history still shows the item's title (snapshot)
4. **Single-item setlist**: Create a session with 1 item → works correctly (no "Next" needed, only "Finish")

---

## Success Criteria Checklist

| SC | Criteria | How to verify |
|----|----------|---------------|
| SC-001 | Build setlist in <30s | Time the flow in step 1 |
| SC-002 | Time accuracy ±1s | Compare displayed time with stopwatch in step 2 |
| SC-003 | Data persists across reload | Close/reopen in step 5 |
| SC-004 | Mid-session mods don't reset timer | Observe timer continuity in step 3 |
| SC-005 | Recovery after refresh | Step 9 |
| SC-006 | All tests pass | `cargo test` — zero failures |
| SC-007 | Zero old model references | Grep check in step 8 |
| SC-008 | Data model supports future analytics | Verify per-item times, notes, skip status, snapshots all persisted |
