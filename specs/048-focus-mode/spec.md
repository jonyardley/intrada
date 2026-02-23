# Feature Specification: Focus Mode

**Feature Branch**: `048-focus-mode`
**Created**: 2026-02-23
**Status**: Draft
**Input**: GitHub Issue #48 — Focus mode — minimal UI during active practice

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Distraction-Free Practice (Priority: P1)

A musician starts a practice session and the active practice screen shows only
what's immediately relevant: the current item name, a visual timer, and the
controls needed to move through the session. The navigation bar, completed items
list, and any other non-essential UI elements are hidden. The musician can focus
entirely on playing.

**Why this priority**: This is the core value proposition — reducing visual noise
during active practice. Without this, there is no focus mode. It directly
addresses the ADHD-informed design principle of minimising competing visual
stimuli during the practice session itself (VISION.md §4.3, §7.4).

**Independent Test**: Can be tested by starting any practice session and
verifying that only essential elements are visible during active practice.

**Acceptance Scenarios**:

1. **Given** a musician has started a practice session, **When** the active
   practice screen is displayed, **Then** only the current item name, timer,
   and session controls are visible.
2. **Given** a musician is in active practice, **When** they look at the screen,
   **Then** the main navigation bar is not visible.
3. **Given** a musician is in active practice, **When** there are completed items
   in the session, **Then** the completed items list is not visible by default.

---

### User Story 2 - Visual Progress Timer (Priority: P1)

The timer during active practice uses a visual progress indicator (ring)
that is easy to perceive peripherally, rather than relying solely on digits. The
musician can glance at the screen and quickly gauge elapsed time without reading
numbers.

**Why this priority**: Co-equal with Story 1 — a visual timer is essential to
focus mode. Digit-only timers require focused reading; a circular progress ring can be
perceived peripherally while playing. This supports musicians with time blindness
(VISION.md §7.2) and reduces the need to stop playing to check the clock.

**Independent Test**: Can be tested by starting a session and verifying the timer
displays a visual progress indicator alongside or instead of the digit display.

**Acceptance Scenarios**:

1. **Given** a musician is practising an item, **When** they glance at the timer,
   **Then** a circular progress ring shows elapsed time relative to the
   planned duration.
2. **Given** an item has a planned duration of 5 minutes, **When** 2.5 minutes
   have elapsed, **Then** the progress indicator shows approximately 50% filled.
3. **Given** an item has no planned duration, **When** the timer is running,
   **Then** the digital timer is shown without a progress indicator (since there
   is no target to measure against).

---

### User Story 3 - Reveal Full Controls on Demand (Priority: P2)

A musician needs to access something hidden by focus mode — the completed items
list, navigation, or other controls. They can tap or interact to temporarily
reveal the full UI without leaving the session.

**Why this priority**: Focus mode must never feel like a trap. Musicians need
confidence that controls are accessible when needed. This is the safety valve
that makes the minimal UI acceptable.

**Independent Test**: Can be tested by entering focus mode and verifying that a
single interaction reveals the hidden UI elements.

**Acceptance Scenarios**:

1. **Given** a musician is in focus mode, **When** they tap to reveal controls,
   **Then** the navigation bar and completed items list become visible.
2. **Given** a musician has revealed full controls, **When** they dismiss the
   expanded view or continue practising, **Then** the UI returns to the focused
   minimal state.
3. **Given** a musician has revealed full controls, **When** they use the
   navigation to leave the session, **Then** they are taken to the expected
   destination (with appropriate session state handling).

---

### User Story 4 - Gentle Transition Prompts Between Items (Priority: P2)

When the planned time for an item is reached, the musician receives a gentle
visual cue suggesting they move to the next item. The prompt shows what's coming
next to reduce the cognitive load of "what do I do now?"

**Why this priority**: Transitions are where focus often breaks down, especially
for musicians with executive function challenges. A gentle prompt that previews
the next item reduces the decision burden at transition points.

**Independent Test**: Can be tested by letting an item's planned time elapse and
verifying the transition prompt appears with next-item information.

**Acceptance Scenarios**:

1. **Given** a musician is practising an item with a planned duration, **When**
   the planned time elapses, **Then** a gentle visual indicator appears
   suggesting a transition.
2. **Given** a transition prompt is shown, **When** the musician views it,
   **Then** it includes the name of the next item in the session.
3. **Given** a transition prompt is shown, **When** the musician is on the last
   item, **Then** the prompt suggests finishing the session rather than showing
   a next item.
4. **Given** a transition prompt is shown, **When** the musician ignores it and
   continues practising, **Then** the prompt remains visible but does not block
   or interrupt practice.

---

### Edge Cases

- What happens when a session has only one item? The transition prompt should
  suggest finishing the session when planned time elapses.
- What happens when an item has no planned duration? No progress ring is shown
  for that item (digital timer only), and no time-based transition prompt fires.
- What happens when the rep counter is active? The rep counter remains visible
  in focus mode since it is actively used during practice.
- What happens if the musician resizes the browser or rotates a device during
  focus mode? The layout adapts responsively while maintaining focus mode state.
- What happens during crash recovery? If a session is restored from crash
  recovery, focus mode state should match what a freshly started session would
  show (i.e., focused by default).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The active practice screen MUST hide the main navigation bar during
  practice.
