# Feature Specification: Basic Goal Setting

**Feature Branch**: `152-goal-setting`
**Created**: 2026-02-24
**Status**: Draft
**Input**: User description: "Issue #60: Basic goal setting — session frequency, practice time, item mastery, and milestone goals. Four goal types with CRUD, progress computed from session data, dedicated /goals page with 5th nav tab, active goals summary card on library home page."

## Clarifications

### Session 2026-02-24

- Q: What fields are editable when updating an existing goal? → A: Title, target value, and deadline are editable. Goal type is immutable after creation.
- Q: Can completed or archived goals be returned to Active? → A: Archived goals can be reactivated to Active. Completed goals are final (completion is a meaningful achievement moment).
- Q: When showing up to 3 goals on the library summary card, which 3 are shown? → A: Most recently created first (newest goals shown).
- Q: What happens when a deadline passes on an active goal? → A: Display as overdue with a visual indicator only. No automated status change — the musician keeps full control.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create a Practice Frequency Goal (Priority: P1)

A musician wants to build a consistent practice habit. They navigate to the Goals page and create a goal to practise a certain number of days per week. After creating the goal, they see a progress indicator showing how many days they've practised this week compared to their target.

**Why this priority**: Consistency is the single strongest predictor of musical progress. A frequency goal provides immediate motivational feedback and is the simplest goal to compute — it only needs a count of distinct practice days.

**Independent Test**: Can be fully tested by creating a frequency goal and completing one practice session. The progress bar should update to reflect the session, and the user should see encouraging progress text.

**Acceptance Scenarios**:

1. **Given** a musician with no goals, **When** they navigate to /goals and tap "Set a Goal", **Then** they see a form with four goal type options.
2. **Given** the goal form is open, **When** they select "Practice Frequency" and enter 5 days/week, **Then** a goal is created with a title like "Practise 5 days per week" and appears on the goals list.
3. **Given** an active frequency goal of 5 days/week and 3 sessions logged this week, **When** they view the goals page, **Then** they see progress showing "3 of 5 days" with a 60% progress bar and positive framing text.
4. **Given** a frequency goal, **When** a new ISO week begins, **Then** progress resets to 0 for the new week.

---

### User Story 2 - Create a Practice Time Goal (Priority: P1)

A musician wants to ensure they're putting in enough total practice time each week. They create a weekly time goal specifying a target number of minutes. Progress accumulates from session durations across the current week.

**Why this priority**: Time-based goals complement frequency goals — a musician might practise every day but only for 5 minutes. Together with frequency, time goals capture the two most fundamental practice metrics.

**Independent Test**: Can be fully tested by creating a time goal and running a timed practice session. The total minutes from the session should add to progress.

**Acceptance Scenarios**:

1. **Given** the goal form is open, **When** they select "Practice Time" and enter 120 minutes/week, **Then** a goal is created with title "Practise 120 minutes per week".
2. **Given** an active time goal of 120 min/week and 85 minutes logged this week, **When** they view the goals page, **Then** they see "85 of 120 min" with approximately 71% progress.
3. **Given** sessions totalling 130 minutes this week against a 120-minute goal, **When** they view progress, **Then** the progress shows 100% (capped) with congratulatory text.

---

### User Story 3 - Create an Item Mastery Goal (Priority: P2)

A musician is working on a specific piece and wants to reach a target mastery score (1-5 scale). They create a mastery goal linked to a specific library item. Progress reflects the latest score recorded for that item during practice sessions.

**Why this priority**: Item mastery goals connect the goal system to the library and scoring features already built, creating a feedback loop between what musicians practise and their goals. Lower priority than frequency/time because it requires item selection UI and depends on the user having library items.

**Independent Test**: Can be fully tested by creating a mastery goal for an existing library item, then completing a session that includes that item with a score. The goal progress should update to reflect the new score.

**Acceptance Scenarios**:

1. **Given** the goal form with "Item Mastery" selected, **When** the user picks a library item and sets target score 4, **Then** a goal is created linked to that item.
2. **Given** a mastery goal for a piece with target score 4, and the latest score is 3, **When** they view the goal, **Then** progress shows "Score 3 of 4" with a 75% bar.
3. **Given** a mastery goal, **When** the linked item receives a score equal to or above the target in a session, **Then** the progress shows 100%.
4. **Given** the goal form with "Item Mastery" selected, **When** the user has no library items, **Then** the form shows a message directing them to add items to their library first.

---

### User Story 4 - Create a Milestone Goal (Priority: P2)

A musician has a non-quantitative goal, such as "Memorise the first movement" or "Perform at the recital". They create a milestone goal with a title, optional description, and optional deadline. Progress is binary — they mark it complete when achieved.

**Why this priority**: Milestones capture goals that don't fit the quantitative types. They support autonomy by letting musicians define success on their own terms. Lower priority because they have no automatic progress computation.

**Independent Test**: Can be fully tested by creating a milestone goal and manually marking it complete. The goal should move to the completed section.

