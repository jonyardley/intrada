# Quickstart: Tempo Progress Charts

**Feature Branch**: `151-tempo-progress-charts`

## Prerequisites

- Rust stable (1.89.0+)
- trunk 0.21.x
- Running API server with Turso database containing session data with achieved tempos

## Build & Run

```bash
# Run all tests (should pass before and after changes)
cargo test

# Start API server (separate terminal)
cd crates/intrada-api && cargo run

# Start web dev server
cd crates/intrada-web && trunk serve
```

## Verification Steps

### V1: Chart renders with tempo data

1. Open the app and navigate to the library
2. Select an item that has been practised with achieved tempos logged across multiple sessions
3. Scroll to the "Practice Summary" card
4. **Verify**: A line chart appears in the "Tempo Progress" section showing data points connected by lines
5. **Verify**: Each data point corresponds to a recorded tempo from a past session

### V2: Target reference line displays

1. View an item that has a target BPM set (e.g., "120 BPM" in the tempo field)
2. **Verify**: A dashed horizontal line appears at the target BPM level
3. **Verify**: The target line is visually distinct from the data line (different colour and dashed)
4. **Verify**: A small label identifies it as "Target"

### V3: Progress percentage

1. View an item with both a target BPM and at least one achieved tempo
2. **Verify**: A progress percentage appears above the chart (e.g., "75% of target")
3. If latest tempo exceeds target, **verify** percentage shows above 100% (e.g., "108% of target")

### V4: Empty state

1. View an item with no tempo history (never practised with tempo logging)
2. **Verify**: No chart is shown
3. **Verify**: An appropriate empty state message appears or the tempo section is hidden

### V5: Single data point

1. View an item with exactly 1 tempo recording
2. **Verify**: A single data point (circle) is plotted on the chart without a connecting line

### V6: Tooltip interaction

1. On desktop, hover over a data point on the tempo chart
2. **Verify**: A tooltip shows the session date and exact BPM value
3. On mobile (or browser dev tools mobile emulation), tap a data point
4. **Verify**: The same information is accessible

### V7: No target BPM

1. View an item that has tempo history but no target BPM set
2. **Verify**: The chart renders correctly without a target reference line
3. **Verify**: No progress percentage is shown

### V8: Existing functionality unaffected

1. Navigate to the Analytics page
2. **Verify**: The 28-day practice history chart still renders correctly
3. Navigate to any item detail page
4. **Verify**: Score history section still displays correctly
5. **Verify**: All other detail page sections are unaffected
