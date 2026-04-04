# Research: iOS Routines

## R1: Routine List Pattern

**Decision**: Follow SessionHistoryView pattern — NavigationStack on iPhone, NavigationSplitView on iPad, date section headers replaced by simple list.

**Rationale**: Routines are simpler than sessions (no date grouping needed). The List + swipe-to-delete + NavigationLink pattern is well-established.

## R2: Edit View Pattern

**Decision**: Separate RoutineEditView pushed via NavigationStack, with name field + reorderable List + "Add from Library" section.

**Rationale**: Matches the web's `/routines/:id/edit` pattern. SwiftUI List with `.onMove` provides native drag-to-reorder. Library items shown inline below the routine entries.

## R3: Save-as-Routine Component

**Decision**: Reusable `RoutineSaveForm` component — collapsed state shows "Save as Routine" button, expanded shows name TextField + Save/Cancel. Used in both SetlistSheetContent and SessionSummaryView.

**Rationale**: Matches web's `RoutineSaveForm` pattern exactly. Shared component avoids duplication.

## R4: Load Routine Flow

**Decision**: In SetlistSheetContent, replace placeholder "Load Routine" button with a sheet/menu showing available routines. Tap to load dispatches `.routine(.loadRoutineIntoSetlist(routineId:))`.

**Rationale**: Simple and discoverable. Routine items are appended to the existing setlist (additive, per spec).

## R5: Routine Entry Manipulation for Edit

**Decision**: Use `RoutineEntry` (not `RoutineEntryView`) for the `updateRoutine` event. Build the entries array from the current edited state in the view.

**Rationale**: The `updateRoutine` event expects `[RoutineEntry]` not `[RoutineEntryView]`. The view tracks edits locally and dispatches the full updated entry list on Save.
