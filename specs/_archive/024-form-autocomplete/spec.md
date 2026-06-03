# Feature Specification: Form Autocomplete

**Feature Branch**: `024-form-autocomplete`
**Created**: 2026-02-18
**Status**: Draft
**Input**: User description: "Autocomplete for tags and composer fields in library forms. Two related improvements: (1) Inline tag management — when adding/editing tags on a piece or exercise, autocomplete from existing tags in the library and allow creating new ones inline. (2) Composer autocomplete — when entering a composer name on a piece, suggest from existing composer names already in the library. Both share the same autocomplete UX pattern. References: GitHub issues #84 and #85."

## Clarifications

### Session 2026-02-18

- Q: Minimum characters before suggestions appear? → A: 2 characters — balances responsiveness with relevance for short tags and composer names.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Tag Autocomplete on Library Forms (Priority: P1)

A musician adding or editing a library item begins typing in the tags field. As they type, a dropdown appears showing existing tags from their library that match. They can select a tag from the dropdown to add it, or continue typing and press comma or Enter to create a new tag. Tags already applied to the current item are not shown in suggestions.

**Why this priority**: Tags are the primary organisational tool in the library and are used on both pieces and exercises. Autocomplete prevents typos and inconsistent naming (e.g. "classical" vs "Classical" vs "clasical"), which directly improves filtering and discovery.

**Independent Test**: Can be fully tested by opening the add or edit form, typing in the tags field, and verifying that matching suggestions appear from existing library tags. Delivers immediate value by reducing input errors and speeding up tagging.

**Acceptance Scenarios**:

1. **Given** a library with items tagged "scales", "sight-reading", and "baroque", **When** the user types "sc" in the tags field, **Then** a dropdown appears showing "scales" as a suggestion.
2. **Given** a library with existing tags, **When** the user selects a suggestion from the dropdown, **Then** the tag is added to the item and the input clears ready for the next tag.
3. **Given** the user types a tag name that does not exist in the library, **When** they press comma or Enter, **Then** the new tag is created and added to the item.
4. **Given** the item already has the tag "scales", **When** the user types "sc", **Then** "scales" does not appear in the suggestions (no duplicates).
5. **Given** suggestions are displayed, **When** the user clicks outside the dropdown or presses Escape, **Then** the dropdown closes.

---

### User Story 2 - Composer Autocomplete on Library Forms (Priority: P1)

A musician adding or editing a piece begins typing a composer name. A dropdown appears showing existing composer names from their library that match. They can select a composer to populate the field instantly, or continue typing to enter a new name. This works for both the required composer field on pieces and the optional composer field on exercises.

**Why this priority**: Composer names are prone to inconsistency (e.g. "J.S. Bach" vs "Bach, J.S." vs "Johann Sebastian Bach"). Autocomplete from existing entries encourages consistency across the library.

**Independent Test**: Can be fully tested by opening the add form for a piece, typing a partial composer name, and verifying that matching suggestions appear from existing library entries. Delivers immediate value by reducing typos and encouraging naming consistency.

**Acceptance Scenarios**:

1. **Given** a library with pieces by "J.S. Bach" and "Beethoven", **When** the user types "ba" in the composer field, **Then** a dropdown appears showing both "J.S. Bach" and "Beethoven" (partial match on "ba").
2. **Given** suggestions are displayed, **When** the user selects "J.S. Bach", **Then** the composer field is populated with "J.S. Bach" and the dropdown closes.
3. **Given** the user types a composer name not in the library, **When** they move to the next field, **Then** the new composer name is accepted as-is (no restriction to existing names only).
4. **Given** the user is on the exercise form, **When** they type in the optional composer field, **Then** suggestions appear from both pieces and exercises in the library.

---

### User Story 3 - Keyboard Navigation of Suggestions (Priority: P2)

A musician navigating the autocomplete dropdown can use keyboard controls for efficient input without reaching for the mouse. Arrow keys move through suggestions, Enter/Tab selects the highlighted suggestion, and Escape dismisses the dropdown.

**Why this priority**: Keyboard navigation is important for accessibility and power-user efficiency, but the feature delivers core value through mouse interaction alone.

**Independent Test**: Can be tested by typing in a field with suggestions, using arrow keys to navigate, and pressing Enter to select. Delivers accessibility compliance and efficient workflow.

**Acceptance Scenarios**:

