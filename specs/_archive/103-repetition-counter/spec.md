# Feature Specification: Repetition Counter

**Feature Branch**: `103-repetition-counter`
**Created**: 2026-02-21
**Status**: Draft
**Input**: User description: "Repetition counter (#49) — optional per-item counter tracking consecutive correct repetitions toward a configurable target during active practice sessions."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Use Counter During Active Practice (Priority: P1)

During an active practice session, a musician enables the repetition counter on the current item and uses "got it" / "missed" buttons to track consecutive correct repetitions. When the counter reaches the target, the app signals achievement and prompts transition to the next item.

**Why this priority**: This is the core interaction — without it, nothing else matters. A musician can practise with the counter and get an objective "done" signal, which is the primary value.

**Independent Test**: Can be fully tested by starting a session with one item, enabling the counter, tapping "got it" until the target is reached, and verifying the achievement prompt appears.

**Acceptance Scenarios**:

1. **Given** a musician is on an item during active practice, **When** they enable the repetition counter, **Then** a counter appears showing 0 of the target number (default 5), with "got it" and "missed" buttons.
2. **Given** the counter is active and shows 3/5, **When** the musician taps "got it", **Then** the counter increments to 4/5.
3. **Given** the counter is active and shows 3/5, **When** the musician taps "missed", **Then** the counter decrements to 2/5.
4. **Given** the counter is active and shows 0/5, **When** the musician taps "missed", **Then** the counter remains at 0/5 (never goes below zero).
5. **Given** the counter is active and shows 4/5, **When** the musician taps "got it", **Then** the counter reaches 5/5, a visual achievement indicator appears, and the app prompts transition to the next item.
6. **Given** the target has been reached (5/5), **When** the musician dismisses the achievement prompt, **Then** the counter freezes at the target, the "got it" / "missed" buttons are hidden, and the timer continues — the musician can keep practising freely without the counter.
7. **Given** the counter is not enabled, **When** the musician is practising an item, **Then** the practice experience is identical to today — timer and controls only, no counter visible.

---

### User Story 2 - Configure Target Per Item (Priority: P2)

A musician configures the repetition target for individual items during the building phase, so different items can have different difficulty thresholds. The target persists across sessions via the setlist entry.

**Why this priority**: Configuring targets makes the counter genuinely useful for varied repertoire — a simple scale might need 3 reps, while a tricky passage needs 7. Without this, the counter works but is less flexible.

**Independent Test**: Can be tested by adding two items to a setlist, setting different targets (e.g. 3 and 7), starting the session, and verifying each item shows its correct target.

**Acceptance Scenarios**:

1. **Given** a musician is in the building phase, **When** they view a setlist entry, **Then** they see a small "Add rep target" link. Tapping it reveals a target stepper (defaulting to 5).
2. **Given** an entry has a default target of 5, **When** the musician changes it to 7, **Then** the counter during active practice for that item shows 0/7.
3. **Given** a musician sets a target below the minimum (3), **When** they attempt to save, **Then** the target is constrained to the minimum of 3.
4. **Given** a musician sets a target above the maximum (10), **When** they attempt to save, **Then** the target is constrained to the maximum of 10.

---

### User Story 3 - View Rep Count in Summary and History (Priority: P3)

After completing a session, the musician can see the final rep count and whether the target was achieved in the session summary and in session history. This provides a record of quality-focused practice.

**Why this priority**: Visibility into past rep counts connects to the Track pillar. Without it, the counter is useful in-the-moment but doesn't contribute to longer-term insight. Still valuable as a standalone feature.

**Independent Test**: Can be tested by completing a session with the counter, saving it, and verifying rep count data appears in both the summary view and session history.

**Acceptance Scenarios**:

1. **Given** a musician finishes an item with the counter showing 5/5 (target achieved), **When** they view the session summary, **Then** the entry shows the target was achieved with the final count.
2. **Given** a musician skips an item with the counter showing 2/5, **When** they view the session summary, **Then** the entry shows the count reached (2) and the target (5) — not achieved.
3. **Given** a musician completed a session with rep count data yesterday, **When** they view session history, **Then** the entry shows rep count data alongside duration, score, and notes.
4. **Given** a musician completed a session without using the counter, **When** they view session history, **Then** no rep count information is shown for that entry.

---

### User Story 4 - Persist Rep Target and Count (Priority: P4)

Rep targets set during the building phase and final counts recorded during active practice are saved to the database when the session is saved. Existing sessions without rep data continue to work without issues.

**Why this priority**: Persistence is essential for US3 and for future analytics, but is a backend concern that supports the other stories rather than delivering direct user value on its own.

**Independent Test**: Can be tested by saving a session with rep data, refreshing the page, and verifying the data is present when viewing the session.

**Acceptance Scenarios**:

1. **Given** a session is saved with rep count data, **When** the session is loaded from the API, **Then** all rep targets and counts are present.
2. **Given** an existing session was saved before this feature existed, **When** it is loaded, **Then** it displays correctly with no rep count information (backward compatibility).
3. **Given** a session with rep data is loaded, **When** the API returns the data, **Then** rep targets and counts are associated with the correct setlist entries.

---

### Edge Cases

