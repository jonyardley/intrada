# Feature Specification: Web App Component Architecture

**Feature Branch**: `005-component-architecture`
**Created**: 2026-02-14
**Status**: Draft
**Input**: User description: "Using atomic design principles (or whatever modern approach is now best practice) componentise the web app"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Extract Shared UI Building Blocks (Priority: P1)

The web application currently exists as a single 1,900-line file containing all components, helpers, validation logic, types, and stub data together. A developer working on the application needs commonly used visual building blocks — such as error messages, buttons, form fields, and badges — to be reusable across different parts of the interface. When a developer adds a new view or modifies an existing one, they should be able to compose it from a library of existing building blocks without duplicating markup or styles.

**Why this priority**: Shared building blocks are the foundation of a component architecture. Every other view and form depends on these primitives. Extracting them first enables all subsequent decomposition work to reference a single source of truth for common UI patterns.

**Independent Test**: Can be fully tested by verifying that after extraction, the existing library list view, detail view, and all forms render identically to the current state. Every shared building block is importable and usable in isolation.

**Acceptance Scenarios**:

1. **Given** the current monolithic web application, **When** a developer looks for a reusable error display element, **Then** they find it as a self-contained unit in a dedicated location alongside other shared building blocks.
2. **Given** a shared form field component exists, **When** a developer builds a new form, **Then** they can compose it from existing building blocks without copying markup from other forms.
3. **Given** all shared building blocks are extracted, **When** the application is compiled and run, **Then** it renders identically to the pre-extraction state with no visual or behavioural regressions.

---

### User Story 2 - Organise Views into Logical Groups (Priority: P1)

A developer navigating the codebase needs each major view — the library list, item detail view, add forms, and edit forms — to live in its own clearly identified location rather than being interleaved in a single file. When a developer needs to modify the detail view, they should navigate directly to it without scrolling through 1,900 lines of unrelated code.

**Why this priority**: Separating views is the highest-impact change for developer productivity. It directly reduces cognitive load and merge conflicts when multiple features are developed simultaneously.

**Independent Test**: Can be fully tested by verifying that each view is located in its own identifiable unit, the application compiles without errors, and all user flows (list, detail, add, edit, delete) work exactly as before.

**Acceptance Scenarios**:

1. **Given** the current monolithic file, **When** a developer wants to modify the detail view, **Then** they can find it in a dedicated, clearly named location without reading through list or form code.
2. **Given** views are separated, **When** a developer modifies a form view, **Then** changes do not risk accidentally affecting unrelated views like the list or detail view.
3. **Given** all views are separated, **When** the full application is compiled and run, **Then** all existing functionality works identically — list, detail, add piece, add exercise, edit piece, edit exercise, and delete with confirmation.

---

### User Story 3 - Isolate Non-Visual Logic from Views (Priority: P2)

A developer maintaining the application needs validation functions, parsing helpers, type definitions, and stub data management to be separate from visual components. When validation rules change (e.g., a new maximum length for titles), the developer should find and modify validation logic in one place without navigating through view code.

**Why this priority**: Separating logic from presentation improves testability and makes it straightforward to update business rules without risk of breaking visual layouts. It also enables future reuse of these helpers across different shells (e.g., if a mobile shell is added).

**Independent Test**: Can be fully tested by verifying that all validation functions, parsers, and type definitions are accessible from their new locations, the application compiles, and submitting forms with valid and invalid data produces the same results as before.

**Acceptance Scenarios**:

1. **Given** validation logic is extracted, **When** a developer needs to change the title length limit, **Then** they find all validation rules in a single dedicated location, not scattered across form components.
2. **Given** type definitions (like ViewState) are in their own location, **When** a new view state is added, **Then** the developer updates types in one place and the change is visible to all components that reference it.
3. **Given** stub data helpers are separated from views, **When** the data source changes (e.g., from stubs to persistence), **Then** only the data management code needs updating, not the view code.

---

### User Story 4 - Establish Consistent Component Organisation Conventions (Priority: P2)

