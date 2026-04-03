# Implementation Plan: iOS Session Summary & History

**Branch**: `198-ios-session-summary` | **Date**: 2026-04-03 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/198-ios-session-summary/spec.md`

## Summary

Build the iOS session summary and history views — replacing `SummaryPlaceholderView` and the idle-state placeholder with real implementations. The summary shows post-session review with per-item results and inline editing. The history shows a chronological list of saved sessions with tap-to-detail. **Shell-only** — all events and ViewModel types already exist.

## Technical Context

**Language/Version**: Swift 6.0, iOS 17.0+
**Primary Dependencies**: SwiftUI, UniFFI (CoreFfi), BCS serialization (auto-generated SharedTypes)
**Storage**: N/A (all persistence via Crux core HTTP effects → REST API → Turso)
**Testing**: `just ios-swift-check` (compile), `just ios-smoke-test` (runtime), `just ios-preview-check` (previews)
**Target Platform**: iOS 17.0+ (iPhone + iPad)
**Project Type**: Mobile (iOS shell consuming shared Crux core)
**Performance Goals**: Smooth scrolling with 50+ sessions, summary loads instantly after session end
**Constraints**: No core/API changes. Reuse existing ScoreSelectorView from #197.
**Scale/Scope**: 4 new views, replace 2 placeholders

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ PASS | Single responsibility per component, follows existing patterns |
| II. Testing Standards | ✅ PASS | Compile check, smoke test, preview check. Core logic tested in Rust. |
| III. UX Consistency | ✅ PASS | Reuses ButtonView, CardView, TypeBadge, ScoreSelectorView, EmptyStateView |
| IV. Performance | ✅ PASS | LazyVStack for session history. No new API calls beyond existing ViewModel. |
| V. Architecture Integrity | ✅ PASS | Pure shell implementation. Events dispatched via `core.update()`. |
| VI. Inclusive Design | ✅ PASS | Clear save/discard actions, confirmation on destructive operations, no time pressure on review. |

## Project Structure

### Source Code

```text
ios/Intrada/
├── Views/Practice/
│   ├── PracticeTabRouter.swift          # UPDATE: Replace SummaryPlaceholderView, replace PracticeIdleView
│   ├── SessionSummaryView.swift         # NEW: Post-session review with inline editing
│   ├── SessionHistoryView.swift         # NEW: Chronological session list (replaces idle placeholder)
│   ├── SessionDetailView.swift          # NEW: Full detail for a past session
│   └── SessionEntryResultRow.swift      # NEW: Entry result display (shared by summary + detail)
├── Components/
│   └── (no new components — reuses ScoreSelectorView, ButtonView, TypeBadge, EmptyStateView)
└── Navigation/
    └── (no changes)
```

## Design References

Pencil frames in `design/intrada.pen`:
- **iOS / Session Summary (iPhone)** — Header with stats, entry list with score/tempo/rep badges, session notes, Save/Discard
- **iOS / Session History (iPhone)** — Date-grouped session cards with duration, item count, intention
- **iPad / Session Summary** — Split layout: stats+notes+actions left, entries right

## Architecture

### State Flow

```text
ViewModel.sessionStatus == .summary
  └─ PracticeTabRouter renders SessionSummaryView
       ├─ Reads: core.viewModel.summary (SummaryView)
       ├─ Inline editing: score, tempo, notes dispatched immediately
       └─ Save/Discard actions transition to .idle

ViewModel.sessionStatus == .idle
  └─ PracticeTabRouter renders SessionHistoryView
       ├─ Reads: core.viewModel.sessions ([PracticeSessionView])
       └─ Tap session → NavigationStack push to SessionDetailView
```

### Event Mapping

| User Action | Core Event |
|-------------|------------|
| Tap score (1–5) in summary | `.session(.updateEntryScore(entryId:, score:))` |
| Enter tempo in summary | `.session(.updateEntryTempo(entryId:, tempo:))` |
| Edit entry notes in summary | `.session(.updateEntryNotes(entryId:, notes:))` |
| Edit session notes | `.session(.updateSessionNotes(notes:))` |
| Tap "Save Session" | `.session(.saveSession(now:))` |
| Tap "Discard" + confirm | `.session(.discardSession)` |
| Tap "New Session" | `.session(.startBuilding)` |
| Delete past session + confirm | `.session(.deleteSession(id:))` |

### iPad Adaptation

- **Summary**: `horizontalSizeClass == .regular` → split view (stats/notes/actions left, entries right)
- **History**: `NavigationSplitView` — session list left, detail right (same pattern as Library)

### Implementation Notes

- `SessionEntryResultRow` is shared between summary (editable) and detail (read-only). Uses an `isEditable: Bool` parameter.
- Score editing reuses `ScoreSelectorView` from #197.
- Tempo editing uses the same wheel picker pattern from `TransitionPromptSheet`.
- Session history replaces `PracticeIdleView` — when idle with sessions, show history. When idle with no sessions, show empty state with "New Session" CTA.

## Complexity Tracking

No constitution violations. All gates pass.
