# Research: Session Item Scoring

**Feature**: 022-session-scoring
**Date**: 2026-02-17

## Overview

This feature adds an optional confidence score (1–5) to each completed setlist entry. The research below resolves all technical decisions needed before design and implementation.

## Decision 1: Score Data Type

**Decision**: `Option<u8>` in Rust; `INTEGER` (nullable) in SQLite/Turso

**Rationale**: A score is a small integer (1–5). `u8` is the smallest standard Rust integer type and maps cleanly to SQLite `INTEGER`. Using `Option` makes the field nullable, preserving backward compatibility with existing entries (which will have `NULL` scores). No need for a float, enum, or separate table — the value is a simple number attached to an existing record.

**Alternatives considered**:
- `Option<u16>` / `Option<i32>`: Unnecessarily wide for a 1–5 range.
- Custom `Score` newtype: Adds indirection with no practical benefit. Validation at boundaries (event handler + API route) is sufficient.
- Separate `scores` table: Over-engineered for a single optional field on an existing entity.

## Decision 2: Score Validation Strategy

**Decision**: Define `MIN_SCORE: u8 = 1` and `MAX_SCORE: u8 = 5` in `intrada-core/src/validation.rs`. Validate in two places: (1) the core `UpdateEntryScore` event handler rejects out-of-range values, (2) the API `save_session` route rejects out-of-range values before database insert.

**Rationale**: Consistent with existing validation pattern — constants are defined once in core and imported by both core event handlers and API routes. The core validates during the session summary (client-side), the API validates on save (server-side). Belt-and-braces approach prevents invalid data regardless of entry point.

**Alternatives considered**:
- Validate only in core: Server would accept any value if called directly (bypassing the web shell).
- Validate only in API: Invalid scores could appear in the local summary view before save.
- Newtype with `TryFrom`: Cleaner in isolation, but inconsistent with how existing validation (notes length, BPM range) is handled in this codebase.

## Decision 3: Database Migration Approach

**Decision**: Add a new migration in `migrations.rs`: `ALTER TABLE setlist_entries ADD COLUMN score INTEGER;`. The column is nullable with no default — existing rows remain `NULL`.

**Rationale**: SQLite supports `ALTER TABLE ... ADD COLUMN` with nullable columns. This is the simplest possible migration. Existing sessions remain untouched (NULL score = "not scored"), and the application code already uses `Option<u8>` which maps directly to nullable INTEGER. No data backfill needed.

**Alternatives considered**:
- Default value (e.g., `DEFAULT 0`): Would make existing entries appear scored at 0, which is semantically wrong and outside the 1–5 range.
- New table: Unnecessary complexity for a single column addition.

## Decision 4: Progress Computation Location

**Decision**: Compute item score history in the core `view()` function alongside the existing `compute_practice_summary`. Add a new `compute_score_history` function that filters scored entries for a given `item_id` and returns them chronologically.

**Rationale**: The core already computes `ItemPracticeSummary` per library item during `view()` by iterating all sessions. Extending this to also collect score history follows the same pattern — pure computation in core, no I/O. The data is already loaded into the `Model` (all sessions are fetched on app start).

**Alternatives considered**:
- New API endpoint (`GET /items/{id}/progress`): Would require a new server-side query. Not needed — all session data is already available client-side in the Model. At this scale (hundreds of sessions), client-side filtering is instant.
- Compute in the shell (web): Violates the pure-core principle. The shell should only render what core provides.

## Decision 5: Score UI Control

**Decision**: Use a row of 5 tappable number buttons (1–5) per completed entry on the summary screen. Selected button is visually highlighted. Tapping the same button again deselects it (returns to no score).

**Rationale**: Five buttons are the most direct, accessible, and mobile-friendly control for a 1–5 discrete scale. Each number is a single tap — no sliders, no dropdowns, no modals. The toggle-to-deselect behaviour supports the "scoring is optional" requirement. This is consistent with the app's existing interaction style (inline controls on entry rows, similar to the notes input).

**Alternatives considered**:
- Star rating: Common but adds visual complexity and ambiguity ("is 3 stars good or bad?"). Numbers are explicit.
- Slider: Poor for discrete values; imprecise on mobile.
- Dropdown/select: Requires two interactions (open + select); slower.

## Decision 6: Progress Display Format

**Decision**: On the item detail page, show: (1) the most recent confidence score displayed prominently as a large number with label, (2) below it, a chronological list of past scores with session date and score value, most recent first.

**Rationale**: Per clarification, users want to see "where am I now?" at a glance (latest score), plus the ability to scan history. A simple list is the minimum viable presentation that enables this. Charts or trend indicators can be added in a future feature.

**Alternatives considered**:
- Chart/sparkline: Valuable but out of scope per spec assumptions.
- Trend arrow: Requires defining "trend" algorithm — deferred to future feature.
- Average score: Loses temporal information; masks recent improvement or decline.
