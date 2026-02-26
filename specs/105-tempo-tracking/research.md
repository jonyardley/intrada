# Research: Tempo Tracking

**Feature**: 105-tempo-tracking
**Date**: 2026-02-24

## Research Questions

### RQ-1: How does the existing score tracking pattern work, and how should achieved tempo mirror it?

**Decision**: Follow the exact same pattern as `score: Option<u8>` on `SetlistEntry`.

**Rationale**: Score tracking is a proven, well-tested pattern in the codebase. It stores an optional value per session entry, aggregates into `ItemPracticeSummary` via `build_practice_summaries()`, and displays in both the summary phase and item detail view. Achieved tempo has identical semantics: an optional numeric value per completed entry, aggregated over time per item.

**Key pattern elements**:
- `SetlistEntry.score: Option<u8>` → add `achieved_tempo: Option<u16>`
- `#[serde(default)]` for backward compatibility with existing sessions
- `SessionEvent::UpdateEntryScore` → add `SessionEvent::UpdateEntryTempo`
- `ScoreHistoryEntry` → add `TempoHistoryEntry`
- `ItemPracticeSummary.latest_score` / `score_history` → add `latest_tempo` / `tempo_history`
- Same gating: only available for completed entries (`EntryStatus::Completed`)

**Alternatives considered**:
- Separate tempo tracking struct: Rejected — adds complexity without benefit, diverges from established patterns
- Store as part of existing `Tempo` struct: Rejected — `Tempo` is an item-level concept (target), not session-level (achieved)

### RQ-2: What validation range should achieved tempo use?

**Decision**: New constants `MIN_ACHIEVED_TEMPO: u16 = 1` and `MAX_ACHIEVED_TEMPO: u16 = 500`, separate from the existing item BPM range.

**Rationale**: The spec defines achieved tempo as 1–500 BPM. The existing `MIN_BPM`/`MAX_BPM` constants (1–400) govern the *target* tempo on library items (the `Tempo` struct). These are distinct concepts:
- **Target tempo** (item-level): the written/intended tempo for the piece, typically within standard metronome markings (1–400 covers Grave through Prestissimo)
- **Achieved tempo** (session entry-level): the tempo the musician actually played, which can exceed the written tempo (e.g., technique exercises deliberately pushed fast, or overshooting the target)

Using separate constants avoids confusion and allows each to evolve independently.

**Alternatives considered**:
- Reuse `MIN_BPM`/`MAX_BPM` (1–400): Rejected — spec explicitly requires 500 upper bound, and the concepts are semantically different
- No upper bound: Rejected — 500 BPM is already extremely generous (metronomes rarely go above 300); an uncapped field invites data entry errors

### RQ-3: How should achieved tempo integrate with the precomputed practice summaries cache?

**Decision**: Extend the existing `build_practice_summaries()` function to also collect tempo history in the same single pass.

**Rationale**: Issue #150 introduced a `HashMap<String, ItemPracticeSummary>` cache that is rebuilt at three mutation points (sessions loaded, session saved, session deleted). Adding tempo tracking to this same function means:
- No new cache mechanism needed
- No additional mutation points to track
- Single pass over sessions collects both score and tempo data
- The `ItemPracticeSummary` struct simply gains two new fields

**Implementation detail**: The accumulator tuple in `build_practice_summaries()` changes from `(usize, u64, Vec<ScoreHistoryEntry>)` to `(usize, u64, Vec<ScoreHistoryEntry>, Vec<TempoHistoryEntry>)`.

**Alternatives considered**:
- Separate tempo cache: Rejected — doubles the cache management code for no benefit
- Compute tempo history on demand: Rejected — this is exactly what #150 fixed for score history

### RQ-4: How should the library list display achieved tempo alongside target tempo?

**Decision**: Add `latest_achieved_tempo: Option<u16>` to `LibraryItemView`. The web shell formats the display combining it with the existing `tempo: Option<String>` field (which shows the target).

**Rationale**: The view model already carries the formatted target tempo as `tempo: Option<String>`. Adding the latest achieved tempo as a separate field keeps the view model clean and lets the shell decide formatting. The spec describes a "Tempo Badge" showing "108 / 120 BPM" — this is a shell-level formatting concern.

**Alternatives considered**:
- Pre-format in core as a combined string: Rejected — loses the individual values for conditional rendering (e.g., show only target if no achieved, show only achieved if no target)
- Add a dedicated `TempoBadge` view struct: Rejected — over-engineering for two optional numbers

### RQ-5: What database migration is needed?

**Decision**: Single `ALTER TABLE setlist_entries ADD COLUMN achieved_tempo INTEGER` migration.

**Rationale**: Follows the same incremental migration pattern used for `score`, `intention`, `rep_target`, `rep_count`, `rep_target_reached`, `rep_history`, and `planned_duration_secs`. A nullable INTEGER column with no default is the lightest-weight change. Existing rows get NULL (no achieved tempo), which maps to `Option<u16> = None` in Rust.

**Alternatives considered**:
- Separate tempo tracking table: Rejected — achieved tempo is a property of the session entry, not a separate entity
- JSON blob column: Rejected — a single integer value doesn't warrant JSON overhead

### RQ-6: Where in the session summary UI should the tempo input appear?

**Decision**: Below the score selector (confidence buttons) for each completed entry, as a compact numeric input field. Label: "Achieved tempo (BPM)".

**Rationale**: The spec places it "below the existing score selector". This maintains the summary phase's top-to-bottom flow: status → intention → duration → reps → score → tempo → notes. The numeric input uses the existing `TextField` component with `input_type="number"`.

**Alternatives considered**:
- Inline with score buttons: Rejected — mixing a text input with button row creates visual clutter
- Separate tempo section: Rejected — the per-entry pattern (score, notes, etc.) groups all entry data together
