# Feature Specification: Glassmorphism UI & Responsive Layout

**Feature Branch**: `016-glassmorphism-responsive`
**Created**: 2026-02-15
**Status**: Draft
**Input**: User description: "Use inspiration image as basis for glassmorphism design with transparent blurred-background cards over a gradient background. Make the app more mobile and tablet friendly."

## User Scenarios & Testing

### User Story 1 - Glassmorphism Visual Theme (Priority: P1)

As a user, I want the app to have a modern, polished visual style with a rich gradient background and translucent frosted-glass cards so the interface feels premium and cohesive.

The current app uses a plain white-card-on-light-grey aesthetic. The new design replaces this with a deep purple/indigo gradient background and glassmorphism-styled containers — semi-transparent panels with backdrop blur that let the gradient show through. This creates visual depth and a unified, immersive feel.

**Why this priority**: Visual identity is the foundation — all other UI work builds on top of the theme. Without the base gradient and glass card styles, responsive layout changes would still look like the old design.

**Independent Test**: Can be fully tested by loading any page and verifying the gradient background is visible, cards are semi-transparent with backdrop blur, and text remains readable against the glass panels.

**Acceptance Scenarios**:

1. **Given** the app is loaded on any page, **When** I view the background, **Then** I see a rich gradient (deep purple/indigo tones) filling the viewport behind all content
2. **Given** a page with card-based content (library list, detail view, sessions), **When** I view the cards, **Then** they appear as frosted glass panels — semi-transparent with a visible backdrop blur effect and a subtle border
3. **Given** any text on a glass card, **When** I read it, **Then** the text is white or light grey, providing sufficient contrast against the dark gradient showing through the translucent card background
4. **Given** the header/navigation area, **When** I view it, **Then** it uses the glass treatment consistently with the rest of the UI
5. **Given** buttons and form inputs, **When** I interact with them, **Then** they are styled to complement the glassmorphism theme (not the old opaque white style)
6. **Given** any page in the app, **When** I scroll, **Then** the gradient background remains fixed (does not scroll with content), creating a parallax-like depth effect

---

### User Story 2 - Mobile-Friendly Layout (Priority: P2)

As a user on a mobile phone, I want the app layout to adapt to my small screen so I can comfortably use all features without horizontal scrolling, tiny tap targets, or content being cut off.

The current app uses a fixed `max-w-4xl` container with no breakpoint-based layout changes. On mobile screens, content is squeezed but not re-flowed. Navigation links are horizontal and may wrap awkwardly. Cards have fixed padding that wastes space on small screens.

**Why this priority**: Mobile is the most common access pattern for a personal practice tool — musicians often use their phone on a music stand during practice. Without mobile-friendly layout, the app is frustrating to use in its primary context.

**Independent Test**: Can be fully tested by resizing the browser to 375px width (iPhone SE) and verifying all pages are usable, all content is reachable, and no horizontal scrollbar appears.

**Acceptance Scenarios**:

1. **Given** the app is viewed on a screen narrower than 640px, **When** I view any page, **Then** no horizontal scrollbar appears and all content fits within the viewport width
2. **Given** the library list on mobile, **When** I view it, **Then** cards stack vertically with appropriate padding and readable text — no content is truncated beyond what makes sense
3. **Given** the navigation on mobile, **When** I look at the screen, **Then** the header navigation links are hidden and a persistent bottom tab bar is displayed with icon+label buttons for each main section (Library, Sessions), fixed to the bottom of the viewport with glass styling
4. **Given** a form page (add/edit item) on mobile, **When** I interact with it, **Then** form fields are full-width, labels are clearly visible, and buttons are easy to tap (minimum 44px touch target)
5. **Given** the session timer on mobile, **When** I am in an active practice session, **Then** the timer display and control buttons are large enough to tap comfortably and visible without scrolling
6. **Given** any button or link on mobile, **When** I try to tap it, **Then** the touch target is at least 44x44px and there is adequate spacing between adjacent targets to prevent mis-taps

---

### User Story 3 - Tablet-Optimised Layout (Priority: P3)

As a user on a tablet (e.g. iPad), I want the app to take better advantage of the wider screen than mobile but narrower than desktop, so the layout feels purposeful rather than just a stretched mobile view or cramped desktop view.

On tablet-width screens (641px-1024px), the layout should adapt — for example, library items could display in a two-column grid, the session timer could show more context side-by-side, and padding/margins should be balanced for the medium screen size.

**Why this priority**: Tablets are a natural device for musicians at a piano or desk. The experience should feel intentional at this size, not just an accidental in-between state.

**Independent Test**: Can be fully tested by resizing the browser to 768px width (iPad portrait) and verifying the layout uses the available space effectively with multi-column layouts where appropriate.

**Acceptance Scenarios**:

