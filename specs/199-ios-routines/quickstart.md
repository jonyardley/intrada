# Quickstart: iOS Routines Verification

## Build Verification

```bash
just ios-swift-check
just ios-smoke-test
```

## Manual Verification

### 1. Routine List (US1)
1. Go to Routines tab → **verify** empty state with message
2. (Create a routine via web or save-as-routine)
3. Go to Routines tab → **verify** routine shows name + item count
4. Swipe to delete → confirm → **verify** removed

### 2. Routine Detail
1. Tap a routine → **verify** detail shows name and ordered item list with type badges

### 3. Edit Routine (US2)
1. Open routine detail → tap Edit
2. Change name → **verify** field updates
3. Drag item to reorder → **verify** order changes
4. Remove an item → **verify** removed
5. Add item from library → **verify** added
6. Save → **verify** changes persist
7. Cancel → **verify** changes discarded

### 4. Load Routine (US3)
1. New Session → tap "Load Routine" in setlist
2. Select a routine → **verify** items appended to setlist
3. Add another item manually → **verify** both routine items + manual item present

### 5. Save as Routine (US4)
1. Build a 3-item setlist → tap "Save as Routine"
2. Enter name → Save → **verify** routine appears in Routines tab
3. Complete a session → on summary, tap "Save as Routine"
4. Enter name → Save → **verify** routine appears

### 6. iPad
1. Routines tab → **verify** split view (list + detail)
2. Edit → **verify** works in detail pane
