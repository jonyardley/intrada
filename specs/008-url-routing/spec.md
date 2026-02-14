# Feature Specification: URL Routing for Web App Views

**Feature Branch**: `008-url-routing`
**Created**: 2026-02-14
**Status**: Draft
**Input**: User description: "Add URL routing and implement routes for all existing views in the web app"

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Every View Has a Unique URL (Priority: P1)

A user navigates through the web application and sees the browser address bar update to reflect their current location. When viewing the library list, they see the root path. When they click into a piece or exercise, the URL changes to show the item's identity. When they navigate to add or edit forms, the URL similarly reflects the current view. This allows users to see where they are at all times and gives the browser a meaningful location for each page.

**Why this priority**: Without distinct URLs, no other routing feature (bookmarking, sharing, back/forward) can work. This is the foundational requirement that all other stories build upon.

**Independent Test**: Navigate through every view in the app and verify the browser address bar displays a distinct, descriptive URL for each. The application renders the correct content for each URL.

**Acceptance Scenarios**:

1. **Given** the user opens the app at the root URL, **When** the page loads, **Then** the library list view is displayed and the URL shows the library path
2. **Given** the user is on the library list, **When** they click on an item, **Then** the detail view is displayed and the URL updates to include the item identifier
3. **Given** the user is on the library list, **When** they choose to add a new piece, **Then** the add piece form is displayed and the URL updates to the add-piece path
4. **Given** the user is on the library list, **When** they choose to add a new exercise, **Then** the add exercise form is displayed and the URL updates to the add-exercise path
5. **Given** the user is viewing an item's detail page, **When** they choose to edit the item, **Then** the edit form is displayed and the URL updates to include the item identifier and an edit indicator
6. **Given** the user navigates to any valid URL directly (e.g. by typing it into the address bar), **When** the page loads, **Then** the correct view is rendered for that URL

---

### User Story 2 — Browser Back and Forward Navigation Works (Priority: P2)

A user navigates through multiple views in the application (e.g. library list, then item detail, then edit form, then back to detail, then back to library). When they press the browser back button, they return to the previous view. When they press the browser forward button, they go forward again. The application behaves like a standard multi-page website in terms of history navigation, even though it is a single-page application.

**Why this priority**: Browser back and forward is the most fundamental navigation expectation web users have. Without it, the app feels broken compared to any standard website.

**Independent Test**: Navigate through a sequence of at least four views, then press Back repeatedly to retrace each step, then press Forward to go forward again. Each navigation returns the user to the correct view with the correct content.

**Acceptance Scenarios**:

1. **Given** the user has navigated from the library list to an item detail, **When** they press the browser back button, **Then** the library list view is displayed and the URL reverts to the library path
2. **Given** the user has navigated Library → Detail → Edit, **When** they press Back twice, **Then** they return first to the detail view, then to the library list, with correct URLs at each step
3. **Given** the user has gone back to a previous view, **When** they press the browser forward button, **Then** they are returned to the view they navigated away from
4. **Given** the user submits a form (add or edit), **When** the form is saved and the view transitions to the next screen, **Then** pressing Back does NOT resubmit the form or return to the completed form; it navigates to the view prior to the form

---

### User Story 3 — Users Can Bookmark and Share Links to Views (Priority: P3)

A user can copy the URL from their browser address bar while viewing any page in the application and share it (via messaging, email, or bookmarks). When the recipient opens that link — or the user returns to the bookmark — the same view is loaded with the same content. This enables users to reference specific items in their library by URL.

**Why this priority**: Bookmarking and sharing are high-value features but depend on US1 (unique URLs) and US2 (history integration) being in place first.

**Independent Test**: Copy the URL while viewing a specific item's detail page, open that URL in a new browser tab (or after a full page reload), and verify the same item detail is displayed.

**Acceptance Scenarios**:

1. **Given** the user is viewing a specific item's detail page, **When** they copy the URL and open it in a new browser tab, **Then** the same item detail view is rendered
2. **Given** the user bookmarks the library list page, **When** they return to the bookmark, **Then** the library list is displayed
3. **Given** the user refreshes the browser on any view, **When** the page reloads, **Then** the same view is restored (including the correct item for detail and edit views)
4. **Given** the user shares a link to an item detail view, **When** the recipient opens the link for the first time, **Then** they see the same item detail view without needing any prior navigation

---

