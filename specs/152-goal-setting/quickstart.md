# Quickstart: Basic Goal Setting

**Feature**: 152-goal-setting
**Date**: 2026-02-24

## Prerequisites

- Rust stable (1.89.0+)
- trunk 0.21.x
- Running API server with Turso credentials

## Build & Test

```bash
# Full workspace check
cargo fmt --check
cargo clippy -- -D warnings
cargo test

# Core tests only (domain, validation, progress computation)
cargo test -p intrada-core

# API tests only (DB, routes, integration)
cargo test -p intrada-api
```

## Verification Steps

### 1. Goal CRUD via API

```bash
# Start API server
cd crates/intrada-api && cargo run

# Create a frequency goal
curl -X POST http://localhost:3001/api/goals \
  -H "Content-Type: application/json" \
  -d '{"title":"Practise 5 days per week","kind":{"type":"session_frequency","target_days_per_week":5}}'

# List goals
curl http://localhost:3001/api/goals

# Update goal (change target)
curl -X PUT http://localhost:3001/api/goals/{id} \
  -H "Content-Type: application/json" \
  -d '{"title":"Practise 6 days per week"}'

# Delete goal
curl -X DELETE http://localhost:3001/api/goals/{id}
```

### 2. Goal Types (create one of each)

```bash
# Session Frequency
curl -X POST http://localhost:3001/api/goals \
  -H "Content-Type: application/json" \
  -d '{"title":"Practise 5 days per week","kind":{"type":"session_frequency","target_days_per_week":5}}'

# Practice Time
curl -X POST http://localhost:3001/api/goals \
  -H "Content-Type: application/json" \
  -d '{"title":"Practise 120 minutes per week","kind":{"type":"practice_time","target_minutes_per_week":120}}'

# Item Mastery (replace ITEM_ID with real item)
curl -X POST http://localhost:3001/api/goals \
  -H "Content-Type: application/json" \
  -d '{"title":"Master Chopin Nocturne","kind":{"type":"item_mastery","item_id":"ITEM_ID","target_score":4}}'

# Milestone
curl -X POST http://localhost:3001/api/goals \
  -H "Content-Type: application/json" \
  -d '{"title":"Memorise first movement","kind":{"type":"milestone","description":"Memorise the first movement of Moonlight Sonata"}}'
```

### 3. UI Verification

1. Start the web shell: `cd crates/intrada-web && trunk serve`
2. Navigate to `/goals` — should show empty state with "Set a Goal" CTA
3. Create each goal type via the form
4. Verify progress bars update after completing a practice session
5. Complete a goal — verify it moves to history section
6. Archive a goal — verify "Reactivate" action appears
7. Check `/` (library page) — active goals summary card should appear
8. Check navigation — Goals tab visible on both mobile and desktop

### 4. Constitution Compliance

- [ ] Pure core: `cargo test -p intrada-core` passes without WASM dependencies
- [ ] Shell isolation: No browser APIs in intrada-core
- [ ] Validation sharing: Constants in `validation.rs`, used by both core and API
- [ ] Design tokens: No raw Tailwind colours in new views
- [ ] Accessibility: All form inputs have labels, interactive elements have ARIA attributes
- [ ] Inclusive design: Progress language is positive, process-focused, no shaming