**Acceptance Scenarios**:

1. **Given** the goal form with "Milestone" selected, **When** the user enters a title, description, and optional deadline, **Then** a milestone goal is created showing "In progress" status.
2. **Given** an active milestone goal, **When** the user taps "Mark Complete", **Then** the goal moves to the Completed section with a completion timestamp.
3. **Given** a milestone with a deadline of next Friday, **When** viewing the goal, **Then** the deadline is displayed alongside the goal title.

---

### User Story 5 - View Goal Progress on Library Page (Priority: P2)

A musician opens the app to their library and sees a compact summary of their active goals above the library list. This provides an at-a-glance motivational snapshot without requiring navigation to the goals page.

**Why this priority**: The library page is the most-visited page. Surfacing goal progress here creates a passive reminder of targets and progress, reinforcing the practice loop. Lower priority because it depends on the goals system being built first.

**Independent Test**: Can be fully tested by creating active goals and navigating to the library page. The summary card should appear showing up to 3 active goals with progress indicators.

**Acceptance Scenarios**:

1. **Given** 2 active goals, **When** the user opens the library page, **Then** a compact summary card shows both goals with mini progress bars.
2. **Given** 5 active goals, **When** viewing the library summary card, **Then** only the first 3 are shown with a "View all" link to /goals.
3. **Given** no active goals, **When** viewing the library page, **Then** no goals summary card appears.

---

### User Story 6 - Complete and Archive Goals (Priority: P3)

A musician has achieved a goal or decides it's no longer relevant. They can mark a goal as completed (celebrating the achievement) or archive it (removing it from the active list without deletion). Both completed and archived goals remain visible in a history section.

**Why this priority**: Goal lifecycle management prevents the active goals list from growing unbounded and gives musicians a sense of closure. Lower priority because the app is fully functional without it — goals simply stay active.

**Independent Test**: Can be fully tested by creating a goal, completing it, and verifying it moves to the history section. Separately, creating a goal and archiving it.

**Acceptance Scenarios**:

1. **Given** an active goal, **When** the user taps "Complete", **Then** the goal moves to a Completed section with a completion date.
2. **Given** an active goal, **When** the user taps "Archive", **Then** the goal moves to an Archived section.
3. **Given** completed and archived goals exist, **When** viewing the goals page, **Then** a collapsible "History" section shows them below active goals.
4. **Given** an archived goal, **When** the user taps "Reactivate", **Then** the goal returns to the Active section with progress recalculated.
5. **Given** a completed goal, **When** viewing the goal in history, **Then** no "Reactivate" action is available — completion is final.
6. **Given** an active goal, **When** the user taps "Delete", **Then** they see a confirmation prompt, and after confirming, the goal is permanently removed.

---

### User Story 7 - Navigate to Goals via Tab Bar (Priority: P3)

A musician wants quick access to their goals from anywhere in the app. A 5th tab labelled "Goals" appears in both the desktop header navigation and the mobile bottom tab bar, with a target/bullseye icon.

**Why this priority**: Navigation is essential for discoverability but is a low-risk, low-complexity change. It doesn't deliver value on its own — it only matters once the goals page exists.