- What happens when a musician disables the counter mid-item? The count resets and the counter UI disappears. The entry has no rep data recorded.
- What happens when a musician enables the counter after already practising an item for a while? The counter starts at 0 regardless of time already spent — it tracks repetitions, not time.
- What happens during crash recovery? If the session is recovered from localStorage, the counter state (current count, target, enabled flag) is restored along with the rest of the active session.
- What happens when a musician reaches the target but doesn't want to move on? The achievement indicator shows, but transition is a prompt, not automatic. The musician dismisses the prompt, the counter freezes at the target with buttons hidden, and the timer continues for free practice.
- What happens when loading a routine that was saved before rep targets existed? Entries from the routine have no rep target set — they use the default (5) if the musician enables the counter.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide an optional repetition counter during active practice, toggled on/off per item.
- **FR-002**: The counter MUST track consecutive correct repetitions, incrementing on "got it" and decrementing on "missed".
- **FR-003**: The counter MUST never decrement below zero.
- **FR-004**: When the counter reaches the configured target, the system MUST display a visual achievement indicator and prompt (not force) transition to the next item.
- **FR-005**: The default repetition target MUST be 5, configurable per setlist entry in the range 3–10 inclusive.
- **FR-006**: Rep targets MUST be configurable during the building phase per entry via an opt-in toggle. The target stepper is hidden by default and revealed by an "Add rep target" action per entry.
- **FR-007**: The counter state (current count, target, enabled) MUST be included in crash recovery data so it survives tab closure.
- **FR-008**: The final rep count and target MUST be persisted when the session is saved.
- **FR-009**: Rep count data MUST be visible in the session summary alongside existing entry data (duration, score, notes).
- **FR-010**: Rep count data MUST be visible in session history for past sessions.
- **FR-011**: Sessions saved before this feature MUST continue to load and display correctly (backward compatibility).
- **FR-012**: Disabling the counter mid-item MUST reset the count and remove rep data from that entry.
- **FR-015**: When a musician skips an item with the counter active, the partial rep count and target MUST be saved with the entry (e.g. 2/5, not achieved).
- **FR-013**: The achievement prompt after reaching the target MUST be dismissible — the musician can choose to continue practising or move to the next item.
- **FR-014**: After the target is reached and the prompt dismissed, the counter MUST freeze at the target value with "got it" / "missed" buttons hidden. The timer continues normally.

### Key Entities

- **RepetitionState**: Per-entry state during active practice — current count, target, enabled flag. Transient during the session, not persisted directly (only the final count and target are saved).
- **SetlistEntry** (extended): Gains optional rep target (set during building), final rep count (set during/after active practice), and target-achieved flag.

## Design *(include if feature has UI)*

### Existing Components Used

- **Button** — "Got it" and "Missed" tap targets, and the toggle to enable/disable the counter
- **Card** — Container for the counter display during active practice
- **TypeBadge** — No change, but displayed alongside rep data in summary

### New Components Needed

- **RepCounter**: Displays the current count vs target (e.g. "3 / 5"), the "got it" and "missed" buttons, and the achievement state. Needs to be prominent but not intrusive during active practice — positioned below the timer area.
- **RepTargetInput**: A small numeric stepper (3–10) for configuring the target per entry during the building phase. Inline with each setlist entry row.
- **RepAchievement**: A brief celebratory indicator when the target is reached — communicates "done" without being disruptive. Includes a prompt to move to the next item.

### Wireframe / Layout Description

**Active Practice (counter enabled):**
- Below the existing timer display and above the navigation buttons
- Shows: counter value prominently (e.g. "3 / 5" in large text)
- Two buttons side by side: "Got it" (positive/accent) and "Missed" (neutral/subdued)
- Small toggle or link to disable the counter
- When target reached: counter area transforms to show achievement state with "Next Item" prompt

**Building Phase:**
- Each setlist entry row shows a small "Add rep target" link below the existing intention input
- Tapping it reveals a numeric stepper (3–10, default 5) and a remove option to hide it again
- No rep target UI visible by default — keeps the builder clean for musicians who don't use the counter

**Summary & History:**
- Rep data shown as a small badge or line below the entry: "Reps: 5/5 ✓" or "Reps: 2/5"
- Positioned near the existing score and duration display

### Responsive Behaviour

- **Mobile**: "Got it" and "Missed" buttons should be large enough for easy tapping during practice (minimum 44px touch targets). Counter display centred above buttons.
- **Desktop**: Same layout, buttons can be slightly smaller. Counter display can be inline with the timer area.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Musicians can enable the repetition counter and reach a target in under 30 seconds of interaction overhead (enabling + tapping through a sequence).
- **SC-002**: The counter adds zero disruption to musicians who don't use it — no additional UI visible, no behaviour changes.
- **SC-003**: 100% backward compatibility — all existing sessions load and display correctly without rep data.
- **SC-004**: Counter state survives crash recovery — a session recovered after tab closure retains the counter's current count and target.
- **SC-005**: Rep count data is visible in session summary and history within the same view as existing entry data (no navigation required to see it).

## Clarifications

### Session 2026-02-21

- Q: What happens to the counter after the target is reached and the musician dismisses the prompt? → A: Counter freezes at target, "got it" / "missed" buttons hidden, timer continues for free practice.
- Q: What happens to the rep count when the musician taps "Skip"? → A: Partial count and target are saved (e.g. 2/5, not achieved). Summary shows the data.
- Q: In the building phase, is the rep target stepper always visible or toggled? → A: Toggle per entry. Small "Add rep target" link expands the stepper. No stepper shown by default.

## Assumptions

- The "got it" / "missed" interaction is deliberately simple — no partial credit, no "almost" — because the research basis (overlearning, 85% rule) supports binary correct/incorrect for repetition counting.
- The default target of 5 is based on the VISION.md research analysis balancing overlearning benefit against the 85% accuracy principle.
- The target range of 3–10 is based on the VISION.md analysis: below 3 doesn't confirm consistency; above 10 risks frustration at productive difficulty levels.
- Achievement prompt is non-blocking (musician chooses to advance) because auto-advancing could interrupt flow if the musician wants one more repetition.
- The counter is per-entry, not per-session — different items in the same session can have different targets and independent counters.
- Disabling the counter mid-item is treated as "I don't want to use this" — the count resets rather than pausing, keeping the data clean.
