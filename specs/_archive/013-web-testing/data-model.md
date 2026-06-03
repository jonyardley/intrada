# Data Model: Web UI Testing & E2E Test Infrastructure

**Feature**: 013-web-testing | **Date**: 2026-02-15

## Overview

This feature introduces no new domain entities. It adds test infrastructure that exercises existing entities (`Piece`, `Exercise`, `Session`, `LibraryData`, `SessionsData`) through three test layers.

## Test Layers

### Layer 1: Rust Unit Tests (`#[test]`)

**Target**: Pure functions in `helpers.rs` and `validation.rs`

| Function | Input Type | Output Type | Key Test Cases |
|----------|-----------|-------------|----------------|
| `parse_tags` | `&str` | `Vec<String>` | Empty, single, multiple, whitespace, trailing commas |
| `parse_tempo` | `(&str, &str)` | `Option<Tempo>` | Both empty, marking only, BPM only, both present, invalid BPM |
| `parse_tempo_display` | `&Option<String>` | `(String, String)` | None, marking only, BPM only, "Marking (BPM BPM)" format |
| `validate_library_form` | `(ItemType, &FormData)` | `HashMap<String, String>` | Valid piece, valid exercise, missing title, missing composer for piece, oversized fields, invalid BPM, tag too long |

**Existing types used**: `Tempo`, `ItemType`, `FormData`

### Layer 2: WASM Integration Tests (`#[wasm_bindgen_test]`)

**Target**: Browser-dependent code in `core_bridge.rs`

| Test Scenario | Entities Exercised | Browser API |
|---------------|--------------------|-------------|
| Library data round-trip | `LibraryData`, `Piece`, `Exercise` | localStorage |
| Session data round-trip | `SessionsData`, `Session` | localStorage |
| Empty localStorage seed | `LibraryData` | localStorage |
| Load after save | `LibraryData` | localStorage |

**Existing types used**: `LibraryData`, `SessionsData`, `Piece`, `Exercise`, `Session`

### Layer 3: E2E Tests (Playwright)

**Target**: Built application served as static files

| Test Scenario | User Action | Expected Outcome |
|---------------|-------------|------------------|
| Smoke test | Navigate to `/` | Library list renders with items |

**No domain types directly referenced** — E2E tests interact through the rendered UI only.

## State Management for Test Isolation

### Unit Tests
No shared state — pure functions with no side effects.

### WASM Integration Tests
- `thread_local! LIBRARY: RefCell<LibraryData>` — reset by clearing localStorage + calling `load_library_data()`
- `thread_local! SESSIONS: RefCell<SessionsData>` — reset by clearing localStorage + calling `load_sessions_data()`

### E2E Tests
- Each test starts with a fresh page load (Playwright default)
- localStorage is populated by the app's own initialisation logic