A developer joining the project or returning after time away needs to understand the organisational approach quickly. The component structure should follow a clear, documented convention so that when a new feature is added, the developer knows exactly where to place new components and how to structure them relative to existing ones.

**Why this priority**: Conventions prevent the codebase from drifting back into a monolithic structure as features are added. Without clear patterns, future development risks recreating the same disorganisation.

**Independent Test**: Can be fully tested by verifying that a developer unfamiliar with the codebase can locate any given component within 30 seconds, and that adding a hypothetical new view follows an obvious pattern established by existing views.

**Acceptance Scenarios**:

1. **Given** the refactored component structure, **When** a new developer explores the project, **Then** they can identify the location of any component (e.g., the edit piece form) within 30 seconds.
2. **Given** clear conventions exist, **When** a developer needs to add a new view (e.g., a settings page), **Then** the pattern for where to place it and how to structure it is obvious from the existing organisation.
3. **Given** the application is restructured, **When** reviewing the project file layout, **Then** all related files are grouped by purpose, not spread arbitrarily.

---

### Edge Cases

- What happens when the refactored application is compiled with existing tests — do all 82+ tests still pass without modification?
- How does the system handle circular dependencies between components that reference each other (e.g., list view navigating to detail, detail navigating to edit)?
- What happens if a shared building block is modified — do all consumers reflect the change consistently?
- How does the application behave when compiled after restructuring — are there any runtime regressions in the browser (WASM-specific concerns)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The application MUST render identically after restructuring — zero visual or behavioural regressions across all views (list, detail, add piece, add exercise, edit piece, edit exercise, delete confirmation)
- **FR-002**: Shared visual building blocks (error messages, type badges, tag displays, metadata fields, action buttons) MUST be reusable units that can be composed into any view
- **FR-003**: Each major view (library list, item detail, add piece form, add exercise form, edit piece form, edit exercise form) MUST be in its own dedicated location, separate from other views
- **FR-004**: Validation logic (piece form validation, exercise form validation) MUST be isolated from visual component code
- **FR-005**: Parsing helpers (tag parsing, tempo parsing, tempo display parsing) MUST be isolated from visual component code
- **FR-006**: Type definitions (ViewState, SharedCore) MUST be in a shared location accessible to all components
- **FR-007**: Stub data creation and sample data constants MUST be separated from visual components
- **FR-008**: The effect processing function MUST be in a shared location accessible to any component that dispatches events
- **FR-009**: All existing tests (82+ across the workspace) MUST pass without modification after restructuring
- **FR-010**: The application MUST compile to WASM and serve via the existing build tooling without configuration changes
- **FR-011**: The component organisation MUST follow a discoverable convention where the location of any component can be determined by its name or purpose

### Assumptions

- This is a pure refactoring effort — no new features, no changed behaviour, no new UI elements
- The restructuring applies only to the web shell (the Crux core remains unchanged)
- The existing build tooling and configuration continue to work after restructuring
- The application will continue to use the same styling approach (Tailwind CSS utility classes inline in components)
- No new dependencies are needed — this is purely an organisational change using the existing language and framework capabilities
- The "atomic design" reference in the user request is interpreted as a desire for a layered component architecture appropriate for the project's size, not necessarily a strict atoms/molecules/organisms/templates/pages hierarchy (which would be overkill for a ~10 component application)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: No single source file exceeds 300 lines (current state: one file at 1,900 lines)
- **SC-002**: A developer can locate any specific component's source within 30 seconds by following the file naming convention
- **SC-003**: All 82+ existing workspace tests pass without modification after restructuring
- **SC-004**: The application compiles to WASM and passes the existing build process without errors
- **SC-005**: Zero visual or behavioural regressions — all user flows (list, add, detail, edit, delete) work identically
- **SC-006**: Adding a hypothetical new view requires creating a new file in an obvious location, not modifying a monolithic file
- **SC-007**: Zero linting warnings across the workspace after restructuring
- **SC-008**: The total line count across all files does not increase by more than 10% compared to the current single-file total (accounting for module declarations and imports)
