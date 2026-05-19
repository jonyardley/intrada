# Goals targets — per-item commitments with derived progress

> Tier 3 follow-on spec. Builds on [specs/goals.md](goals.md) (the Plan-pillar spine from #711). This spec turns Goals from "a bag of linked items with a deadline" into "a commitment with measurable targets, progress derived from session data."

## Problem

After #711, #719, #726, and #732, a Goal can be created, linked to library items, edited, completed, and deleted. But it's a write-only diary — a user can say "I want to play these pieces" but can't say *how well* or *by when* per piece, and the app has no way to show progress against the commitment.

The hard part of practising isn't picking what to play — it's knowing whether you're on track. The current model has the *what* (linked items) and the *when* (a goal-wide deadline) but is missing the *how well* (tempo, confidence) and the *per-item when* (different pieces ready at different points before the goal date). And nothing in the goal connects to the rich session data the app already collects.

This spec adds **per-item targets** to Goals, with **progress derived from existing session data** (not stored). Targets are date, confidence, and tempo. Targets inherit from goal-level defaults and can be overridden per item.

## Approach

### Data model — additive only, optional everywhere

```text
Goal {
  // existing
  id, user_id, title, notes, deadline, status, completed_at, items, photos, ...

  // new (goal-level defaults — items inherit unless overridden)
  target_confidence: Option<u8>   // 1..=5
  target_tempo:      Option<u32>  // bpm  [Phase C — deferred]
  // note: no `target_date` at goal level — `deadline` IS the goal-level date
}

GoalItem {
  // existing
  item_id, item_title, item_type

  // new (overrides goal-level defaults)
  target_date:       Option<String>  // ISO date "YYYY-MM-DD"
  target_confidence: Option<u8>      // 1..=5
  target_tempo:      Option<u32>     // bpm  [Phase C — deferred]
}
```

All new fields are nullable in DB, optional in API request bodies, optional in core types. No existing data is invalidated; un-targeted items remain valid forever.

### Inheritance — effective target is `item.foo.or(goal.foo)`

A goal-level `target_confidence = 4` means "every item targets 4 unless I say otherwise on the item." Override semantics are read-only — clearing `goal.target_confidence` doesn't cascade to clear item overrides; clearing `item.target_confidence` causes that item to fall back to the goal-level default (or to having no confidence target if the goal-level is also None).

Date is *not* inherited. `Goal.deadline` is the urgency frame for the whole goal; `GoalItem.target_date`, when set, is a per-item earlier deadline ("Bach must be ready 2 weeks before the recital"). An item without `target_date` shows the goal's `deadline` as its countdown context.

### Progress — derived, not stored

Goals store *intent*. Sessions store *evidence*. The view-model projection derives progress:

- **current_confidence** for an item = `vm.items.find(id).practice.latest_score` (already exists, see [session.rs:1892](crates/intrada-core/src/domain/session.rs)).
- **current_tempo** for an item = `vm.items.find(id).practice.latest_achieved_tempo` (already exists, see [LibraryItemView](crates/intrada-core/src/lib.rs)).
- **date status** for an item = simple comparison of `today` to `item.target_date.or(goal.deadline)`. No storage, no compute.

There is no goal/session coupling — a session is just a session. It updates `latest_score` and `latest_achieved_tempo` on each library item it touches, which in turn moves the needle on *every* goal containing that item. This is what enables cross-goal practice in a future phase without any data-model gymnastics.

### Completion — auto-suggest, manual confirm

The "looks ready" prompt fires when:

```
goal.items
  .filter(|i| i.has_any_target())   // un-targeted items are ignored
  .all(|i| effective_target_met_by_latest_session(i))
```

Where "effective target met" means:
- if `effective_target_confidence` is set: `latest_score ≥ target`
- if `effective_target_tempo` is set (Phase C): `latest_achieved_tempo ≥ target`
- if neither is set on the item: item doesn't gate the prompt

**Date is informational only** — it drives countdowns and overdue badges; it does not gate the prompt. A goal can be "ready" before its deadline; a goal past its deadline with unmet confidence targets is *overdue* not *complete*.

**Goals with no targets at all never auto-suggest.** User taps Mark Complete manually, same as today.

The user clicks "Mark Complete" — auto-suggest never auto-completes. This preserves the current handler (`GoalEvent::Complete`) unchanged.

## Key decisions

| # | Decision | Reason |
|---|---|---|
| 1 | Targets live on the `GoalItem` link, not the library item | Same piece can have different targets in different goals (Bach for the lesson at 80bpm, Bach for the recital at 120bpm) |
| 2 | Inheritance via `item.foo.or(goal.foo)` | One place to set "performance tempo for the whole recital"; cheap escape valve per piece |
| 3 | Progress derived from session data; goals store no progress fields | Goals stay as views over evidence; sessions are the single source of truth for what actually happened |
| 4 | Date is informational, not a completion gate | Date = urgency; readiness = confidence/tempo. Confusing these would mean a goal could be "complete" just by waiting |
| 5 | Un-targeted items don't gate the auto-suggest | Otherwise adding a piece with no target would silently block completion. Make absence-of-target mean absence-of-opinion |
| 6 | Latest-session-meets-target is the "looks ready" rule | Simplest defensible rule. Risks false-positives from a single good day — flagged as the **#1 open question** to revisit |
| 7 | Manual completion only — auto-suggest never auto-completes | User decides when "embedded enough to count." Removes a class of "why did my goal disappear?" surprises |
| 8 | Single migration adds all new columns | Phase B uses confidence; Phase C uses tempo; one migration is cheaper than two and won't break ordering invariants |

## Phase rollout

| Phase | Scope | Status |
|---|---|---|
| **1 (A+B bundled)** | Date target on `GoalItem` + Confidence target (goal-level default + per-item override) + per-item edit UI + countdown badges + progress display pulling `latest_score` + auto-suggest banner on completion criterion | **This spec ships with Phase 1's PR** |
| **2 (C)** | Tempo target (goal-level default + per-item override) + tempo progress from `latest_achieved_tempo` + auto-suggest extended | Separate PR |
| **3 (D)** | "Practice this goal" button on goal detail → starts a session pre-loaded with the goal's items in order of "least ready" | Separate PR; depends on session-builder reuse |
| **4 (Future)** | Cross-goal session builder ("show me what's due across all active goals") + spaced-practice / interleaving suggester | Out of scope for this spec; revisit after Phase 3 |

### Phase 1 — file inventory

- **Migration**: new sequential migration in [crates/intrada-api/src/migrations.rs](crates/intrada-api/src/migrations.rs). Adds `target_confidence INTEGER` to `goals`, adds `target_date TEXT, target_confidence INTEGER, target_tempo INTEGER` to `goal_items`. (Tempo column added now; left unused until Phase 2 — single migration is cheaper than two.)
- **Core types**: [crates/intrada-core/src/domain/types.rs](crates/intrada-core/src/domain/types.rs) — extend `Goal`, `GoalItem`, `CreateGoal`, `UpdateGoal`, `LinkGoalItem`, plus a new `UpdateGoalItem { goal_id, item_id, target_date?, target_confidence? }` request type. Add `effective_target_*` helpers on the view-model projection.
- **API endpoints**: extend `POST /goals` and `PATCH /goals/:id` to accept `target_confidence`. Extend `POST /goals/:id/items` to accept `target_date?` and `target_confidence?`. New `PATCH /goals/:id/items/:item_id` for editing a link's targets without unlinking+relinking.
- **HTTP wrapper**: [crates/intrada-core/src/http.rs](crates/intrada-core/src/http.rs) — `update_goal_item(...)` builder.
- **Validation**: [crates/intrada-core/src/validation.rs](crates/intrada-core/src/validation.rs) — `validate_target_confidence` (1..=5), `validate_target_date` (ISO-parsable).
- **UI — goal edit form**: [crates/intrada-web/src/views/goal_edit_form.rs](crates/intrada-web/src/views/goal_edit_form.rs) (introduced in #726) — add goal-level `target_confidence` field.
- **UI — goal detail**: [crates/intrada-web/src/views/goal_detail.rs](crates/intrada-web/src/views/goal_detail.rs) — linked-item rows show "target X / current Y" pill; tapping an item opens a per-item target editor (BottomSheet); auto-suggest banner above actions when criterion met.
- **UI — new component**: `GoalItemTargetsSheet` (private to `goal_detail.rs`, mirrors `LinkItemsSheet` shape from #732).

### Phase 1 — what stays the same

- `#732`'s `LinkItemsSheet` (link/unlink picker). Adding an item still creates a target-less link; targets are set in the new per-item editor.
- `#726`'s edit form. We add fields to it; we don't change its shape.
- The `goal_snapshot` flicker-prevention pattern. Untouched.

## Open questions — for later, not for Phase 1

1. **Consolidation rule for "looks ready."** The latest-session rule is intentionally simplistic. Revisit once we have user behaviour to learn from. Candidate refinements: last-N-sessions, N-sessions-over-M-days, weighted-by-recency. (User flagged wanting to think more on this.)
2. **Sets and Goals overlap.** Sets stay separate for now. Worth revisiting once Phase 3 ("Practice this goal") exists — at that point a Set and a Goal both become "things you can launch a practice session from," and the conceptual cost of having both gets clearer.
3. **What happens when the user re-records a piece (e.g. changes BPM)?** Currently the goal's `target_tempo` refers to a bpm number, not a percentage of the piece's "natural" tempo. If the user updates the library item's tempo marking from Allegro to Presto, the goal target doesn't auto-adjust. That's probably correct (the goal target is an absolute commitment) but worth a follow-up if it surprises users.
4. **Cross-goal practice suggester.** The "show me what's due" / interleaving / spaced-practice feature. Depends on Phase 3 existing and on enough goal data flowing through sessions to be useful. Out of scope here; flagged to anchor future spec work.

## Coverage expectations (per PR)

Phase 1 PR description should include:

> **Coverage: full on core + API; UI is shell-only (excluded).** New core handlers `UpdateGoalItem`, the `effective_target_*` projection helpers, and `validate_target_*` get unit tests. New API endpoints get happy-path + auth-rejection tests in `intrada-api/tests/`. Goal-detail and goal-edit-form changes are WASM shell code, excluded from coverage per `codecov.yml`.

## Out of scope (this spec)

- Tempo targets (Phase 2)
- Practice integration (Phase 3)
- Cross-goal suggesters (Phase 4 / future spec)
- Goal templates / cloning ("save this goal shape to reuse next term")
- Notification / reminder when a target's date is approaching
- Tasks (the non-instrument work concept from the original Goals+Tasks design memory) — not in scope; this spec keeps Goals as collections of *library items* with targets, not arbitrary tasks. Revisit only if user research surfaces it as a gap.
