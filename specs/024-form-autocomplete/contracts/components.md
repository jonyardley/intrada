# Component Contracts: Form Autocomplete

**Feature**: 024-form-autocomplete
**Date**: 2026-02-18

## Overview

No new API endpoints are introduced. This feature is entirely client-side. The contracts below define the component interfaces.

## Component: Autocomplete

A reusable dropdown-with-suggestions component.

### Props

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| id | &'static str | Yes | HTML id for the input element |
| suggestions | Signal\<Vec\<String\>\> | Yes | Reactive list of all possible suggestions (pre-deduped) |
| value | RwSignal\<String\> | Yes | Current input text (two-way bound) |
| on_select | Callback\<String\> | Yes | Called when user selects a suggestion |
| placeholder | Option\<&'static str\> | No | Placeholder text for the input |
| min_chars | usize | No | Minimum chars before showing suggestions (default: 2) |
| max_suggestions | usize | No | Maximum suggestions to display (default: 8) |
| exclude | Signal\<Vec\<String\>\> | No | Values to exclude from suggestions (for tag dedup) |

### Behaviour

- Filters `suggestions` by `value` text (case-insensitive, prefix-first ranking)
- Shows dropdown when filtered list is non-empty and input length ≥ `min_chars`
- Keyboard: ArrowDown/ArrowUp navigate, Enter/Tab select, Escape dismiss
- Click on suggestion fires `on_select`
- Dropdown closes on: selection, Escape, focus leaving the component

### ARIA

- Input: `role="combobox"`, `aria-autocomplete="list"`, `aria-expanded`, `aria-activedescendant`
- Dropdown: `role="listbox"`
- Each suggestion: `role="option"`, `aria-selected` for highlighted item

---

## Component: TagInput

A chip-based multi-tag input with integrated autocomplete.

### Props

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| id | &'static str | Yes | HTML id for the input element |
| tags | RwSignal\<Vec\<String\>\> | Yes | Currently selected tags (two-way) |
| available_tags | Signal\<Vec\<String\>\> | Yes | All unique tags from library |
| field_name | &'static str | Yes | Field name for error display |
| errors | RwSignal\<HashMap\<String, String\>\> | Yes | Form errors map |

### Behaviour

- Renders each tag in `tags` as a chip with a remove (×) button
- Inline text input after chips for typing new tags
- Typing triggers Autocomplete suggestions (excluding already-selected tags)
- Selecting a suggestion or pressing comma/Enter adds tag to `tags` and clears input
- Removing a chip removes the tag from `tags`
- Pasting comma-separated text parses and adds all tags directly

### Visual Structure

```
┌─────────────────────────────────────────────┐
│ [classical ×] [baroque ×] [type here...   ] │
├─────────────────────────────────────────────┤
│  scales                                      │
│  sight-reading                               │
└─────────────────────────────────────────────┘
```

---

## No API Changes

This feature reads from the existing ViewModel (populated by `GET /api/pieces` and `GET /api/exercises`). No new endpoints, request/response schemas, or database changes are needed.
