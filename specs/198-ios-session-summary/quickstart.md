# Quickstart: iOS Session Summary & History Verification

## Prerequisites

- `just ios` or `just ios-sim` completed (Rust cross-compilation)
- Simulator running

## Build Verification

```bash
just ios-swift-check       # Must pass
just ios-smoke-test        # Must pass
just ios-preview-check     # Must pass
```

## Manual Verification Steps

### 1. Session Summary (P1)

1. Start and complete a 3-item session (with scores/tempo from transition prompts)
2. **Verify**: Summary shows "Session Complete!" header, total duration, "3 items", "Completed" status
3. **Verify**: Each item shows status icon, title, type badge, duration, score badge, tempo badge
4. **Verify**: Skipped items show "Skipped" status with muted styling
5. Tap a different score dot on an item → **verify** score updates visually
6. Type in session notes → **verify** text persists
7. Tap "Save Session" → **verify** returns to history, session appears in list

### 2. Discard Flow

1. Complete a session → on summary, tap "Discard"
2. **Verify**: Confirmation dialog appears
3. Confirm → **verify** returns to idle/history, session NOT in list

### 3. Session History (P2)

1. Save 2+ sessions across different days
2. Go to Practice tab (idle) → **verify** session list with date grouping
3. **Verify**: Each card shows duration, item count, time, completion status
4. **Verify**: "Ended Early" sessions show warm accent badge
5. Tap a session → **verify** detail view with full item list

### 4. Delete Session

1. On history list, swipe left on a session
2. **Verify**: Delete option appears
3. Confirm delete → **verify** session removed from list

### 5. Empty State

1. Delete all sessions (or fresh account)
2. **Verify**: Empty state with "Start your first session" message and "New Session" CTA

### 6. iPad Layout

1. Run on iPad simulator
2. Complete a session → **verify** summary split view (stats left, entries right)
3. Go to history → **verify** master-detail layout
