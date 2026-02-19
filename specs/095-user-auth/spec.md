# Feature Specification: User Authentication

**Feature Branch**: `095-user-auth`
**Created**: 2026-02-18
**Status**: Draft
**Input**: User description: "078-user-authentication"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Sign In to Access Library (Priority: P1)

A musician opens Intrada for the first time (or after signing out). They see a sign-in screen with a "Sign in with Google" button. After clicking it and completing the Google OAuth flow, they are redirected back to Intrada and see their personal library. All data they create (items, routines, sessions) is private to their account.

**Why this priority**: Without authentication, all data is global and unprotected. This is the foundational gate — no other user-facing feature works without it.

**Independent Test**: Can be fully tested by signing in with a Google account and verifying that the library loads with user-specific data. Delivers the core value of private, per-user data.

**Acceptance Scenarios**:

1. **Given** an unauthenticated user, **When** they open the app, **Then** they see a sign-in screen with a "Sign in with Google" button and cannot access library, routines, or sessions.
2. **Given** a user on the sign-in screen, **When** they click "Sign in with Google" and complete the OAuth flow, **Then** they are redirected to their library view.
3. **Given** a signed-in user, **When** they make any API request (list items, create item, etc.), **Then** the request includes valid credentials and the server returns only data belonging to that user.

---

### User Story 2 - Data Isolation Between Users (Priority: P1)

When two different users sign in, each sees only their own library items, routines, and practice sessions. User A cannot see, modify, or delete User B's data through any means.

**Why this priority**: Data privacy is a core security requirement — equal priority to sign-in itself.

**Independent Test**: Can be tested by creating data as User A, signing in as User B, and verifying User B's library does not contain User A's data.

**Acceptance Scenarios**:

1. **Given** User A has created 5 library items, **When** User B signs in, **Then** User B sees zero items (or only their own).
2. **Given** User A has an item with a known identifier, **When** User B attempts to access that item directly, **Then** the system returns a "not found" response.
3. **Given** User A has practice sessions, **When** User B views analytics, **Then** User B sees only their own session data.

---

### User Story 3 - Sign Out (Priority: P2)

A signed-in user can sign out from the app. After signing out, they are returned to the sign-in screen. Their session is invalidated and they cannot access protected data until they sign in again.

**Why this priority**: Essential for shared devices and security hygiene, but secondary to the ability to sign in.

**Independent Test**: Can be tested by signing in, clicking sign out, and verifying the sign-in screen appears and API requests are rejected.

**Acceptance Scenarios**:

1. **Given** a signed-in user, **When** they click the sign-out button, **Then** they are returned to the sign-in screen.
2. **Given** a user who just signed out, **When** they attempt to navigate to a protected route directly, **Then** they are redirected to the sign-in screen.

---

### User Story 4 - Persistent Session Across Page Reloads (Priority: P2)

A signed-in user refreshes the page or closes and reopens the browser tab. They remain signed in and see their library without having to sign in again, until their session expires or they explicitly sign out.

**Why this priority**: Without session persistence, users would need to sign in on every page load, creating an unusable experience.

**Independent Test**: Can be tested by signing in, refreshing the page, and verifying the user is still signed in with their data visible.

**Acceptance Scenarios**:

1. **Given** a signed-in user, **When** they refresh the browser, **Then** they see their library without a sign-in prompt.
2. **Given** a signed-in user, **When** their session token has expired, **Then** they are redirected to the sign-in screen on the next interaction.

---

### Edge Cases

