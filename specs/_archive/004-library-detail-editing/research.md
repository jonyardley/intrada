# Research: Library Add, Detail View & Editing

**Feature**: `004-library-detail-editing`
**Date**: 2026-02-14

## Overview

No critical unknowns or NEEDS CLARIFICATION items exist for this feature. All technology choices are established from previous features (001-music-library, 003-leptos-app-mvp). The Crux core already implements all required domain events. This research documents the key patterns to follow.

## Research Items

### 1. Leptos Form Handling Pattern

**Decision**: Use individual `RwSignal<String>` per form field with `on:input` event bindings.

**Rationale**: Leptos CSR uses fine-grained reactivity. Each form field bound to its own signal allows independent updates without re-rendering the entire form. The `prop:value=signal.get()` + `on:input` pattern is the standard Leptos approach for controlled inputs.

**Alternatives considered**:
- Uncontrolled inputs (read DOM on submit) — rejected because validation errors need to preserve field values, and the form state needs to be accessible for pre-population in edit mode.
- Single struct signal for all fields — rejected because updating one field would trigger re-renders of all fields.

### 2. View Navigation Without Routing

**Decision**: Use a shell-side `RwSignal<ViewState>` enum to track which view is active. The `App` component pattern-matches on this signal to render the correct view component.

**Rationale**: The spec explicitly excludes URL-based routing. A simple enum signal is the lightest-weight approach for state-based navigation in Leptos CSR. The enum variants carry item IDs where needed (`Detail(id)`, `EditPiece(id)`, etc.).

**Alternatives considered**:
- leptos_router — rejected because spec excludes URL routing, and adding a router dependency for 4 views is overkill.
- Core-side navigation state — rejected because navigation is a shell concern per Crux architecture.

### 3. Client-Side Validation Before Core Dispatch

**Decision**: Validate form input in the web shell before dispatching Crux events. Display inline errors for fields that fail validation.

**Rationale**: The Crux core already validates on event processing and sets `model.last_error`. However, core validation returns a single error string, not per-field errors. For inline field-level errors (FR-005), the shell should validate first using the same rules and show errors next to the relevant field. If a field passes shell validation but fails core validation (shouldn't happen if rules match), the core error is displayed in the error banner.

**Alternatives considered**:
- Core-only validation with error parsing — rejected because the core's `LibraryError::Validation { field, message }` is serialized as a string in the ViewModel, losing structured field information.
- Adding structured validation errors to ViewModel — considered but deferred; would require core changes. Shell-side validation is sufficient for this feature.

### 4. Crux Event Inventory (Existing)

All required events already exist in `intrada-core`. No core changes needed for event handling:

| Action | Event | Handler |
|--------|-------|---------|
| Add piece | `Event::Piece(PieceEvent::Add(CreatePiece))` | `handle_piece_event` |
| Add exercise | `Event::Exercise(ExerciseEvent::Add(CreateExercise))` | `handle_exercise_event` |
| Edit piece | `Event::Piece(PieceEvent::Update { id, input: UpdatePiece })` | `handle_piece_event` |
| Edit exercise | `Event::Exercise(ExerciseEvent::Update { id, input: UpdateExercise })` | `handle_exercise_event` |
| Delete piece | `Event::Piece(PieceEvent::Delete { id })` | `handle_piece_event` |
| Delete exercise | `Event::Exercise(ExerciseEvent::Delete { id })` | `handle_exercise_event` |

### 5. Finding Items by ID for Detail/Edit Views

**Decision**: Look up items from the `ViewModel.items` list using the item's `id` field.

**Rationale**: The ViewModel already contains all `LibraryItemView` items with their IDs. When navigating to a detail or edit view, the shell can find the item by scanning `view_model.get().items.iter().find(|i| i.id == target_id)`. This avoids needing a separate "get by ID" event in the core.

**Alternatives considered**:
- Adding a `GetById` event to the core — rejected as unnecessary; the ViewModel already has all items.
- Storing a "selected item" in the core Model — rejected because selection is a shell concern.