- **FR-002**: The active practice screen MUST hide the completed items list by
  default during practice.
- **FR-003**: The active practice screen MUST display the current item name
  prominently.
- **FR-004**: The active practice screen MUST display a visual progress indicator
  (ring) for the item timer when the item has a planned duration.
- **FR-005**: The active practice screen MUST continue to display the digital
  timer (MM:SS) alongside the progress indicator.
- **FR-006**: The session controls (Next Item / Finish Session, Skip, End Early)
  MUST remain visible and accessible at all times.
- **FR-007**: The rep counter MUST remain visible when active (since it requires
  direct interaction during practice).
- **FR-008**: Users MUST be able to reveal hidden UI elements (navigation,
  completed items) via a dedicated toggle button (e.g., chevron icon).
- **FR-009**: Users MUST be able to return to the focused state after revealing
  the full UI.
- **FR-010**: A gentle visual transition prompt MUST appear when an item's
  planned duration elapses.
- **FR-011**: The transition prompt MUST include the name of the next item (or
  indicate session completion for the last item).
- **FR-012**: The transition prompt MUST NOT block practice — the musician can
  continue playing past the suggested time.
- **FR-013**: Focus mode MUST be the default state when an active session is
  displayed (no opt-in required).
- **FR-014**: The session builder MUST allow an optional per-item duration to be
  set for each setlist entry.
- **FR-015**: Items without a set duration MUST show only the digital timer
  (no progress ring) and MUST NOT trigger time-based transition prompts.

## Design

### Existing Components Used

- **SessionTimer** — restructured to show only essential elements in focused state
- **Button** — for session controls (Next Item, Skip, End Early)
- **TypeBadge** — retained alongside item title for quick identification
- **Rep counter UI** — retained as-is since it requires active interaction

### New Components Needed

- **ProgressRing**: A circular visual progress indicator showing elapsed time as
  a filling arc. Displays the digital timer (MM:SS) in the centre. When no
  planned duration exists, the ring is not shown and only the digital timer
  displays.
- **TransitionPrompt**: A non-blocking visual cue that appears when planned time
  elapses. Shows a gentle message with the next item's name. Dismissible or
  auto-clears when the musician advances.

### Wireframe / Layout Description

**Focused state (default during practice):**

The screen is vertically centred with generous whitespace. From top to bottom:
1. Current item name (large, prominent) with type badge
2. Progress ring with digital timer in centre (or digital timer alone if no planned duration)
3. Rep counter (only when active for current item)
4. Session controls (Next Item / Skip / End Early)
5. A dedicated toggle button (small chevron icon) to reveal hidden elements

**Expanded state (after tapping to reveal):**
- Navigation bar reappears at its usual position
- Completed items list appears below the session controls
- Session intention text reappears
- The same toggle button to return to minimal state

**Transition prompt:**
- Appears as a subtle banner or card between the timer and controls
- Shows: "Up next: [Item Name]" or "Session complete — ready to finish?"
- Non-blocking — does not obscure any active controls

### Responsive Behaviour

- **Mobile**: Progress ring sized to fit comfortably without scrolling. Controls
  stacked vertically. Item name uses a large but single-line-friendly font size.
- **Desktop**: Progress ring can be larger. Controls can sit in a row. More
  whitespace around elements for a calmer visual feel.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: During active practice, no more than 4 visual elements are visible
  on screen by default (item name, timer/progress, rep counter if active,
  controls).
- **SC-002**: A musician can start a session and begin practising without seeing
  the navigation bar or completed items list.
- **SC-003**: Hidden UI elements can be revealed and re-hidden within 2
  interactions (one to show, one to hide).
- **SC-004**: The visual progress indicator accurately represents elapsed time
  as a proportion of planned duration to within 1 second of accuracy.
- **SC-005**: Transition prompts appear within 1 second of the planned duration
  elapsing for an item.

## Clarifications

### Session 2026-02-23

- Q: Per-item planned durations don't exist in the current session builder — how should focus mode handle this? → A: Add a simple optional per-item duration field to the session builder as part of this feature. A richer "session time budgeting" concept (declare total time, auto-allocate across items) is tracked separately as #142.
- Q: Should the progress indicator be a ring or a bar? → A: Circular progress ring. It works as a natural centrepiece for the minimal UI, frames the digital timer in its centre, and is peripherally readable from any angle.
- Q: How does the user reveal hidden UI elements? → A: Dedicated toggle button (small chevron icon). Intentional, discoverable, no accidental triggers — consistent with existing show/hide patterns in the rep counter.

## Assumptions

- Focus mode is always on during active practice — there is no toggle to disable
  it. The "expanded" state is a temporary reveal, not a permanent mode switch.
- The existing session data model does not need changes — focus mode is purely a
  presentation-layer feature.
- A simple optional per-item duration field is added to the session builder as
  part of this feature. Items without a set duration show no progress ring and
  do not trigger time-based transition prompts. A richer time budgeting UX is
  tracked separately (#142).
- The session intention and per-item intentions are useful during building but
  not essential during active practice — they are hidden in focus mode but
  visible in the expanded state.