- What happens when a user's authentication token expires mid-session (e.g., while filling out a form)? The system silently refreshes the token in the background. Only if the refresh fails entirely should the user be prompted to re-authenticate. No unsaved work is discarded due to token expiry.
- What happens when the authentication provider is temporarily unavailable? The app should show a clear error message rather than a blank screen.
- What happens when a user signs in for the very first time? They should see an empty library with appropriate onboarding messaging (existing empty-state behavior).
- What happens if the user denies Google OAuth permissions? They remain on the sign-in screen with a clear message.
- What happens on an API request with an invalid or tampered token? The server returns a 401 Unauthorized response.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST require authentication before any library, routine, or session data can be accessed.
- **FR-002**: System MUST support Google OAuth as the sign-in method via a managed authentication provider (Clerk).
- **FR-003**: System MUST include a valid authentication credential with every API request from the web client.
- **FR-004**: System MUST validate authentication credentials on every API request on the server side.
- **FR-005**: Server MUST scope all data queries (items, sessions, routines) by the authenticated user's identity, ensuring complete data isolation.
- **FR-006**: Server MUST return 401 Unauthorized for requests with missing, expired, or invalid credentials.
- **FR-007**: System MUST persist the user's session across page reloads and browser restarts (until expiry or explicit sign-out).
- **FR-008**: System MUST provide a sign-out mechanism that clears the user's session and returns them to the sign-in screen.
- **FR-009**: The health-check endpoint MUST remain publicly accessible without authentication.
- **FR-010**: Existing data created before authentication is enabled will be associated with an "anonymous" user identity and will not be accessible to authenticated users. This is acceptable as the app is pre-launch.

### Key Entities

- **User Identity**: A unique identifier for each person using the app. Provided by the authentication service. Associated with all data the user creates (items, routines, sessions).
- **Authentication Token**: A short-lived credential issued after successful sign-in. Included in every API request. Validated by the server on each request.

## Design *(include if feature has UI)*

### Existing Components Used

- **AppHeader** — Will be extended with a sign-out button/icon
- **Empty state messaging** — Reused for first-time users who have no data yet

### New Components Needed

- **Sign-In Screen**: A full-page view shown to unauthenticated users. Displays the Intrada logo/branding and a "Sign in with Google" button. Centered layout with minimal distraction.
- **Auth Gate**: A wrapper that checks authentication status on app load. Shows a loading indicator while checking, the sign-in screen if unauthenticated, or the main app if authenticated.

### Wireframe / Layout Description

**Sign-In Screen**:
- Vertically and horizontally centered content
- Intrada logo/title at top
- "Sign in with Google" button below, styled consistently with the app's glassmorphism design system
- Subtle background consistent with existing app aesthetic

**App Header (updated)**:
- Existing navigation remains unchanged
- Sign-out action added to the header (icon or text link), positioned at the trailing end

### Responsive Behaviour

- **Mobile**: Sign-in screen uses full viewport width with appropriate padding. Sign-out action in header remains accessible.
- **Desktop**: Sign-in screen content is centered within the viewport. No layout differences beyond standard responsive behavior.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Unauthenticated users cannot access any library, routine, or session data — all API requests without valid credentials return 401.
- **SC-002**: Users can complete the sign-in flow (click button through to seeing their library) in under 10 seconds.
- **SC-003**: Signed-in users see only their own data — zero data leakage between user accounts.
- **SC-004**: Users remain signed in across page reloads without re-entering credentials.
- **SC-005**: All existing E2E tests continue to pass with authentication mocked in the test environment.
- **SC-006**: The sign-out flow returns the user to the sign-in screen within 2 seconds.

## Clarifications

### Session 2026-02-18

- Q: What should happen when a token expires mid-session (e.g., during form entry or active practice session)? → A: Silently refresh the token in the background; only show sign-in prompt if refresh fails entirely.

## Assumptions

- The app is pre-launch, so migration of existing anonymous data to specific users is not required. Existing anonymous data can remain inaccessible to authenticated users.
- Google is the only sign-in method needed initially. Additional providers (email/password, Apple, etc.) may be added later but are out of scope.
- Clerk is the chosen managed authentication provider. The user has confirmed this choice.
- No user profile management (display name, avatar, preferences) is in scope — the authentication provider handles profiles.
- No invitation or sharing features are in scope — this is single-user-per-account data isolation only.
- Rate limiting and abuse prevention are handled by the authentication provider and are out of scope for this feature.
