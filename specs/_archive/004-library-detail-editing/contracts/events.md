# Event Contracts: Library Add, Detail View & Editing

**Feature**: `004-library-detail-editing`
**Date**: 2026-02-14

## Overview

This feature has no REST API. All interactions flow through Crux core events. The web shell dispatches events via `core.process_event(event)` and reads state via `core.view()`. This document maps user actions to existing Crux events.

## Event Contracts

### Add Piece

**Trigger**: User fills add piece form and clicks "Save"
**Shell action**: Parse form fields, construct `CreatePiece`, dispatch event
**Event**: `Event::Piece(PieceEvent::Add(CreatePiece { title, composer, key, tempo, notes, tags }))`
**Success**: Piece added to `Model.pieces`, `ViewModel.items` updated, view navigates to List
**Failure**: `Model.last_error` set with validation message, `ViewModel.error` populated

**CreatePiece fields**:
| Field | Source | Conversion |
|-------|--------|------------|
| title | `title` signal | Direct string |
| composer | `composer` signal | Direct string |
| key | `key` signal | `None` if empty, `Some(value)` otherwise |
| tempo | `tempo_marking` + `bpm` signals | `None` if both empty; `Some(Tempo { marking, bpm })` otherwise |
| notes | `notes` signal | `None` if empty, `Some(value)` otherwise |
| tags | `tags` signal | Split by comma, trim, filter empty, collect to `Vec<String>` |

---

### Add Exercise

**Trigger**: User fills add exercise form and clicks "Save"
**Event**: `Event::Exercise(ExerciseEvent::Add(CreateExercise { title, composer, category, key, tempo, notes, tags }))`
**Success**: Exercise added to `Model.exercises`, `ViewModel.items` updated, view navigates to List
**Failure**: `Model.last_error` set

**CreateExercise fields** — same as CreatePiece plus:
| Field | Source | Conversion |
|-------|--------|------------|
| composer | `composer` signal | `None` if empty (optional for exercises) |
| category | `category` signal | `None` if empty, `Some(value)` otherwise |

---

### Edit Piece

**Trigger**: User modifies fields in edit piece form and clicks "Save"
**Event**: `Event::Piece(PieceEvent::Update { id, input: UpdatePiece { title, composer, key, tempo, notes, tags } })`
**Success**: Piece updated in `Model.pieces`, view navigates to Detail(id)
**Failure**: `Model.last_error` set (e.g., NotFound if item was deleted)

**UpdatePiece fields** — all optional (only changed fields sent):
| Field | Type | Conversion |
|-------|------|------------|
| title | `Option<String>` | `Some(value)` — always sent (required field) |
| composer | `Option<String>` | `Some(value)` — always sent (required field) |
| key | `Option<Option<String>>` | `Some(None)` to clear, `Some(Some(value))` to set |
| tempo | `Option<Option<Tempo>>` | `Some(None)` to clear, `Some(Some(tempo))` to set |
| notes | `Option<Option<String>>` | `Some(None)` to clear, `Some(Some(value))` to set |
| tags | `Option<Vec<String>>` | `Some(vec)` — always sent with current tag list |

---

### Edit Exercise

**Trigger**: User modifies fields in edit exercise form and clicks "Save"
**Event**: `Event::Exercise(ExerciseEvent::Update { id, input: UpdateExercise { ... } })`
**Success/Failure**: Same pattern as Edit Piece

**UpdateExercise fields** — same as UpdatePiece plus:
| Field | Type | Conversion |
|-------|------|------------|
| composer | `Option<Option<String>>` | `Some(None)` to clear, `Some(Some(value))` to set |
| category | `Option<Option<String>>` | `Some(None)` to clear, `Some(Some(value))` to set |

---

### Delete Item

**Trigger**: User clicks "Delete" on detail view, then confirms
**Event (piece)**: `Event::Piece(PieceEvent::Delete { id })`
**Event (exercise)**: `Event::Exercise(ExerciseEvent::Delete { id })`
**Success**: Item removed from model, view navigates to List
**Failure**: `Model.last_error` set with NotFound message

---

## Effect Handling (Unchanged)

The web shell's `process_effects()` function already handles all storage effects as no-ops (except LoadAll which returns stub data). No changes needed:

| StorageEffect | Shell Behavior |
|---------------|----------------|
| SavePiece | No-op |
| SaveExercise | No-op |
| UpdatePiece | No-op |
| UpdateExercise | No-op |
| DeleteItem | No-op |
| LoadAll | Returns stub data via DataLoaded event |

## Validation Rules (Shell-Side, Pre-Dispatch)

The shell validates before dispatching to provide inline field-level errors:

| Field | Rule | Error Message |
|-------|------|---------------|
| title | Non-empty, max 500 chars | "Title is required" / "Title must be between 1 and 500 characters" |
| composer (piece) | Non-empty, max 200 chars | "Composer is required" / "Composer must be between 1 and 200 characters" |
| composer (exercise) | If present, max 200 chars | "Composer must be between 1 and 200 characters" |
| category | If present, max 100 chars | "Category must be between 1 and 100 characters" |
| notes | If present, max 5000 chars | "Notes must not exceed 5000 characters" |
| bpm | If present, 1-400 | "BPM must be between 1 and 400" |
| tempo marking | If present, max 100 chars | "Tempo marking must not exceed 100 characters" |
| tags | Each tag 1-100 chars | "Each tag must be between 1 and 100 characters" |
| tempo | If tempo has marking or bpm, at least one must be set | "Tempo must have at least a marking or BPM value" |
