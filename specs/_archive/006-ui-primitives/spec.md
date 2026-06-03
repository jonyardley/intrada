# Feature Specification: UI Primitive Components

**Feature Branch**: `006-ui-primitives`
**Created**: 2026-02-14
**Status**: Draft
**Input**: User description: "I'd like further componentisation for UI elements such as typography, header, footer, label, card etc..."

## Context

The Intrada web application was recently refactored (feature 005) from a monolithic file into a multi-file component architecture with `components/`, `views/`, and root modules. While the structure is now well-organised, the individual view files still contain a significant amount of duplicated UI markup. The same styling patterns for buttons, form fields, headings, badges, cards, and layout elements are repeated 30+ times across views.

This feature extracts those repeated visual patterns into a library of reusable UI primitive components. The result is a consistent, maintainable visual language where changes to a shared element (e.g., the primary button style) only need to happen in one place.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Replace Inline Button Styles with Button Components (Priority: P1)

A developer working on Intrada views currently has to copy-paste long CSS class strings whenever they add a button. This is error-prone and makes visual consistency hard to maintain. By introducing a shared Button component with variants (primary, secondary, danger), every button in the app can be rendered with a single component call and consistent styling.

**Why this priority**: Buttons are the most interaction-critical UI element and appear in every view (6+ primary, 7+ secondary, 1 danger). Extracting them first delivers the highest reduction in duplicated markup.

**Independent Test**: Can be fully tested by verifying that all buttons across the application render with identical styling for each variant and that the application compiles and passes existing tests.

**Acceptance Scenarios**:

1. **Given** a view that needs a primary action button, **When** the developer uses the shared Button component with the primary variant, **Then** the button renders with the standard primary styling (indigo background, white text, rounded corners, hover state)
2. **Given** a view that needs a secondary action button, **When** the developer uses the shared Button component with the secondary variant, **Then** the button renders with the standard secondary styling (white background, slate border, hover state)
3. **Given** a view that needs a destructive action button, **When** the developer uses the shared Button component with the danger variant, **Then** the button renders with the standard danger styling (red background or red-outlined, hover state)
4. **Given** the Button component is used in all views, **When** comparing the rendered output to the original inline-styled buttons, **Then** the visual appearance is identical pixel-for-pixel

---

### User Story 2 - Replace Inline Form Field Markup with Form Field Components (Priority: P1)

The four form views (add piece, add exercise, edit piece, edit exercise) each repeat identical label-input-error markup 7-8 times. A shared form field component that wraps label, input, and error display would eliminate approximately 30 duplicated label patterns and 30 duplicated input patterns.

**Why this priority**: Form fields represent the single largest source of duplicated markup (60+ repetitions across label and input patterns). Extracting them also improves consistency of form UX — every field gets the same spacing, focus styling, and error display behaviour.

**Independent Test**: Can be fully tested by verifying that all form fields across add/edit views render with identical styling and that form validation still works correctly with the extracted components.

**Acceptance Scenarios**:

1. **Given** a form that needs a text input field, **When** the developer uses the shared TextField component with a label, **Then** the field renders with the standard label (block, small, medium-weight, slate text), input (full-width, rounded border, focus ring), and optional error message beneath
2. **Given** a form that needs a textarea field, **When** the developer uses the shared TextArea component, **Then** the field renders with the same label and error styling as text inputs
3. **Given** a field has a validation error, **When** the form is submitted, **Then** the error message appears below the field using the existing FormFieldError component
4. **Given** fields are replaced with shared components, **When** the user submits a form with invalid data, **Then** validation behaviour is identical to the current implementation

---

### User Story 3 - Extract Layout Shell Components (Priority: P2)

The app header and footer in `app.rs` are currently inline markup. A developer who wants to adjust the header or footer must edit the root App component. Extracting these into dedicated AppHeader and AppFooter components improves separation of concerns and makes the App component focused solely on routing.

**Why this priority**: The header and footer are shared across all views and define the application's visual frame. Extracting them reduces the App component's responsibilities and makes the layout easier to modify independently.

**Independent Test**: Can be fully tested by verifying the header displays the application name and version badge, the footer displays the informational text, and the overall page layout is unchanged.

**Acceptance Scenarios**:

1. **Given** the App component, **When** it renders the page, **Then** the header component displays the application name, tagline, and version badge with the same styling as the current inline markup
2. **Given** the App component, **When** it renders the page, **Then** the footer component displays the informational text with the same styling as the current inline markup
3. **Given** the extracted components, **When** comparing the rendered page to the original, **Then** the visual appearance is identical

