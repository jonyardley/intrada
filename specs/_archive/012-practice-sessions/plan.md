# Implementation Plan: Practice Sessions

**Branch**: `012-practice-sessions` | **Date**: 2026-02-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/012-practice-sessions/spec.md`

## Summary

Add practice session tracking to the Intrada music library. Sessions are a new domain entity (separate from pieces/exercises) that record practice time against library items with duration, timestamps, and notes. Storage follows the segmented JSON pattern from 011-json-persistence: `sessions.json` (CLI) and `intrada:sessions` localStorage key (web). The web shell adds a practice timer (client-side only, not in Crux core). The core domain handles session CRUD with the same per-domain event handler pattern used for pieces and exercises.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition)
**Primary Dependencies**: crux_core 0.17.0-rc2, serde/serde_json 1, ulid, chrono, clap 4.5 (CLI), leptos 0.8.x (web), web-sys with Storage+Window features (web)
**Storage**: JSON files (CLI: `~/.local/share/intrada/sessions.json`), localStorage (web: `intrada:sessions` key)
**Testing**: `cargo test` (unit + integration)
**Target Platform**: CLI (macOS/Linux), WASM (web)
**Project Type**: Workspace with 3 crates (intrada-core, intrada-cli, intrada-web)
**Performance Goals**: Session history for 100 sessions loads without perceptible delay; view() with sessions completes in <200ms
**Constraints**: Local-only storage, no network, offline-capable by design
**Scale/Scope**: Single user, hundreds to low thousands of sessions over time

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Code Quality

- **Clarity over cleverness**: Session domain follows exact same patterns as piece/exercise — no novel abstractions. PASS
- **Single Responsibility**: New `session.rs` module handles session events only; storage is shell-side. PASS
- **Consistent Style**: Follows existing `cargo fmt` + `cargo clippy` rules. PASS
- **No Dead Code**: All new types and functions serve specific spec requirements. PASS
- **Explicit over Implicit**: Storage effects are explicit enum variants; timer state is explicit Leptos signals. PASS
- **Type Safety**: `u32` for duration with validation bounds; `DateTime<Utc>` for timestamps. PASS

### II. Testing Standards

- **Test Coverage**: All public functions (validate, handle_session_event, view) will have tests. PASS
- **Test Independence**: Each test creates its own model; no shared mutable state. PASS
- **Meaningful Assertions**: Tests verify domain behavior (session created, validation errors) not implementation. PASS
- **Fast Feedback**: All tests are in-process, no I/O. PASS
- **Failure Clarity**: Validation errors include field name and descriptive message. PASS
- **Contract Tests**: StorageEffect variants are tested via event handling. PASS

### III. User Experience Consistency

- **Interaction Patterns**: CLI session commands follow same patterns as library commands (add/list/show/edit/delete). PASS
- **Error Communication**: Reuses existing `LibraryError::Validation` pattern with clear messages. PASS
- **Accessibility**: Web components follow existing Tailwind + semantic HTML patterns. PASS

### IV. Performance Requirements

- **Response Time**: Session operations are O(n) on session count — acceptable for expected scale (<10k). PASS
- **Payload Efficiency**: ViewModel includes only sessions relevant to current view. PASS
- **Measurement**: Performance test for 100-session history load time. PASS

**Gate Status**: ALL PASS — no violations requiring justification.

### Post-Design Re-check

- Separate `sessions.json` file prevents session growth from affecting library read performance. PASS
- `SessionView` as separate struct (not overloading `LibraryItemView`) maintains clean separation. PASS
- Client-side timer avoids flooding Crux core with tick events. PASS

## Project Structure

### Documentation (this feature)

```text
specs/012-practice-sessions/
├── plan.md              # This file
├── research.md          # Phase 0 output — 7 research decisions
├── data-model.md        # Phase 1 output — Session, SessionsData, SessionView entities
├── quickstart.md        # Phase 1 output — implementation guide
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── intrada-core/src/
│   ├── app.rs              # Event, Effect, StorageEffect additions; update() and view() changes
│   ├── model.rs            # SessionView, ItemPracticeSummary structs; ViewModel additions
│   ├── validation.rs       # validate_log_session(), validate_update_session()
│   └── domain/
│       ├── mod.rs           # Export session module
│       ├── session.rs       # NEW: Session struct, SessionEvent, handle_session_event()
│       └── types.rs         # SessionsData, LogSession, UpdateSession types
│
├── intrada-cli/src/
│   ├── main.rs             # Session CLI subcommands
│   ├── shell.rs            # Handle new StorageEffect variants, load sessions
│   ├── storage.rs          # sessions.json read/write
│   └── display.rs          # Session display formatting
│
└── intrada-web/src/
    ├── app.rs              # Wire session events
    ├── core_bridge.rs      # Sessions localStorage persistence
    ├── views/
    │   ├── detail.rs       # Practice history + timer integration
    │   └── sessions.rs     # NEW: All-sessions list view
    └── components/
        ├── session_history.rs  # NEW: Session list for item detail
        └── practice_timer.rs   # NEW: Client-side timer component
```

**Structure Decision**: Existing Crux workspace with 3 crates. New files added within existing structure. The `session.rs` domain module follows the same pattern as `piece.rs` and `exercise.rs`. Web components added under `components/` following existing patterns.

## Complexity Tracking

> No constitution violations — table not needed.
