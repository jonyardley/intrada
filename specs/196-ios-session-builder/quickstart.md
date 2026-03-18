# Quickstart: iOS Session Builder

**Feature**: #196 — iOS Session Builder
**Branch**: `196-ios-session-builder`

## Prerequisites

- Xcode 16+ with iOS 17.0+ SDK
- Rust toolchain with iOS targets (`just ios` builds successfully)
- Running API server (`just api` or `INTRADA_API_URL` pointing to production)
- At least 3 library items created (for testing the builder)

## Build & Run

```bash
# 1. Build Rust core for iOS simulator + generate Swift types
just ios-sim

# 2. Validate Swift compiles
just ios-swift-check

# 3. Run smoke test (launches on simulator, checks for crash)
just ios-smoke-test

# 4. Open in Xcode for development
open ios/Intrada.xcodeproj
```

## Verification Steps

### V1: Builder Opens (FR-001, FR-002, FR-018)

1. Launch app, sign in, navigate to **Practice** tab
2. Tap **"New Session"**
3. ✅ Session builder appears with library items listed
4. ✅ Bottom bar shows "0 items" with disabled Start Session button

### V2: Tap-to-Queue (FR-003, FR-016)

1. Tap any library item in the list
2. ✅ Item shows accent left bar and check-circle icon (selected state)
3. ✅ Bottom bar updates to "1 item"
4. Tap the same item again
5. ✅ Item returns to unselected state (+ icon)
6. ✅ Bottom bar updates to "0 items"

### V3: Start Session (FR-010, FR-011)

1. Select 2-3 items
2. ✅ Bottom bar shows item count and "Start Session →" button (enabled)
3. Tap **"Start Session →"**
4. ✅ App transitions to active session state (placeholder view for now)

### V4: Setlist Sheet — iPhone (FR-004, FR-005, FR-009, FR-017, FR-022)

1. Select 3 items, tap the count area of the bottom bar
2. ✅ Bottom sheet opens with "Your Setlist" heading
3. ✅ Session intention field is visible
4. ✅ Entries have drag handles and × remove buttons
5. Drag an entry to reorder
6. ✅ Entry moves to new position
7. Tap an entry to expand
8. ✅ Duration/intention/reps fields appear (progressive disclosure)
9. Tap × on an entry
10. ✅ Entry removed, library row returns to unselected

### V5: iPad Split View (FR-001 on iPad)

1. Run on iPad or iPad simulator
2. Navigate to Practice tab, tap "New Session"
3. ✅ Split view: library list on left, setlist panel on right
4. Tap items on left → ✅ they appear in right panel
5. Drag to reorder in right panel → ✅ works
6. ✅ "Start Session" button visible at bottom of right panel

### V6: Search Filter (FR-021)

1. Type in the search bar
2. ✅ Library list filters to matching items
3. Clear search
4. ✅ Full library list returns

### V7: State-Driven Navigation (FR-013, FR-018, FR-020)

1. Start building a session (select some items)
2. Switch to Library tab, then back to Practice tab
3. ✅ Builder is still showing with items selected (state preserved)
4. Start the session
5. ✅ Practice tab now shows active session (not builder)

### V8: Tab Bar Indicator (FR-019)

1. Start building a session
2. Navigate to Library tab
3. ✅ Practice tab icon shows a visual indicator (accent dot)

### V9: Cancel Building (FR-012)

1. Open builder, select some items
2. Tap "< Practice" back link
3. ✅ Returns to session list, building state cleared

## Common Issues

| Issue | Fix |
|-------|-----|
| Types not found | Run `just typegen` then `just ios-swift-check --clean` |
| Stale cache | `just ios-swift-check --clean` |
| Smoke test crash | Check `.environment()` ordering — modifiers reading `@Environment` must be applied before the injector |
| Events not dispatching | Verify `IntradaCore.processEvent()` is called and effects are processed |