**Independent Test**: Can be fully tested by verifying the Goals tab appears on all screen sizes and navigates to /goals. The tab should show an active state when on any /goals/* route.

**Acceptance Scenarios**:

1. **Given** the app is loaded on mobile, **When** the user sees the bottom tab bar, **Then** a 5th "Goals" tab with a bullseye icon is visible.
2. **Given** the user is on the goals page, **When** they look at the navigation, **Then** the Goals tab/link is highlighted as active.
3. **Given** the user is on any non-goals page, **When** they tap the Goals tab, **Then** they navigate to /goals.

---

### Edge Cases

- What happens when a user creates a frequency goal with 7 days/week and the week is already half over? Progress shows days so far (e.g., "2 of 7 days") — no penalty for starting mid-week.
- What happens when the user deletes a library item that has a linked mastery goal? The goal remains but shows "Item no longer in library" and progress cannot advance. The user can archive or delete the goal.
- What happens when no sessions exist for the current week? Frequency and time goals show 0 progress with encouraging text ("Start your first session this week!").
- What happens when a user tries to create a mastery goal for an item that already has one? The system allows it — multiple goals for the same item are permitted (e.g., different target scores).
- What happens when goals are viewed across timezone boundaries? Progress computation uses the user's local week boundary (ISO week, Monday start).
- What happens when a goal's deadline passes? The goal remains active with an "overdue" visual indicator. No automated archival or status change — the user decides whether to extend, complete, or archive.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support four goal types: Session Frequency (days/week), Practice Time (minutes/week), Item Mastery (target score for a library item), and Milestone (custom description with optional deadline).
- **FR-002**: System MUST provide full CRUD operations for goals — create, read (list and detail), update, and delete. When updating, the title, target value, and deadline are editable; the goal type is immutable after creation.
- **FR-003**: System MUST compute goal progress from existing session data in real-time, not from stored progress values. Progress is recalculated whenever goals or sessions change.
- **FR-004**: System MUST validate goal inputs: title max 200 characters, frequency target 1-7 days, time target 1-10080 minutes, mastery target score 1-5, milestone description max 1000 characters.
- **FR-005**: System MUST support three goal statuses: Active, Completed, and Archived. Completing a goal records a completion timestamp and is final (not reversible). Archived goals can be reactivated to Active.
- **FR-006**: System MUST display an optional deadline for any goal type. When a deadline passes on an active goal, it is displayed as overdue with a visual indicator but no automated status change occurs.
- **FR-007**: System MUST provide a dedicated goals page at /goals showing active goals with progress and a collapsible history section for completed/archived goals.
- **FR-008**: System MUST display an active goals summary card on the library home page showing up to 3 active goals with mini progress indicators, ordered by most recently created first.
- **FR-009**: System MUST add a 5th navigation tab for Goals in both desktop header and mobile bottom tab bar.
- **FR-010**: System MUST use positive, process-focused language in all progress displays (e.g., "3 of 5 days — great spacing for retention" rather than "2 days remaining" or "60% complete").
- **FR-011**: System MUST cap progress percentage at 100% when a target is exceeded — progress never shows negative or above-target values.
- **FR-012**: System MUST scope all goal data to the authenticated user. Users can only see and manage their own goals.
- **FR-013**: System MUST auto-generate a default title from goal type and target when creating a goal (editable by the user).

### Key Entities

- **Goal**: The central entity. Has an identity, title, type (one of four kinds), status (active/completed/archived), optional deadline, and creation/update/completion timestamps. Owned by a single user.
- **Goal Kind**: A discriminated union describing the goal's measurable dimension — frequency (target days), time (target minutes), mastery (linked item + target score), or milestone (description text).
- **Goal Progress**: A computed (not stored) projection combining a goal's targets with the user's session history. Includes current value, target value, percentage (0-100), and a human-readable display string.

## Design *(include if feature has UI)*

### Existing Components Used

- `PageHeading` — page title on the goals list page
- `Card` — container for each goal on the list and for the library summary card
- `Button` — create, complete, archive, delete actions
- `BackLink` — back navigation from goal form to goals list
- `TextField` — title input, numeric inputs for targets
- `TextArea` — milestone description
- `Toast` — success/error notifications after goal operations

### New Components Needed

- **GoalProgressBar**: A horizontal progress bar showing percentage filled with current/target text. Supports different colour accents per goal type. Displays positive framing text below the bar.
- **GoalTypeSelector**: A set of selectable cards/tabs representing the four goal types. Selecting one reveals the type-specific input fields.
- **GoalCard**: A card displaying a single goal with its type icon, title, progress bar (for active goals), deadline (if set), and action buttons.
- **ActiveGoalsSummary**: A compact card for the library page showing up to 3 active goals with mini progress indicators and a "View all" link.
- **ItemPicker**: A dropdown/selector allowing the user to choose from their library items. Used in the mastery goal form.

### Wireframe / Layout Description

**Goals List Page (/goals)**:
- `PageHeading` "Goals" with "Set a Goal" CTA button
- Active goals section: vertical list of `GoalCard` components, each showing type icon, title, progress bar, and actions
- If no active goals: empty state with encouraging message and CTA
- History section (collapsible): completed and archived goals without progress bars, showing completion date or archived status

**Goal Form Page (/goals/new)**:
- `BackLink` to /goals
- `PageHeading` "Set a Goal"
- `GoalTypeSelector` — four options: Practice Frequency, Practice Time, Item Mastery, Milestone
- Dynamic form fields based on selected type
- Title field (auto-generated, editable)
- Optional deadline date picker
- Create button

**Library Summary Card**:
- Compact card above the library list
- Shows up to 3 active goals with mini progress bars (thin, inline)
- "View all goals" link at the bottom
- Hidden entirely when no active goals exist

### Responsive Behaviour

- **Mobile**: Goals list is full-width single column. Goal form is full-width. Library summary card spans full width. Bottom tab bar shows 5th Goals tab.
- **Desktop**: Goals list has comfortable padding with max-width constraint. Goal form centered. Library summary card aligns with library list width. Header nav shows Goals link.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can create any of the four goal types and see it on their goals list within 3 seconds of submission.
- **SC-002**: Goal progress updates visibly within 5 seconds of completing a practice session, without manual refresh.
- **SC-003**: Active goals summary appears on the library page for any user with at least one active goal.
- **SC-004**: All progress displays use positive, process-focused language — no shaming, no deficit framing, no countdown-to-failure messaging.
- **SC-005**: Users can complete the full goal lifecycle (create, view progress, complete/archive, view in history) without confusion or errors.
- **SC-006**: Goal data is fully isolated per user — no cross-user data leakage.
- **SC-007**: The Goals navigation tab is discoverable and accessible from all pages on both mobile and desktop layouts.