---

### User Story 4 - Extract Card, Badge, and Typography Components (Priority: P2)

Several visual patterns appear in multiple places: the white card container (6 occurrences), type badges for "piece" vs "exercise" (4 occurrences), page headings (5 occurrences), back-navigation links (5 occurrences), and definition-term labels in the detail view (5 occurrences). Extracting these into shared components creates a consistent visual vocabulary.

**Why this priority**: These patterns are less numerous than buttons and form fields but still represent meaningful duplication. Extracting them completes the primitive component library and ensures every visual element has a single source of truth.

**Independent Test**: Can be fully tested by verifying that all card containers, badges, headings, and navigation links render with identical styling to the current inline versions and that the application compiles and passes existing tests.

**Acceptance Scenarios**:

1. **Given** a view that wraps content in a white card with shadow and border, **When** the developer uses the shared Card component, **Then** it renders with the standard card styling (white background, rounded-xl, shadow, border)
2. **Given** an item that has a type (piece or exercise), **When** the developer uses the shared TypeBadge component, **Then** it renders with the correct colour (violet for piece, emerald for exercise) and consistent sizing
3. **Given** a view that needs a page heading, **When** the developer uses the shared PageHeading component, **Then** it renders with the standard heading styling (large, bold, slate-900)
4. **Given** a view that needs a back-navigation link, **When** the developer uses the shared BackLink component, **Then** it renders with the arrow icon, slate text, and hover transition
5. **Given** the detail view shows labelled data fields, **When** the developer uses the shared FieldLabel component, **Then** it renders with the uppercase, small, slate-400 styling used for definition terms

---

### Edge Cases

- What happens when a Button component receives no children? It should render as an empty button (the consumer is responsible for providing content).
- What happens when a TextField is used without an error signal? The error area should simply not render — no empty space.
- What happens when the TypeBadge component receives an unknown item type? It should fall back to a neutral grey badge.
- What happens when the Card component receives no content? It should render as an empty card container — no minimum height enforced.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The application MUST provide a shared Button component with at least three visual variants: primary, secondary, and danger
- **FR-002**: The shared Button component MUST support an `on:click` handler and arbitrary child content (text or nested elements)
- **FR-003**: The application MUST provide a shared TextField component that wraps a label, text input, and optional error display
- **FR-004**: The shared TextField component MUST support required/optional indicators, placeholder text, and integration with the existing validation error system
- **FR-005**: The application MUST provide a shared TextArea component with the same label and error integration as TextField
- **FR-006**: The application MUST provide an AppHeader component that renders the application name, tagline, and version badge
- **FR-007**: The application MUST provide an AppFooter component that renders the informational footer text
- **FR-008**: The application MUST provide a Card component that renders the standard white container with shadow and border
- **FR-009**: The application MUST provide a TypeBadge component that renders a coloured badge based on item type (piece or exercise)
- **FR-010**: The application MUST provide a PageHeading component for consistent page-level headings
- **FR-011**: The application MUST provide a BackLink component for consistent back-navigation links
- **FR-012**: The application MUST provide a FieldLabel component for consistent definition-term labels in data display contexts
- **FR-013**: All extracted components MUST produce visually identical output to the current inline markup they replace
- **FR-014**: The existing test suite MUST continue to pass without modification after extraction
- **FR-015**: The application MUST compile to WASM without errors after all component extractions

## Assumptions

- This is a pure refactoring — no new user-facing functionality or visual changes are introduced
- The existing Tailwind CSS class-based styling approach is retained; no CSS-in-JS or design token system is introduced at this stage
- Components are composable — a Card can contain a PageHeading, for example — but there is no enforced composition hierarchy
- The FormFieldError component (already extracted in feature 005) remains as-is and is used within the new TextField and TextArea components
- Components accept only the props they need; they do not accept a generic "className" override (to enforce visual consistency)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every button in the application uses the shared Button component — zero inline button class strings remain in view files
- **SC-002**: Every form label-input pair in add/edit views uses the shared TextField or TextArea component — zero inline label or input class strings remain in form views
- **SC-003**: The application compiles and all existing tests pass without modification
- **SC-004**: The application compiles to WASM without errors
- **SC-005**: No individual file exceeds 300 lines (maintaining the standard from feature 005)
- **SC-006**: The visual appearance of every page is identical before and after the refactoring (verified via manual comparison or screenshot comparison)
- **SC-007**: The total number of lines of duplicated CSS class strings across view files is reduced by at least 50%
- **SC-008**: Zero clippy warnings across the entire workspace
