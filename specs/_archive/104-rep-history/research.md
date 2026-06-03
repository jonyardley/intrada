# Research: Rep History Tracking

**Feature**: 104-rep-history
**Date**: 2026-02-21

## R1: Storage Format for Rep History Sequence

**Decision**: Store `rep_history` as a JSON-serialised TEXT column in SQLite (`Option<String>` in the database, `Option<Vec<RepAction>>` in Rust).

**Rationale**:
- The history is a small, ordered sequence (typically <50 actions per item) that is always read and written as a whole unit.
- SQLite TEXT with JSON is the simplest approach — no new table, no join, no foreign keys.
- serde_json serialisation/deserialisation is already available in the workspace (serde 1 + serde_json used by axum).
- The existing rep fields (rep_target, rep_count, rep_target_reached) are individual columns. The history is fundamentally different — it's a variable-length sequence — so a JSON TEXT column is more appropriate than trying to encode it as fixed columns.
- Query performance is not a concern: we never query *within* the history (e.g., "find sessions where the user missed 3 times"). We always read the full entry.

**Alternatives considered**:
- **Separate `rep_actions` table** with one row per action: Normalised but adds query complexity (joins), migration complexity, and N+1 risk. Rejected because the data is always read/written as a unit and is small.
- **Comma-separated string** (e.g., "G,M,G,G,G"): Simpler than JSON but less extensible (what if we want to add timestamps per action later?). Rejected in favour of proper JSON structure.
- **Binary/BLOB encoding**: No advantage over JSON TEXT for this data size. Harder to debug.

## R2: RepAction Enum Design

**Decision**: Define `RepAction` as a `#[repr(i8)]` enum with two variants: `Missed = -1` and `Success = 1`. Serialise with `serde_repr` as signed integers representing the delta applied to the count.

**Rationale**:
- Values represent deltas: `1` = count + 1, `-1` = count − 1. This is semantically cleaner than arbitrary identifiers.
- Enables simple analytics on the raw array: sum for net progress, running total for sparkline charts, count of `-1`s for total misses, longest streak of `1`s for best consecutive run.
- UI labels ("Got it", "Missed") are free to change without migrating stored data.
- Extensible: a future neutral/partial state could use `0`, which makes intuitive sense as "no change to count".
- Integer serialisation produces compact JSON: `[1,1,-1,1,1,1]`.
- The enum lives in `intrada-core/src/domain/session.rs` alongside `SetlistEntry` since it's a domain type.
- Requires `serde_repr` crate (lightweight, serde ecosystem).

**Alternatives considered**:
- **String-serialised enum** (`"got_it"`, `"missed"`): Readable but couples storage to label naming. Requires migration if labels change. Rejected.
- **Bool vec** (`Vec<bool>` where true=success, false=missed): Compact but loses semantic clarity and is harder to extend beyond two states. Rejected.
- **String vec** (`Vec<String>`): Loses type safety. Rejected.

## R3: Hide/Show Semantics vs Enable/Disable

**Decision**: Redefine `DisableRepCounter` to hide the UI without clearing state. Introduce a new concept: the counter has a `visible` flag that controls UI rendering, but the underlying rep state (target, count, reached, history) persists on the entry.

**Implementation approach**: Rather than adding a separate `rep_visible` field, we reuse the existing `EnableRepCounter`/`DisableRepCounter` events but change the `DisableRepCounter` handler to only toggle visibility. The simplest approach is:

