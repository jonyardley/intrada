# Feature Specification: Leptos Web App MVP

**Feature Branch**: `003-leptos-app-mvp`
**Created**: 2026-02-14
**Status**: Draft
**Input**: User description: "Create the MVP of a Leptos app using tailwind with good DX"

## Clarifications

### Session 2026-02-14

- Q: Should the Leptos crate be a full Crux web shell (importing `intrada-core` and wiring `Core<Intrada>`), a scaffolded shell without wiring, or a standalone crate with no core dependency? → A: Full Crux web shell — wire `Core<Intrada>` with stub effects.
- Q: What should the interactive client-side element be? → A: Mini library view displaying the Crux ViewModel (item count from stub data).
- Q: How should the web shell handle Storage effects from the Crux core? → A: Return hardcoded stub data for LoadAll, no-op for writes — shows pre-populated demo data.
- Q: Which Leptos rendering mode should the MVP use? → A: Client-side rendering (CSR) only — simplest setup, trunk serve for dev.
- Q: Should the CI pipeline be updated to verify the web crate? → A: Yes, add a trunk build check to CI to verify the web crate compiles to WASM.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Web Application Shell with Styled Landing Page (Priority: P1)

A developer clones the repository and starts the web application locally. The application builds and serves a styled landing page that renders in the browser. The page uses Intrada's branding and demonstrates that the web framework and styling system are correctly integrated.

**Why this priority**: Without a working web application shell, no further web features can be built. This is the foundation that everything else depends on.

**Independent Test**: Can be fully tested by running the development server and visiting the local URL in a browser. The landing page should render with visible styled content.

**Acceptance Scenarios**:

1. **Given** a developer has cloned the repository, **When** they run the development server, **Then** the web application starts and is accessible at a local URL.
2. **Given** the development server is running, **When** a user navigates to the root URL, **Then** a styled landing page renders with the application name and basic content.
3. **Given** the landing page is rendered, **When** the user views the page, **Then** the styling is visually correct (utility-based CSS classes are applied and visible).

---

### User Story 2 - Fast Developer Feedback Loop (Priority: P1)

A developer modifies the source code of the web application while the development server is running. The changes are reflected in the browser automatically without requiring a manual restart or full-page reload, providing a fast feedback cycle.

**Why this priority**: Fast iteration speed is critical for developer productivity. Without this, every code change requires manually rebuilding and reloading, dramatically slowing development.

**Independent Test**: Can be tested by making a visible change to the landing page content while the development server is running and observing the update appear in the browser automatically.

**Acceptance Scenarios**:

1. **Given** the development server is running and the landing page is displayed, **When** a developer changes the page content in source code, **Then** the browser reflects the change automatically within seconds.
2. **Given** the development server is running, **When** a developer changes a style, **Then** the updated styles appear in the browser without a full-page reload.
3. **Given** the development server is running, **When** a developer introduces a compile error, **Then** the error is reported clearly in the terminal and the application recovers once the error is fixed.

---

### User Story 3 - Client-Side Interactivity (Priority: P2)

A visitor to the web application sees a mini library view on the landing page, rendered from the Crux `ViewModel`. The view displays stub data (e.g., item count and list) that was loaded through the full Crux event → update → view cycle, confirming the web shell architecture is wired end-to-end.

**Why this priority**: Client-side reactivity via Crux is a core capability. Demonstrating the ViewModel rendering proves the architecture works on the web platform, not just in the CLI.

**Independent Test**: Can be tested by loading the landing page and verifying that stub library data (item count, item names) is displayed, confirming the Crux core processed a `DataLoaded` event and the view rendered the resulting `ViewModel`.

**Acceptance Scenarios**:

1. **Given** the landing page is displayed, **When** the Crux core has processed stub data, **Then** the page displays library information from the ViewModel (e.g., item count and item titles).
2. **Given** the mini library view is rendered, **When** a user triggers an action (e.g., clicking a button to add a stub item), **Then** the view updates dynamically without a full-page reload, reflecting the new ViewModel state.