1. **Given** the app is viewed on a screen between 641px and 1024px wide, **When** I view the library list, **Then** items are displayed in a two-column grid layout
2. **Given** the detail view on a tablet, **When** I view an item, **Then** metadata is laid out to use the available width efficiently (not just a single narrow column)
3. **Given** forms on a tablet, **When** I fill out the add/edit form, **Then** the form uses a comfortable width (not stretched to full screen, not cramped to mobile width)
4. **Given** the session timer on a tablet, **When** I am in an active session, **Then** the timer and item information are displayed with good use of space

---

### Edge Cases

- What happens when the device does not support `backdrop-filter` (older browsers)? The cards should fall back to a semi-opaque solid background that still provides readable contrast.
- What happens when system prefers-reduced-motion is enabled? Any gradient animations or transitions should be reduced or disabled.
- What happens when the user has high-contrast mode enabled? The glass effect should increase opacity to maintain text readability.
- What happens on very wide screens (>1400px)? Content should remain centered and constrained, not stretch to fill.
- What happens with very long text content on mobile (e.g. long notes, many tags)? Text should wrap naturally, tags should flow to new lines.
- What happens with the session timer on a very small screen (320px)? Timer digits and buttons must still be usable.
- What happens with the bottom tab bar during an active practice session? The tab bar should remain visible so users can navigate away if needed, but page content must account for the tab bar height (no content hidden behind it).

## Requirements

### Functional Requirements

- **FR-001**: The app MUST display a gradient background that fills the full viewport and remains fixed during scroll
- **FR-002**: All card-based containers MUST use glassmorphism styling — semi-transparent background with backdrop blur and subtle border
- **FR-003**: The app MUST provide a CSS fallback for browsers that do not support `backdrop-filter`, using a semi-opaque solid background colour
- **FR-004**: All text on glass cards MUST use a light colour scheme (white or light grey) and MUST meet WCAG 2.1 AA contrast ratios (4.5:1 for normal text, 3:1 for large text) against the effective card background
- **FR-005**: The layout MUST adapt at three breakpoints: mobile (<640px), tablet (640px-1024px), and desktop (>1024px)
- **FR-006**: On mobile, all interactive elements (buttons, links, form inputs) MUST have a minimum touch target of 44x44px
- **FR-007**: On mobile, no page MUST produce a horizontal scrollbar
- **FR-008**: On tablet, library item lists MUST display in a multi-column grid layout
- **FR-009**: On mobile (<640px), the header navigation links MUST be hidden and replaced by a persistent bottom tab bar fixed to the viewport bottom, with glass-styled icon+label buttons for each main section. On tablet and desktop, the full horizontal header navigation MUST remain visible and the bottom tab bar MUST be hidden
- **FR-010**: Form inputs and buttons MUST be restyled to complement the glassmorphism theme while maintaining clear affordance (users must still recognise inputs as inputs and buttons as buttons)
- **FR-011**: The header MUST use the glass treatment consistent with the card styling
- **FR-012**: All existing functionality MUST continue to work identically — this is a visual-only change with no behaviour modifications
- **FR-013**: The design MUST respect `prefers-reduced-motion` by reducing or disabling transitions and animations
- **FR-014**: On very wide screens (>1400px), content MUST remain centered and constrained to a maximum width

## Success Criteria

### Measurable Outcomes

- **SC-001**: All 14 existing E2E tests pass without modification (pure visual change, no behavioural changes)
- **SC-002**: All pages are usable at 375px viewport width with no horizontal scroll
- **SC-003**: All pages utilise multi-column layouts at 768px viewport width where appropriate
- **SC-004**: Text on glass cards meets WCAG 2.1 AA contrast ratio (4.5:1 minimum for body text)
- **SC-005**: All interactive elements have minimum 44x44px touch targets on mobile
- **SC-006**: The app loads and renders within its current performance envelope — the visual changes do not introduce perceptible rendering lag
- **SC-007**: The glassmorphism fallback renders correctly in browsers without `backdrop-filter` support

## Assumptions

- Tailwind CSS v4 remains the sole styling approach — no additional CSS frameworks or libraries will be introduced
- The glassmorphism effect can be achieved purely with Tailwind utility classes (`backdrop-blur`, `bg-white/10`, `border-white/20`, etc.)
- The gradient palette will use deep purple/indigo tones inspired by the reference design image, but exact colours will be determined during implementation
- The container constraint may be relaxed or adjusted at tablet/desktop breakpoints, but content should not stretch beyond comfortable reading width
- Most changes are to existing component class attributes; the bottom tab bar for mobile navigation will require a new component
- The sidebar navigation shown in the inspiration image is aspirational reference only; the current top-bar navigation pattern will be retained and made responsive (a sidebar redesign would be a separate feature)

## Clarifications

### Session 2026-02-15

- Q: How should navigation behave on mobile (<640px)? → A: Bottom tab bar — persistent glass-styled bar fixed to viewport bottom with icon+label buttons for each main section
- Q: Should text on glass cards be light or dark? → A: Light text — white or light grey on glass cards, replacing current dark-on-white scheme