- Add a `rep_counter_hidden: bool` field to `SetlistEntry` (not persisted to DB — it's ephemeral UI state for the current session only).
- `DisableRepCounter` sets `rep_counter_hidden = true` without touching rep_target/count/reached/history.
- `EnableRepCounter` sets `rep_counter_hidden = false`. If no rep state exists yet, it initialises defaults.
- The ViewModel uses `rep_counter_hidden` to decide whether to render the counter Card.

**Wait — reconsidering**: Adding an ephemeral field to a domain type that gets serialised feels wrong per the constitution (state boundary rules). The `rep_counter_hidden` flag is UI interaction state, not domain state.

**Revised decision**: Keep it simpler. Change `DisableRepCounter` to simply not clear the rep fields. The UI shows the counter whenever `rep_target.is_some()`. To "hide" the counter, we could either:
1. Leave the domain handler unchanged and just not clear, or
2. Add a separate `rep_counter_visible` signal in Leptos (UI state)

Option 1 is simpler and matches the spec: "hiding the counter preserves the count". The user taps "Disable counter" → nothing happens to the data, the counter just stops showing. But wait — if `rep_target` is still `Some`, the counter would still render because the UI checks `rep_target.is_some()`.

**Final decision**: Change the approach:
- `DisableRepCounter` now sets a new `rep_counter_active: bool` field to `false` on the entry. This field defaults to `true` when a rep target exists.
- The UI renders the counter when `rep_target.is_some() && rep_counter_active`.
- `EnableRepCounter` sets `rep_counter_active = true` and initialises defaults only if `rep_target.is_none()`.
- `rep_counter_active` IS persisted (it's meaningful for crash recovery), but NOT persisted to the API/DB — when a session is saved, we only care about the final rep state, not whether the counter was visible.

**Even simpler final decision**: Don't add a new field. Just use Leptos signal state for visibility. The domain model doesn't need to know if the counter is hidden — that's purely a UI concern. The `DisableRepCounter` event simply stops clearing state. The `EnableRepCounter` event simply stops resetting count/reached when state already exists. The UI in `session_timer.rs` uses a Leptos `RwSignal<bool>` to track whether the counter panel is shown. This respects the state boundary rule in CLAUDE.md: "UI state that has no meaning outside the current view stays in Leptos signals."

**Summary**:
- Remove the `DisableRepCounter` and `EnableRepCounter` events from Crux core (they no longer modify domain state).
- Counter visibility is a Leptos signal toggled by the button.
- On first show, if `rep_target` is None, fire a new `InitRepCounter` event that sets the defaults (target=5, count=0, reached=false, history=[]).
- On subsequent shows, no event needed — the state is already there.
- The "hide" action is purely a Leptos signal flip. No Crux event, no effect.

**Wait — crash recovery**: If the user hides the counter and the app crashes, the counter will reappear on recovery (since visibility is a signal, not persisted). This is acceptable — the data is still there, just the UI state resets. The user sees the counter on recovery, which is actually better (they don't lose awareness of their rep data).

**Alternatives considered**:
- **New `rep_counter_active` field on SetlistEntry**: Adds a persisted field for UI state. Rejected — violates state boundary principle.
- **Rename events to ShowRepCounter/HideRepCounter**: Unnecessary if we remove the events entirely. Simplest is Leptos signal.

## R4: Icon Choice for Enable Button

**Decision**: Use the Unicode character `🔄` (U+1F504, ANTICLOCKWISE DOWNWARDS AND UPWARDS OPEN CIRCLE ARROWS) as the icon prefix on the "Rep Counter" button.

**Rationale**:
- Consistent with the app's existing approach of using Unicode characters for inline icons (e.g., "✓" for got-it, "✗" for missed).
- No SVG or icon library needed.
- The repeat/cycle symbol visually communicates "repetition".

**Alternatives considered**:
- `🔁` (CLOCKWISE RIGHTWARDS AND LEFTWARDS OPEN CIRCLE ARROWS): Similar but less common.
- `↻` (ANTICLOCKWISE OPEN CIRCLE ARROW): Too subtle at small sizes.
- SVG icon: Overengineered for a single button icon.

## R5: Attempt Count Display Logic

**Decision**: In the session summary, show attempt count only when it differs from the rep target. Format: "Reps: 3 / 5 ✓ · 6 attempts" (with the attempt count as muted text).

**Rationale**:
- A clean run (5 taps to reach 5/5) provides no extra information — showing "5 attempts" is redundant.
- A volatile run (15 taps to reach 5/5) is interesting — "15 attempts" tells the user they had to work hard.
- The `·` separator keeps it visually lightweight.

**Alternatives considered**:
- Always show attempt count: Adds noise for clean runs. Rejected.
- Show as a separate line: Takes too much vertical space. Rejected.
- Show a mini sparkline of the history: Too complex for initial implementation. Can be added later using the persisted history data.
