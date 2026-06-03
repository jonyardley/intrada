# Implementation Plan: Tempo Tracking

**Branch**: `105-tempo-tracking` | **Date**: 2026-02-24 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/105-tempo-tracking/spec.md`

## Summary

Add achieved tempo (BPM) tracking per session entry, following the existing score tracking pattern. Musicians can record the tempo they comfortably reached during practice, view tempo history per library item, and see the latest achieved tempo in the library list alongside the target. Implementation extends `SetlistEntry` with an `achieved_tempo: Option<u16>` field, extends the precomputed practice summaries cache with tempo history, and adds UI for input (summary phase), display (item detail), and at-a-glance (library list).

## Technical Context

**Language/Version**: Rust stable (1.89.0 in CI; MSRV 1.75+, intrada-api requires 1.78+)
**Primary Dependencies**: crux_core 0.17.0-rc2, axum 0.8, leptos 0.8.x, serde 1, chrono 0.4
**Storage**: Turso (managed libsql/SQLite) via HTTP protocol
**Testing**: cargo test (unit/integration), wasm-bindgen-test 0.3, Playwright (E2E)
**Target Platform**: WASM (web shell) + Linux server (API)
**Project Type**: Three-crate workspace (core + web + api)
**Performance Goals**: Practice summary cache build < 200ms for 10k items + 500 sessions; no perceptible delay on session save or library render
**Constraints**: Pure core (zero I/O), validation constants shared across crates, refresh-after-mutate pattern
**Scale/Scope**: ~10k library items, ~500 sessions, 3 views modified (summary, detail, library list)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Code Quality ✅

- `achieved_tempo` follows the established `score` field pattern — consistent, readable, self-documenting
- Single responsibility maintained: validation in `validation.rs`, events in `session.rs`, cache in `app.rs`, views in web shell
- New constants (`MIN_ACHIEVED_TEMPO`, `MAX_ACHIEVED_TEMPO`) are explicit, no magic numbers
- No dead code — every new type/field is used in the data flow

### II. Testing Standards ✅

- Core validation: unit tests for `validate_achieved_tempo()` boundary conditions
- Core cache: extend existing `build_practice_summaries` tests to verify tempo history aggregation
- Core events: test `UpdateEntryTempo` event handling (valid, invalid, skipped entry gating)
- API: test `achieved_tempo` persistence round-trip (insert + read back)
- Web: existing E2E patterns cover session save flow
- Performance: existing `test_performance_10k_items` includes cache with tempo data

### III. User Experience Consistency ✅

- Tempo input uses existing `TextField` component with `input_type="number"` — consistent form pattern
- Tempo history follows score history's visual pattern (date + value list in item detail)
- Library list display uses muted metadata styling consistent with key/tempo display
- Error messages use the same validation error pattern as score/intention/notes
- Only shown for completed entries — same gating as score selector

### IV. Performance Requirements ✅

- Tempo history is precomputed in `build_practice_summaries()` (same O(M×E) single pass as score history) — no additional per-render cost
- Single new INTEGER column, no index required (never queried independently)
- No additional API calls — tempo data travels with existing session payloads
- Library list renders `latest_achieved_tempo` from precomputed cache (O(1) lookup)

### V. Architecture Integrity ✅

- Pure core: `achieved_tempo` is a data field on domain types, validated in core, no I/O
- Shell isolation: UI rendering and form handling remain in `intrada-web`
- API isolation: Column addition and row parsing in `intrada-api`, no core dependency changes
- Effect-driven: `UpdateEntryTempo` event → model update, `SaveSession` → effect → API
- Validation sharing: `MIN_ACHIEVED_TEMPO`/`MAX_ACHIEVED_TEMPO` defined in core's `validation.rs`, imported by API and web

### VI. Inclusive Design ✅

- No additional decisions to start practising — tempo input is optional and appears only after session completion
- No sound, animation, or flashing elements introduced
- Numeric input with clear label and range hint — predictable interaction
- Tempo badge in library list is informational only — no pressure/streak mechanics

## Project Structure

### Documentation (this feature)

```text
specs/105-tempo-tracking/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── api.md           # API contract changes
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── intrada-core/
│   └── src/
│       ├── validation.rs        # + MIN/MAX_ACHIEVED_TEMPO, validate_achieved_tempo()
│       ├── model.rs             # + TempoHistoryEntry, extend ItemPracticeSummary, SetlistEntryView, LibraryItemView
│       ├── app.rs               # extend build_practice_summaries(), extend view()
│       └── domain/
│           └── session.rs       # + achieved_tempo on SetlistEntry, + UpdateEntryTempo event
├── intrada-api/
│   └── src/
│       ├── migrations.rs        # + ALTER TABLE setlist_entries ADD COLUMN achieved_tempo
│       ├── db/
│       │   └── sessions.rs      # + achieved_tempo in ENTRY_COLUMNS, row_to_entry, insert
│       └── routes/
│           └── sessions.rs      # + achieved_tempo validation in save_session handler
└── intrada-web/
    └── src/
        ├── validation.rs        # + client-side achieved_tempo validation
        ├── components/
        │   ├── session_summary.rs   # + tempo input field per completed entry
        │   └── library_item_card.rs # + tempo badge (achieved / target)
        └── views/
            └── detail.rs        # + tempo history section in item detail
```

**Structure Decision**: Three-crate workspace (existing). All changes are extensions to existing files — no new crates, modules, or structural changes needed.

## Complexity Tracking

> No constitution violations. All changes follow established patterns (score tracking, precomputed cache, incremental migration). No complexity justification needed.
