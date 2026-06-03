# Data Model: Form Autocomplete

**Feature**: 024-form-autocomplete
**Date**: 2026-02-18

## Overview

This feature introduces no new persisted entities. All data is derived from existing library items already in the ViewModel. The "data model" here describes the transient structures used at the component level.

## Existing Entities (unchanged)

### LibraryItemView (read-only, from ViewModel)

| Field | Type | Notes |
|-------|------|-------|
| id | String | ULID |
| item_type | String | "piece" or "exercise" |
| title | String | Item name |
| subtitle | String | Composer (pieces) or category/composer (exercises) |
| category | Option\<String\> | Exercise category, if set |
| tags | Vec\<String\> | All tags on this item |

## Derived Data (transient, component-level)

### Unique Tags List

Derived from `view_model.items` by:
1. Collecting all `item.tags` across every item
2. Deduplicating case-insensitively (preserving first-seen casing)
3. Sorting alphabetically

**Lifecycle**: Recomputed reactively whenever `view_model` changes. Not persisted.

### Unique Composers List

Derived from `view_model.items` by:
1. For pieces (`item_type == "piece"`): extract `subtitle` (which is the composer)
2. For exercises (`item_type == "exercise"`): extract `subtitle` only when `category` is `None` (otherwise subtitle is the category, not composer)
3. Deduplicating case-insensitively (preserving first-seen casing)
4. Filtering out empty strings
5. Sorting alphabetically

**Lifecycle**: Recomputed reactively whenever `view_model` changes. Not persisted.

### Filtered Suggestions

Derived from the appropriate unique list (tags or composers) by:
1. Requiring minimum 2 characters of input
2. Case-insensitive matching against input text
3. Prefix matches ranked before substring matches
4. For tags: excluding values already in the current item's tag list
5. Limiting to 8 results

**Lifecycle**: Recomputed on every keystroke (after 2-char threshold). Not persisted.

## State Transitions

### Tag Input Component State

```
Empty → Typing (user types characters)
Typing → Suggestions Open (≥2 chars, matches found)
Typing → Empty (user clears input)
Suggestions Open → Tag Added (user selects suggestion or presses comma/Enter)
Suggestions Open → Typing (<2 chars or no matches)
Suggestions Open → Dismissed (Escape, click outside, blur)
Tag Added → Empty (input clears, chip appears)
```

### Composer Input Component State

```
Empty → Typing (user types characters)
Typing → Suggestions Open (≥2 chars, matches found)
Typing → Empty (user clears input)
Suggestions Open → Value Set (user selects suggestion)
Suggestions Open → Typing (<2 chars or no matches)
Suggestions Open → Dismissed (Escape, click outside, blur)
Value Set → Typing (user modifies the field)
```

## Validation Rules (unchanged)

- Tags: each 1–100 characters, no commas allowed (existing)
- Composer: 1–200 characters; required for pieces, optional for exercises (existing)
- No new validation rules introduced by this feature