### User Story 4 — Unrecognised URLs Show a Helpful Message (Priority: P4)

A user navigates to a URL that does not match any known route (e.g. a mistyped path or an outdated link). Instead of a blank screen or a cryptic error, they see a clear message explaining the page was not found and offering navigation back to the library.

**Why this priority**: Error handling for invalid routes is important for robustness but only matters once valid routes exist.

**Independent Test**: Navigate to an invalid URL path and verify a user-friendly not-found message is displayed with a link to the library.

**Acceptance Scenarios**:

1. **Given** a URL that does not match any defined route, **When** the user navigates to it, **Then** a clear "page not found" message is displayed
2. **Given** the not-found message is displayed, **When** the user looks at the page, **Then** there is a visible link or button to return to the library list
3. **Given** a URL that contains an item identifier that no longer exists, **When** the user navigates to it, **Then** the application handles the missing item gracefully (not-found message or redirect to the library)

---

### Edge Cases

- What happens when a user navigates to an edit URL for an item type that doesn't match (e.g. edit-piece URL but the ID belongs to an exercise)?
- What happens when a user navigates directly to a form URL (add or edit) and then submits — does the URL correctly transition to the post-submission view?
- How does the application behave if the URL contains a valid path structure but a malformed or non-existent item identifier?
- What happens if the user manually modifies the URL from one view to another while unsaved form data exists?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The application MUST assign a unique URL path to each view: library list, item detail, add piece, add exercise, edit piece, and edit exercise
- **FR-002**: The application MUST render the correct view when a user navigates directly to any valid URL (direct navigation and deep linking)
- **FR-003**: The browser address bar MUST update to reflect the current view whenever the user navigates within the application
- **FR-004**: The browser back button MUST return the user to the previous view with the correct URL
- **FR-005**: The browser forward button MUST return the user to the next view in history with the correct URL
- **FR-006**: Item-specific views (detail, edit) MUST include the item identifier in the URL so the correct item is loaded on direct navigation
- **FR-007**: The application MUST display a user-friendly not-found message when a URL does not match any defined route
- **FR-008**: The not-found message MUST include a way to navigate back to the library list
- **FR-009**: All existing navigation actions (clicking items, pressing buttons, submitting forms) MUST continue to work identically to the current behaviour, now producing URL changes in addition to view changes
- **FR-010**: A full browser refresh on any view MUST reload the same view (including the correct item for detail and edit views)
- **FR-011**: After a form submission (add or edit), the browser history MUST NOT allow the user to navigate back to the submitted form via the back button (the form submission should replace the history entry, not add a new one)
- **FR-012**: All existing accessibility attributes and keyboard navigation MUST be preserved through the routing changes

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every view in the application is accessible via a distinct, human-readable URL path
- **SC-002**: Direct navigation to any valid URL (typed into the address bar, opened from a bookmark, or opened in a new tab) renders the correct view
- **SC-003**: Browser back and forward buttons navigate correctly through at least a 5-step navigation sequence without errors
- **SC-004**: Full-page refresh on every view preserves the current view and content
- **SC-005**: An unrecognised URL displays a clear not-found message with a link to the library
- **SC-006**: All existing automated tests continue to pass after routing is implemented
- **SC-007**: Zero compiler and linter warnings across the entire workspace after implementation
- **SC-008**: No user-facing behaviour changes to existing views (forms, validation, interactions) other than the addition of URL synchronisation

## Scope

### In Scope

- Adding URL routing to the existing web application views
- Mapping each existing view to a unique URL path
- Integrating browser history (back and forward) with application navigation
- Enabling deep linking and page refresh support
- Handling unknown routes with a not-found view
- Preserving all existing functionality, accessibility, and visual design

### Out of Scope

- Adding new views or pages beyond what currently exists
- Server-side rendering or static site generation
- URL-based search query parameters or filters (library search state)
- Authentication or route-level access control
- Analytics or tracking based on routes
- Changing the visual design or layout of any existing view

## Assumptions

- The application will continue to run as a client-side rendered (CSR) single-page application
- URL paths will be handled client-side; the hosting environment serves the same HTML entry point for all routes (standard SPA fallback)
- Item identifiers used in URLs are the existing ULID strings
- The existing view state model can be adapted or replaced to work with URL-based routing
- The UI framework's built-in routing capabilities are sufficient (no custom routing engine needed)