1. **Given** the autocomplete dropdown is open with suggestions, **When** the user presses the down arrow, **Then** the next suggestion is highlighted.
2. **Given** a suggestion is highlighted, **When** the user presses Enter or Tab, **Then** the highlighted suggestion is selected and applied.
3. **Given** the dropdown is open, **When** the user presses Escape, **Then** the dropdown closes and focus remains on the input field.
4. **Given** the last suggestion is highlighted, **When** the user presses the down arrow, **Then** highlighting wraps to the first suggestion.

---

### Edge Cases

- What happens when the library is empty (no existing tags or composers)? The dropdown simply does not appear; the field behaves as a plain text input.
- What happens when the user pastes a comma-separated list of tags? Each tag is parsed individually; autocomplete is not triggered for pasted content — tags are added directly.
- How does matching work — case-sensitive or case-insensitive? Matching is case-insensitive (e.g. typing "Bach" matches "bach" and "BACH"), but the original casing of the stored value is preserved in the suggestion.
- What happens with very long suggestion lists? The dropdown shows a maximum of 8 suggestions, sorted by relevance (prefix matches first, then substring matches).
- What if a tag contains a comma? Commas are the tag delimiter; a tag cannot contain a comma. This is existing behaviour.
- What happens on slow connections when fetching suggestion data? Suggestions are derived from library data already loaded in the application; no additional network request is needed.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display a dropdown of matching suggestions when the user types at least 2 characters in the tags field on add and edit forms.
- **FR-002**: System MUST display a dropdown of matching suggestions when the user types at least 2 characters in the composer field on add and edit forms.
- **FR-003**: Suggestions MUST be filtered using case-insensitive matching against the user's input text.
- **FR-004**: Suggestions MUST prioritise prefix matches over substring matches (e.g. typing "sc" shows "scales" before "miscellaneous").
- **FR-005**: The dropdown MUST show a maximum of 8 suggestions at a time.
- **FR-006**: For tags, selecting a suggestion MUST add the tag to the item and clear the input for the next tag.
- **FR-007**: For tags, the user MUST be able to create new tags by typing a name and pressing comma or Enter.
- **FR-008**: For tags, suggestions MUST exclude tags already applied to the current item.
- **FR-009**: For composer, selecting a suggestion MUST populate the field with the full composer name.
- **FR-010**: For composer, the user MUST be able to enter names not in the suggestions (free-text input).
- **FR-011**: Suggestions MUST be sourced from existing library data (all pieces and exercises) without additional network requests.
- **FR-012**: The dropdown MUST close when the user clicks outside it, presses Escape, or selects a suggestion.
- **FR-013**: The dropdown MUST support keyboard navigation: arrow keys to move, Enter/Tab to select, Escape to dismiss.
- **FR-014**: Each tag MUST be displayed as a removable chip/badge in the form, replacing the current comma-separated text input.
- **FR-015**: Users MUST be able to remove individual tags by clicking a remove button on the chip.

### Key Entities

- **Tag**: A short text label (1–100 characters) applied to pieces and exercises for organisation. Unique per item, case-insensitive for matching, original case preserved.
- **Composer**: A text field (1–200 characters) identifying who wrote or arranged a piece or exercise. Required for pieces, optional for exercises.
- **Suggestion List**: A derived, transient list of unique values extracted from existing library items, filtered by user input. Not persisted separately.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can add an existing tag to an item in under 3 seconds (type 2+ characters, select from dropdown).
- **SC-002**: Tag consistency improves — zero duplicate tags caused by casing or typos when the correct tag already exists in the library.
- **SC-003**: Composer name consistency improves — users entering a composer already in the library select the existing name at least 80% of the time.
- **SC-004**: All autocomplete interactions are fully keyboard-accessible (no mouse required).
- **SC-005**: Suggestion dropdown appears within 100ms of the user typing, with no perceptible delay.

## Assumptions

- Suggestion data is derived from library items already fetched by the application (the library list is loaded on app start). No new API endpoints are needed.
- The tag input changes from a plain comma-separated text field to a chip-based input with inline autocomplete. This is a UX improvement that replaces the existing pattern.
- Composer suggestions are merged from both pieces and exercises, deduplicated case-insensitively, preserving the first-seen casing.
- The autocomplete component is reusable and can be applied to both tag and composer fields with minimal configuration differences (multi-select for tags, single-select for composer).

## References

- GitHub Issue #84 — Inline tag management
- GitHub Issue #85 — Composer autocomplete