---

### Edge Cases

- What happens when the developer starts the server without the required build tools installed? The application should fail with a clear error message indicating missing prerequisites.
- What happens when a CSS class is used that the styling system does not recognise? The class should be silently ignored and the element should render unstyled (standard browser behaviour).
- What happens when the developer's browser has JavaScript disabled? Since the app uses client-side rendering only, the page MUST display a `<noscript>` message indicating that JavaScript is required.
- What happens when a user triggers a write action (e.g., add item) in the web shell? The Crux core processes the event and the ViewModel updates in-memory, but no data is persisted. On page reload, the view resets to the hardcoded stub data.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The web application MUST serve a landing page at the root URL (`/`) when the development server is running.
- **FR-002**: The landing page MUST display the application name ("Intrada") and a brief description.
- **FR-003**: The landing page MUST be styled using a utility-first CSS approach with visible, correct styling.
- **FR-004**: The development server MUST automatically rebuild and update the browser when source files change.
- **FR-005**: The landing page MUST include a mini library view that displays data from the Crux `ViewModel` (e.g., item count), demonstrating the full Crux event → update → view round-trip with stub data. The view MUST update dynamically without a full-page reload.
- **FR-006**: The application MUST produce a clear error message if the build fails, visible in the terminal where the development server was started.
- **FR-007**: The web application MUST coexist with the existing CLI application in the same repository without breaking the CLI build or tests.
- **FR-008**: The web application MUST render correctly in current versions of major browsers (Chrome, Firefox, Safari).
- **FR-009**: The web application MUST be implemented as a Crux web shell, importing `intrada-core` and wiring `Core<Intrada>` with stub effect handlers. The `LoadAll` storage effect MUST return hardcoded stub data (at least 2 sample library items). All write storage effects (save, update, delete) MUST be silently ignored (no-op). This validates the end-to-end Crux architecture for the web platform.
- **FR-010**: The web application MUST use client-side rendering (CSR) only. A `<noscript>` element MUST inform users that JavaScript is required.
- **FR-011**: The CI pipeline MUST include a job that verifies the web crate compiles to WASM (e.g., `trunk build`). This job MUST run alongside the existing test, clippy, and fmt checks.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can go from a clean clone to a running development server in under 3 minutes (excluding dependency download time).
- **SC-002**: Source code changes are reflected in the browser within 5 seconds of saving the file.
- **SC-003**: The existing CLI test suite (82 tests) continues to pass without modification after the web application is added.
- **SC-004**: The landing page achieves a Lighthouse accessibility score of 90 or above.
- **SC-005**: The CI pipeline passes with all existing jobs (test, clippy, fmt) plus the new WASM build check.

## Scope & Assumptions

### In Scope

- Web application shell with a styled landing page
- Development server with automatic rebuild on file changes
- Client-side interactivity demonstration
- Integration into the existing Cargo workspace
- Full Crux web shell wiring (`Core<Intrada>` with stub effect handlers)
- CI pipeline update with WASM build verification

### Out of Scope

- User authentication or accounts
- Functional connection to the existing music library data (SQLite) — stub effects only, no real persistence
- Routing beyond the landing page
- Production deployment or hosting
- Server-side API endpoints
- Mobile-responsive layout (acceptable for MVP)

### Assumptions

- The developer has Rust stable toolchain installed (existing project requirement).
- The web application will be added as a new crate within the existing Cargo workspace.
- The project already uses a workspace structure (`crates/`) that can accommodate additional members.
- The user has specified Leptos (Rust web framework) and Tailwind CSS (utility-first CSS) as technology choices — these are treated as requirements, not implementation decisions.
- "Good DX" means fast feedback loops (hot-reload), clear error messages, and minimal setup steps.
- The web application uses client-side rendering (CSR) only. Server-side rendering (SSR) is deferred to a future feature.
