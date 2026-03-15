# Feature Specification: iOS Design System Foundation

**Feature Branch**: `194-ios-design-system`
**Created**: 2026-03-15
**Status**: Draft
**Input**: GitHub issue #194 — "iOS: Design system foundation — tokens, shared components & navigation"

## Guiding Principle

**Design consistency and reuse are paramount.** The iOS app must be visually indistinguishable from the web app. Both platforms share the same dark glassmorphism aesthetic, the same token names, and the same component vocabulary. This design system is the foundation that every future iOS view will be built on — get it right once, reuse everywhere.

The architecture follows the same layered approach as the web: **Tokens → Modifiers → Components → Views**. No view should ever contain raw colours, ad-hoc fonts, or hardcoded spacing. Everything traces back to a named token.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Visual Consistency Across Platforms (Priority: P1)

A user who already uses intrada on the web opens the iOS app for the first time. They immediately recognise the dark-on-dark glassmorphism aesthetic, warm indigo accents, and serif heading font. The app feels like the same product — not a generic iOS app with default styling.

**Why this priority**: Brand cohesion is the foundation. Without matching tokens, every subsequent iOS view will be built with ad-hoc colours that diverge from the web, creating an inconsistent experience.

**Independent Test**: Can be tested by comparing screenshots of the iOS design catalogue with the web design catalogue — colours, typography, and spacing should be visually equivalent.

**Acceptance Scenarios**:

1. **Given** the iOS app is open, **When** a user views any screen, **Then** all colours match the web app's named token values (text-primary, text-secondary, bg-surface-secondary, accent, etc.)
2. **Given** the iOS app is open, **When** a user views a page heading, **Then** the heading uses a serif font consistent with the web's heading font
3. **Given** the iOS app is open, **When** a user views any card or container, **Then** the glassmorphism styling matches the web's glass-card treatment

---

### User Story 2 - Reusable Component Library (Priority: P1)

A developer building the Library or Sessions tab can compose views from pre-built components (Card, Button, TextField, StatCard, TypeBadge, etc.) rather than writing one-off markup. Each component uses design tokens internally, so styling is automatically correct.

**Why this priority**: Equal to P1 because components and tokens are co-dependent — tokens without components are academic, components without tokens drift.

**Independent Test**: Can be tested by building a SwiftUI Preview catalogue that renders every component in isolation, verifying visual correctness.

**Acceptance Scenarios**:

1. **Given** a developer needs a card container, **When** they use `CardView`, **Then** it renders with glassmorphism background, correct border, and standard padding
2. **Given** a developer needs a form input, **When** they use `TextFieldView` with a label, hint, and error state, **Then** all three are rendered with correct token-based styling
3. **Given** a developer needs a primary action button, **When** they use `ButtonView(.primary)`, **Then** it renders with the accent colour background and correct typography

---

### User Story 3 - Polished Navigation Shell (Priority: P2)

A user navigates between the four main tabs (Library, Practice, Routines, Analytics). Each tab has a recognisable icon, the tab bar matches the app's dark glassmorphism chrome, and the currently selected tab is clearly highlighted with the accent colour.

**Why this priority**: Navigation is already functional but uses default iOS styling. Polishing it to match the design language is important but less foundational than tokens and components.

**Independent Test**: Can be tested by navigating between tabs and verifying the tab bar appearance, icon clarity, and selection state.

**Acceptance Scenarios**:

1. **Given** the app is open on any tab, **When** the user looks at the tab bar, **Then** it uses the app's chrome styling (not default iOS translucent grey)
2. **Given** a tab is selected, **When** the user views the tab bar, **Then** the selected tab icon and label use the accent colour
3. **Given** the user taps a different tab, **When** the tab switches, **Then** the transition is smooth and the new tab's content loads

---

### User Story 4 - Loading & Empty States (Priority: P2)

A user opens a tab while data is loading from the API. They see skeleton placeholders that pulse in the app's visual style. If there's no data yet, they see a friendly empty state with a clear call-to-action.

**Why this priority**: Loading and empty states are the first things users see. Getting them right prevents the app from feeling broken or unfinished.

**Independent Test**: Can be tested by disconnecting from the network (loading state) or using an account with no data (empty state).

**Acceptance Scenarios**:

1. **Given** data is loading, **When** the user views a list, **Then** skeleton placeholders appear with the correct surface colour and pulsing animation
2. **Given** no items exist, **When** the user views the Library tab, **Then** an empty state message and call-to-action button appear

---

### User Story 5 - Error & Feedback Display (Priority: P3)

When an API call fails or the user performs an action, feedback appears as a toast notification or error banner using the app's design language — not a generic iOS alert.

**Why this priority**: Error handling is important but most actions will succeed. Getting the foundation right first means errors can be styled correctly when feature views are built.

**Independent Test**: Can be tested by triggering an API error (e.g. invalid data) and verifying the toast/banner appears correctly styled.

**Acceptance Scenarios**:

1. **Given** an API call fails, **When** the error response arrives, **Then** an ErrorBanner appears with danger-coloured text and surface styling
2. **Given** a user performs a successful action, **When** the action completes, **Then** a Toast notification briefly appears with success styling

---

### Edge Cases

