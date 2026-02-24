# Quickstart: Focus Mode

**Date**: 2026-02-23
**Feature**: 048-focus-mode

## Verification Steps

After implementation, verify the feature works end-to-end:

### 1. Per-Item Duration in Builder

1. Navigate to `/sessions/new`
2. Add 2–3 items to the setlist
3. Verify each item shows an "Add duration" link (like rep target)
4. Set a duration of 2 minutes on the first item
5. Verify the duration displays as "2 min" on the entry
6. Remove the duration — verify it reverts to no duration
7. Set durations on some items, leave others without

### 2. Focus Mode — Minimal UI

1. Start the session (click "Start Session")
2. Verify the navigation bar (header) is **not visible**
3. Verify the bottom tab bar (mobile) is **not visible**
4. Verify the completed items list is **not visible**
5. Verify only these elements are on screen:
   - Current item name + type badge
   - Timer (progress ring if duration set, digital timer in centre)
   - Rep counter (if active for this item)
   - Session controls (Next Item / Skip / End Early)
   - Toggle button (chevron icon)

### 3. Progress Ring

1. On an item **with** a planned duration (e.g., 2 min):
   - Verify a circular progress ring is visible
   - Verify the digital timer (MM:SS) is centred inside the ring
   - Wait 1 minute — verify the ring shows ~50% filled
2. On an item **without** a planned duration:
   - Verify no progress ring is shown
   - Verify only the digital timer is visible

### 4. Toggle Button — Reveal/Hide

1. While in focus mode, click the toggle button (chevron)
2. Verify the navigation bar reappears
3. Verify the completed items list appears
4. Verify the session intention text reappears
5. Click the toggle button again
6. Verify the UI returns to the focused minimal state

### 5. Transition Prompt

1. Set a 1-minute duration on an item and start the session
2. Wait for the 1-minute duration to elapse
3. Verify a transition prompt appears (e.g., "Up next: [Next Item Name]")
4. Verify the prompt does **not** block the timer or controls
5. Continue to the next item — verify the prompt clears
6. On the **last item**, verify the prompt says something like "Session complete"

### 6. Items Without Duration

1. Start a session where the first item has no planned duration
2. Verify no progress ring is shown
3. Verify no transition prompt fires (timer just counts up)
4. Verify all controls work normally

### 7. Crash Recovery

1. Start a session with durations set on items
2. Close the browser tab
3. Re-open the app — verify crash recovery restores the session
4. Verify focus mode is active (focused state, not expanded)
5. Verify planned durations are preserved

### 8. Session Persistence

1. Complete a full session with planned durations
2. View the session in the sessions list
3. Verify the session saved correctly (items, scores, durations)
4. Verify planned duration data persists in the API

## Build & Test Commands

```bash
# Run all workspace tests
cargo test

# Run API tests (includes new migration + duration field tests)
cargo test -p intrada-api

# Run core tests (includes new event + validation tests)
cargo test -p intrada-core

# Lint and format checks
cargo clippy -- -D warnings
cargo fmt --check

# Build web shell
cd crates/intrada-web && trunk build
```
