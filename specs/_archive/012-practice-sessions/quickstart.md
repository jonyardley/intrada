# Quickstart: Practice Sessions

**Feature**: 012-practice-sessions | **Date**: 2026-02-15

## Overview

Add practice session tracking to the Intrada music library. Sessions record practice time against library items (pieces/exercises) with duration, timestamps, and optional notes. Separate persistence via `sessions.json` (CLI) and `intrada:sessions` localStorage key (web).

## Architecture Summary

```text
intrada-core/src/
├── app.rs              # Add Session(SessionEvent), SessionsLoaded to Event enum
│                       # Add session storage variants to StorageEffect
│                       # Add sessions: Vec<Session> to Model
│                       # Add sessions: Vec<SessionView> to ViewModel
├── model.rs            # Add SessionView struct, ItemPracticeSummary
├── validation.rs       # Add validate_log_session, validate_update_session
├── domain/
│   ├── mod.rs          # Export session module
│   ├── session.rs      # NEW: Session struct, SessionEvent, handle_session_event()
│   └── types.rs        # Add SessionsData, LogSession, UpdateSession types

intrada-cli/src/
├── main.rs             # Add session CLI subcommands (log, sessions, session show/edit/delete)
├── shell.rs            # Handle new StorageEffect variants, load sessions
├── storage.rs          # Add sessions.json read/write methods
├── display.rs          # Add session display formatting

intrada-web/src/
├── app.rs              # Wire session events and views
├── core_bridge.rs      # Add sessions localStorage persistence
├── views/
│   ├── detail.rs       # Add practice history section + timer
│   └── sessions.rs     # NEW: All-sessions list view
├── components/
│   ├── session_history.rs  # NEW: Session list for item detail
│   └── practice_timer.rs   # NEW: Client-side timer component
```

## Implementation Order

### Phase 1: Core Domain (intrada-core)

1. **Session struct + types** in `domain/session.rs` and `domain/types.rs`
   - `Session`, `SessionsData`, `LogSession`, `UpdateSession` structs
   - `SessionEvent` enum

2. **Validation** in `validation.rs`
   - `validate_log_session()`, `validate_update_session()`
   - Constants: `MIN_DURATION = 1`, `MAX_DURATION = 1440`

3. **Event + Effect wiring** in `app.rs`
   - Add `Event::Session(SessionEvent)`, `Event::SessionsLoaded`
   - Add `StorageEffect::SaveSession`, `UpdateSession`, `DeleteSession`, `LoadSessions`
   - Add `sessions: Vec<Session>` to `Model`

4. **Session event handler** in `domain/session.rs`
   - `handle_session_event()` following piece/exercise pattern

5. **ViewModel** in `model.rs` and `app.rs`
   - `SessionView` struct, `ItemPracticeSummary`
   - Update `view()` to build session views and practice summaries

### Phase 2: CLI Shell (intrada-cli)

6. **Storage** in `storage.rs`
   - `load_sessions()`, `save_session()`, `update_session()`, `delete_session()`
   - Atomic writes to `sessions.json`

7. **Shell wiring** in `shell.rs`
   - Handle new `StorageEffect` variants
   - Update `load_data()` to load sessions too

8. **CLI commands** in `main.rs`
   - `intrada log <item-id> --duration <minutes> [--notes "text"]`
   - `intrada sessions [--item <id>]`
   - `intrada session show <session-id>`
   - `intrada session edit <session-id> [--duration N] [--notes "text"]`
   - `intrada session delete <session-id> [-y]`

9. **Display** in `display.rs`
   - Session list and detail formatting

### Phase 3: Web Shell (intrada-web)

10. **Storage** in `core_bridge.rs`
    - `load_sessions_data()`, `save_sessions_data()`
    - localStorage key `intrada:sessions`

11. **Session history component** in `components/session_history.rs`
    - List of sessions for a specific item

12. **Item detail integration** in `views/detail.rs`
    - Practice summary (session count, total time)
    - Session history list
    - Practice timer

13. **Practice timer** in `components/practice_timer.rs`
    - Client-side only (not in Crux core)
    - Start/stop, elapsed display, auto-log on stop

14. **All-sessions view** in `views/sessions.rs`
    - Route `/sessions` showing all sessions across library

## Key Patterns

### Event Handling Pattern (follows piece.rs)

```rust
pub fn handle_session_event(event: SessionEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        SessionEvent::Log(input) => { /* validate, create, notify shell */ }
        SessionEvent::Update { id, .. } => { /* find, validate, update, notify shell */ }
        SessionEvent::Delete { id } => { /* remove from model, notify shell */ }
    }
}
```

### Storage Effect Pattern (follows existing)

```rust
StorageEffect::SaveSession(session) => {
    self.store.save_session(&session)?;
}
```

### Web Timer (client-side only)

```rust
// Timer state lives in Leptos signals, NOT in Crux Model
let (is_running, set_running) = signal(false);
let (elapsed_secs, set_elapsed) = signal(0u32);

// On stop: round to nearest minute (>=30s rounds up), send SessionEvent::Log to core
let duration_minutes = ((elapsed_secs + 30) / 60) as u32; // round to nearest
```

## Testing Strategy

- **Unit tests**: Session creation, validation, event handling (in intrada-core)
- **CLI integration tests**: Log, list, show, edit, delete commands (in intrada-cli)
- **Edge cases**: Orphaned sessions, boundary durations (1 min, 1440 min), timer rounding

## Dependencies

No new crate dependencies. Uses existing: `ulid`, `chrono`, `serde`, `serde_json`, `web-sys`.
