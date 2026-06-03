# Quickstart: Rep History Tracking

**Feature**: 104-rep-history
**Date**: 2026-02-21

## Verification Steps

After implementation, verify the feature works end-to-end:

### 1. Core Logic (cargo test)

```bash
cargo test -p intrada-core
```

Verify these new/modified tests pass:
- `test_rep_history_appended_on_got_it` — history grows with each Got it
- `test_rep_history_appended_on_missed` — history grows with each Missed
- `test_rep_history_frozen_on_next_item` — history stops growing after transition
- `test_rep_history_persisted_through_save` — history included in save payload
- `test_rep_history_none_without_counter` — no history when counter never enabled
- `test_enable_preserves_existing_state` — re-enabling doesn't reset count/history
- `test_disable_preserves_state` — hiding counter doesn't clear rep fields
- `test_rep_history_initialised_on_first_enable` — empty vec on first enable

### 2. API Validation (cargo test -p intrada-api)

```bash
cargo test -p intrada-api
```

Verify:
- Sessions with `rep_history` save and load correctly
- Sessions without `rep_history` (backward compat) still work
- Invalid `rep_history` without `rep_target` returns 400

### 3. Manual UI Verification

1. Start a new practice session with at least one item
2. Tap "Rep Counter" button — verify icon is visible, counter initialises at 0/5
3. Tap "Got it" 3 times, "Missed" once, "Got it" 3 times — verify count tracks correctly
4. Tap "Rep Counter" button to hide — verify counter panel hides
5. Tap "Rep Counter" button to show — verify count is still 3/5 (or whatever the current state is)
6. Move to next item or finish session
7. In session summary, verify attempt count shows (e.g. "7 attempts") for the item
8. View session in history — verify rep badge still shows count/target

### 4. Crash Recovery

1. Start a session, enable counter, tap a few times
2. Close the browser tab
3. Reopen the app — verify session recovery dialog appears
4. Resume — verify rep count AND history are intact

### 5. Full CI Check

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

All must pass.
