# Research: iOS Session Summary & History

## R1: Session Summary Data Shape

**Decision**: Use `SummaryView` from ViewModel directly — no transformation needed

**Rationale**: `SummaryView` already contains `totalDurationDisplay`, `completionStatus`, `notes`, `entries` (as `[SetlistEntryView]`), and `sessionIntention`. Each entry has `score`, `achievedTempo`, `notes`, `status`, `repTarget`, `repCount`, `repTargetReached`. This maps directly to the summary UI.

## R2: Session History Data Shape

**Decision**: Use `sessions: [PracticeSessionView]` from ViewModel — same shape as SummaryView plus `id`, `startedAt`, `finishedAt`

**Rationale**: The ViewModel already provides the full session list sorted by date. No client-side sorting needed. Date grouping (Today, Yesterday, etc.) is a shell-local concern using `RelativeDateTimeFormatter`.

## R3: Entry Result Row — Shared Component

**Decision**: Single `SessionEntryResultRow` with `isEditable: Bool` parameter, shared between summary and history detail

**Rationale**: The web uses the same entry display in both summary and history. The only difference is whether score/tempo/notes are editable. Sharing a component avoids duplication and ensures visual consistency.

## R4: Score Editing in Summary

**Decision**: Reuse `ScoreSelectorView` from #197 inline within each entry row

**Rationale**: Already built and tested. Matches the transition prompt scoring UX. Dispatches `updateEntryScore` immediately on tap.

## R5: History List — Replace Idle Placeholder

**Decision**: Replace `PracticeIdleView` with `SessionHistoryView` that handles both states: empty (no sessions → CTA) and populated (session list)

**Rationale**: The idle state is the natural home for session history. The web puts it at `/sessions`. Having it on the Practice tab when idle creates a complete loop: history → new session → active → summary → save → back to history.

## R6: Delete Session

**Decision**: Swipe-to-delete on session cards with confirmation dialog, dispatches `deleteSession(id:)`

**Rationale**: Standard iOS pattern. The web uses a delete button with confirmation. Swipe-to-delete is more iOS-native and discoverable.

## R7: CompletionStatus Rendering

**Decision**: Map `CompletionStatus` enum to visual indicators: `.completed` → green checkmark, `.endedEarly` → warm accent "Ended Early" badge

**Rationale**: Matches the web's rendering. The `CompletionStatus` enum has two variants: `Completed` and `EndedEarly`.