- What happens when the device is in Light Mode? The app should force dark appearance since the entire design language is built around a dark palette.
- What happens with Dynamic Type (accessibility text sizes)? Components should respect the user's preferred text size where practical.
- What happens on smaller devices (iPhone SE)? Components should remain usable with compact spacing.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: App MUST force dark appearance system-wide, matching the web app's dark-on-dark design language
- **FR-002**: All colour tokens defined in the web's `input.css` MUST have equivalent SwiftUI `Color` extensions using matching token names
- **FR-003**: Typography scale MUST include heading (serif), body, label, muted, and faint text styles as SwiftUI `ViewModifier` or `Font` extensions
- **FR-004**: Spacing constants MUST be defined for card padding (compact/standard/comfortable), section spacing, and grid gaps
- **FR-005**: A `CardView` component MUST render a glassmorphism container with the correct background, border, corner radius, and padding
- **FR-006**: A `ButtonView` component MUST support at minimum Primary, Secondary, and Danger variants with correct colours and typography
- **FR-007**: A `TextFieldView` component MUST render a labelled text input with optional hint text, error state, and validation styling
- **FR-008**: A `TextAreaView` component MUST render a labelled multi-line text input with the same options as TextFieldView
- **FR-009**: A `StatCardView` component MUST display a metric with title, value, and optional subtitle
- **FR-010**: A `TypeBadge` component MUST display a Piece or Exercise type pill with appropriate styling
- **FR-011**: A `PageHeading` component MUST render a serif-font page title consistent with the web app
- **FR-012**: `Toast` and `ErrorBanner` components MUST display feedback and errors using status colour tokens
- **FR-013**: `SkeletonLine` and `SkeletonBlock` components MUST display pulsing loading placeholders using the surface-secondary token
- **FR-014**: The tab bar MUST be styled to match the app's chrome/glassmorphism aesthetic
- **FR-015**: The existing `EmptyStateView` MUST be updated to use the new design tokens
- **FR-016**: A `FormFieldError` component MUST display inline validation errors in danger colour
- **FR-017**: A `BackLink` component MUST provide back-navigation with an arrow icon
- **FR-018**: All components MUST support Dynamic Type for accessibility
- **FR-019**: A SwiftUI Preview catalogue MUST exist that showcases all components for developer reference

### Key Entities

- **Design Token**: A named value (colour, spacing, font, radius) that maps 1:1 to the web app's CSS custom properties. Ensures visual consistency across platforms.
- **Component**: A reusable SwiftUI view that encapsulates a visual pattern (card, button, input field) and internally uses design tokens for all styling.
- **Component Variant**: A configuration of a component that changes its appearance (e.g. Primary vs Danger button, or compact vs comfortable card padding).

## Design *(include if feature has UI)*

### Existing Components Used

- `EmptyStateView` — Already exists, needs token alignment update

### New Components Needed

- **CardView**: Glassmorphism container with configurable padding (compact/standard/comfortable). Displays content in a frosted-glass card with subtle border.
- **ButtonView**: Tappable action with Primary (accent fill), Secondary (surface fill), and Danger (danger fill) variants. Supports disabled state.
- **TextFieldView**: Form input with label above, input field with surface-input background, optional hint text below, and error state with red border + FormFieldError.
- **TextAreaView**: Multi-line version of TextFieldView.
- **StatCardView**: Metric card showing a title (field-label style), a large value, and an optional subtitle. Used for analytics and summaries.
- **TypeBadge**: Small pill showing "Piece" or "Exercise" with distinct background colours.
- **PageHeading**: Serif-font heading for page titles. Maps to the web's `font-heading` class.
- **Toast**: Temporary notification that slides in from the top, shows a message with status colour, and auto-dismisses.
- **ErrorBanner**: Persistent error display with danger styling and optional dismiss action.
- **FormFieldError**: Inline text below a form field showing a validation error in danger colour.
- **BackLink**: Navigation link with a left-arrow icon and accent-coloured text.
- **SkeletonLine**: Pulsing text-width placeholder for loading states.
- **SkeletonBlock**: Pulsing rectangular placeholder for loading states.

### Wireframe / Layout Description

This feature does not introduce new screens — it provides the building blocks for future screens. The deliverable is a SwiftUI Preview catalogue that showcases all components and tokens. The catalogue should be accessible during development via Xcode Previews.

### Responsive Behaviour

- **iPhone SE / compact**: Components use compact padding tokens. Text respects Dynamic Type scaling.
- **Standard iPhone**: Components use standard padding tokens.
- **iPad**: No specific iPad layout required at this stage — standard iPhone layout will display correctly.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All named colour tokens from the web design system have matching SwiftUI Color extensions (100% coverage of the token table in CLAUDE.md)
- **SC-002**: At least 13 reusable components are built and available (Card, Button, TextField, TextArea, StatCard, TypeBadge, PageHeading, Toast, ErrorBanner, FormFieldError, BackLink, SkeletonLine, SkeletonBlock)
- **SC-003**: Every component renders correctly in Xcode Previews without runtime errors
- **SC-004**: A developer can build a new feature view using only design-system components and tokens — no raw colour values or ad-hoc styling needed
- **SC-005**: The iOS app's visual appearance is recognisably the same product as the web app when comparing equivalent UI patterns
- **SC-006**: All components support Dynamic Type at the default and two largest accessibility text sizes
