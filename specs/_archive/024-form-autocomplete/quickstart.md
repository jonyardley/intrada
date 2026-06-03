# Quickstart: Form Autocomplete

**Feature**: 024-form-autocomplete
**Date**: 2026-02-18

## Prerequisites

- Rust stable (1.75+)
- trunk 0.21.x
- Tailwind CSS v4 standalone CLI
- A running intrada-api instance with some library data (pieces/exercises with tags and composers)

## Build & Run

```bash
# From repo root
cargo test                           # Run all tests (core + web)
cargo clippy                         # Lint check
cd crates/intrada-web && trunk serve # Start dev server
```

## Verification Steps

### 1. Tag Autocomplete (US1)

1. Ensure library has items with tags (e.g. "scales", "baroque", "classical")
2. Navigate to Add Item form (`/library/new`)
3. In the Tags field, type "sc"
4. **Verify**: Dropdown appears showing "scales" as a suggestion
5. Click "scales" in the dropdown
6. **Verify**: "scales" appears as a chip/badge, input clears
7. Type "new-tag" and press comma
8. **Verify**: "new-tag" appears as a chip alongside "scales"
9. Click the × on the "scales" chip
10. **Verify**: "scales" chip is removed

### 2. Composer Autocomplete (US2)

1. Ensure library has pieces with composers (e.g. "J.S. Bach", "Beethoven")
2. Navigate to Add Item form (`/library/new`), select Piece tab
3. In the Composer field, type "ba"
4. **Verify**: Dropdown appears showing matching composers
5. Select "J.S. Bach" from the dropdown
6. **Verify**: Composer field is populated with "J.S. Bach", dropdown closes
7. Clear the field, type "New Composer"
8. Tab to next field
9. **Verify**: "New Composer" is accepted (no restriction to existing names)

### 3. Keyboard Navigation (US3)

1. In Tags field, type "sc" to trigger suggestions
2. Press ArrowDown
3. **Verify**: First suggestion is highlighted
4. Press ArrowDown again
5. **Verify**: Highlight moves to next suggestion
6. Press Enter
7. **Verify**: Highlighted suggestion is selected and added as chip
8. Type "ba" in Composer field to trigger suggestions
9. Press Escape
10. **Verify**: Dropdown closes, focus remains on composer input

### 4. Edit Form

1. Navigate to an existing item's edit form (`/library/:id/edit`)
2. **Verify**: Existing tags are displayed as chips
3. **Verify**: Autocomplete works identically to add form for both tags and composer

### 5. Edge Cases

1. Empty library (no items): Tags and composer fields work as plain inputs, no dropdown appears
2. Type only 1 character: No dropdown appears (minimum 2 chars)
3. All suggestions already selected as tags: Dropdown doesn't appear (nothing to suggest)

## Success Criteria Check

- [ ] SC-001: Adding an existing tag takes under 3 seconds (type + select)
- [ ] SC-002: Selecting a suggested tag uses the exact existing casing
- [ ] SC-003: Composer suggestions appear from existing library data
- [ ] SC-004: All interactions work via keyboard only (no mouse needed)
- [ ] SC-005: Dropdown appears within 100ms of typing (no perceptible delay)
