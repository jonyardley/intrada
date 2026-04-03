# Quickstart: iOS Active Session Verification

## Prerequisites

- `just ios` completed successfully (Rust cross-compilation + typegen + UniFFI)
- Xcode open with `ios/Intrada.xcodeproj`
- Simulator running (iPhone 15 Pro or iPad Air)

## Build Verification

```bash
just ios-swift-check       # Must pass — compile all Swift files
just ios-smoke-test        # Must pass — no runtime crashes on launch
just ios-preview-check     # Must pass — all #Preview blocks compile
```

## Manual Verification Steps

### 1. Basic Session Flow (P1)

1. Launch app → sign in → go to Practice tab
2. Tap "New Session" → add 3 items to setlist
3. Set a 1-minute duration on item 1
4. Tap "Start Session"
5. **Verify**: Active session view appears with:
   - "ITEM 1 OF 3" label
   - Item title prominent
   - Progress ring with countdown timer (subtle, ghosted ring)
   - Rep counter (if item has rep target)
   - "Next Item" and "End Early" buttons
   - No navigation bar or tab bar visible

### 2. Timer & Progress Ring (P2)

1. Watch the countdown timer tick down
2. **Verify**: Ring fills smoothly, timer text is readable but subtle
3. When timer reaches 0, **verify**: Transition prompt appears as bottom sheet
4. For an item without a planned duration, **verify**: Elapsed time shown (no ring)

### 3. Transition & Scoring (P3)

1. On transition prompt, **verify**: "Up Next" shows next item title
2. Tap score dot (e.g., 4) → **verify**: visually selected
3. Enter tempo (e.g., 120) → **verify**: field accepts input
4. Enter notes → **verify**: field accepts input
5. Tap "Continue" → **verify**: advances to next item, timer resets
6. On last item, tap "Finish" → **verify**: transitions to summary

### 4. Rep Counter (P4)

1. Start a session with an exercise that has a rep target
2. **Verify**: "0 / 5" counter displayed with Got it / Missed buttons
3. Tap "Got it" 3 times → **verify**: "3 / 5"
4. Tap "Missed" → **verify**: "2 / 5"
5. Reach target → **verify**: celebration state shown

### 5. Pause / End Early / Abandon (P5)

1. Tap pause icon → **verify**: pause overlay with Resume / End Early / Abandon
2. Tap "Resume" → **verify**: session continues, timer resumes
3. Tap "End Early" → **verify**: confirmation dialog
4. Confirm → **verify**: transitions to summary with remaining items as "not attempted"
5. (Separate test) Tap "Abandon" → confirm → **verify**: returns to idle Practice tab

### 6. Crash Recovery (P6)

1. Start a session, advance to item 2
2. Force-quit the app (swipe up from app switcher)
3. Relaunch → **verify**: Active session resumes at item 2 (timer resets to 0)

### 7. iPad Layout (P7)

1. Run on iPad simulator
2. Start a session → **verify**: Split view with sidebar (setlist + stats) and focus area
3. **Verify**: Sidebar shows completed/active/pending items with visual state indicators

### 8. Tab Bar Indicator

1. During active session, switch to Library tab
2. **Verify**: Practice tab shows accent-coloured indicator (dot or icon colour)
3. Tap Practice tab → **verify**: Returns to active session
