# Feature Specification: Teacher Assignment Capture

**Feature Branch**: `269-teacher-capture`  
**Created**: 2026-04-11  
**Status**: Draft  
**Input**: User description: "Quick-capture flow optimised for post-lesson entry. Capture the lesson as a single entity — date, notes, photos. Pure Layer 1: capture what happened, organise later. Item linking deferred to a follow-up feature."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Capture a lesson (Priority: P1)

A musician has just finished a lesson. They open intrada and create a new lesson entry — today's date, a brain dump of everything that happened ("worked on thirds, keep wrist relaxed, new Chopin étude — focus on RH arpeggios very slow"), and a photo of the teacher's handwritten annotations. The whole capture takes under 30 seconds. They put the phone away and head home.

**Why this priority**: This is the true Layer 1 (Capture) value. The notebook replacement. If a musician can capture the raw lesson quickly, nothing is lost — they can organise later. One notes field, no categories, no structure forced at capture time.

**Independent Test**: Can be fully tested by creating a lesson entry with notes and/or a photo, verifying it persists, and viewing it later.

**Acceptance Scenarios**:

1. **Given** the user is on any screen, **When** they tap a "Log Lesson" action, **Then** a streamlined capture form appears with date (defaulting to today), notes, and photo attachment.
2. **Given** the capture form is open, **When** the user writes notes and saves, **Then** a lesson entry is created and visible in the lessons list.
3. **Given** the capture form is open, **When** the user attaches one or more photos, **Then** the photos are stored with the lesson and viewable later.
4. **Given** the user saves a lesson with only a photo (no notes), **Then** the lesson is saved successfully — notes are not required.

---

### User Story 2 - Review and edit a past lesson (Priority: P2)

A musician opens a lesson from last week. They re-read their notes and the photo of the teacher's annotations. They add a line they forgot: "also mentioned practising with metronome at 60 BPM." They save the edit.

**Why this priority**: Capture is often incomplete in the moment. Allowing edits means the lesson entry becomes a living record that can be enriched over time, not a one-shot form.

**Independent Test**: Can be tested by creating a lesson, reopening it, editing the notes, saving, and verifying the changes persist.

**Acceptance Scenarios**:

1. **Given** the user is viewing a lesson, **When** they tap "Edit", **Then** the notes and photos become editable.
2. **Given** the user is editing a lesson, **When** they modify notes or add/remove photos and save, **Then** the changes are persisted.
3. **Given** the user is editing a lesson, **When** they change the date, **Then** the lesson reorders correctly in the list.

---

### User Story 3 - Browse past lessons (Priority: P3)

A musician wants to look back at what their teacher said three weeks ago. They open the lessons list, scan by date, and tap to review the full notes and photos.

**Why this priority**: The value of capture compounds over time — lessons become a searchable history of teacher guidance. This supports Layer 4 (Show) by making the teaching relationship visible.

**Independent Test**: Can be tested by creating several lessons across different dates and browsing through them.

**Acceptance Scenarios**:

1. **Given** the user has multiple lessons, **When** they open the lessons list, **Then** lessons are displayed in reverse chronological order with date and notes preview.
2. **Given** the user is browsing lessons, **When** they tap a lesson, **Then** they see the full detail: date, notes, and photos.
3. **Given** the user has no lessons, **When** they open the lessons list, **Then** an empty state prompts them to log their first lesson.

---

### Edge Cases

- What happens when a lesson has no notes (only photos)? — Allowed; photos alone are a valid capture.
- What happens when a lesson has no photos (only notes)? — Allowed; notes alone are a valid capture.
- What happens when a lesson has neither notes nor photos? — Not allowed; at least one of notes or photos is required.
- What happens when the user deletes a lesson? — Lesson is permanently removed after confirmation. No cascading effects (item linking is a future feature).
- What happens when the user loses connectivity mid-capture? — Form state is preserved locally; lesson syncs when connectivity returns.
- What happens when photos are very large (>10MB each)? — Photos are compressed/resized before storage (e.g., 2048px longest edge).
- What happens when the user creates a lesson for a past date? — Allowed; date is editable for late entries.
- What happens when two lessons have the same date? — Allowed; a musician might have multiple lessons in one day (different teachers, morning/afternoon).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a "Log Lesson" entry point via a prominent action on the Library screen.
- **FR-002**: System MUST capture the following fields per lesson: date (required, defaults to today), notes (free text), and photo attachments (zero or more).
- **FR-003**: At least one of notes or photos MUST be provided to save a lesson.
- **FR-004**: System MUST support attaching multiple photos per lesson from camera or photo library.
- **FR-005**: System MUST display attached photos in the lesson detail view, tappable to view full-size.
- **FR-006**: System MUST allow removing or adding photos when editing a lesson.
- **FR-007**: System MUST display lessons in a chronological list with date and notes preview.
- **FR-008**: System MUST preserve form state if the user navigates away mid-capture.
- **FR-009**: System MUST allow editing a lesson after creation (update date, notes, add/remove photos).
- **FR-010**: System MUST allow deleting a lesson with a confirmation prompt.

### Key Entities

- **Lesson**: A new first-class entity representing a single teaching session. Has a date, notes (free text), and zero or more photo attachments. Owned by a user. No relationship to library items in this iteration.

## Design *(include if feature has UI)*

### Existing Components Used

- **CardView** — container for lesson detail sections
- **ButtonView** — primary action (save), secondary action (edit, delete)
- **EmptyStateView** — when no lessons exist, prompt to log first lesson
- **Form fields** — text area, date picker

### New Components Needed

- **LessonCaptureForm**: Minimal capture form — date picker (defaulting to today), notes text area (the hero field, auto-focused), photo attachment area. Optimised for speed.
- **LessonCard**: List item for the lessons list — shows date and notes preview (truncated). Photo indicator icon when photos are attached.
- **LessonDetailView**: Full view of a captured lesson — date, notes, photo gallery, and edit/delete actions.
- **PhotoGallery**: Horizontal scrollable strip of photo thumbnails, tappable to view full-size. Supports add/remove in edit mode.

### Wireframe / Layout Description

**Lesson Capture Form (full-screen on mobile, modal on desktop)**:
- Date field (today pre-filled, tappable to change)
- Notes field (prominent, auto-focused, multi-line — the hero field)
- Photo attachment area (camera/gallery buttons, thumbnail strip of attached photos)
- Save button

**Lessons List**:
- Reverse chronological list of lesson cards
- Each card: date, notes preview (truncated), photo indicator
- Empty state: "Log your first lesson" prompt

**Lesson Detail View**:
- Date heading
- Notes section (full text)
- Photo gallery (horizontal thumbnail strip, tappable to view full-size)
- Edit and delete actions

### Responsive Behaviour

- **Mobile (iOS primary)**: Full-screen capture form. Native camera/photo picker. Lessons accessible from a prominent entry point. Photo gallery as horizontal scroll.
- **Desktop (web)**: Modal for capture. Lessons list as a dedicated page. Photo gallery as a horizontal strip. Same content and flow.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can capture a lesson (date + notes or photo) in under 30 seconds.
- **SC-002**: Users can find and review any past lesson within 3 taps from the home screen.
- **SC-003**: Zero data loss — all captured lessons and photos persist across app restarts and sync to the server.
- **SC-004**: Users can edit a lesson and verify changes persist immediately.

## Assumptions

- Lessons are a lightweight entity — the goal is speed of capture, not comprehensive structure. A lesson with only a date and a photo is valid.
- Photo storage uses an external object store (not SQLite). The specific mechanism is an implementation detail.
- Lessons do not replace sessions. A lesson is a record of *teaching received*; a session is a record of *practice done*. They are complementary but separate concepts.
- The lessons list is a new navigational concept. Its placement (tab, section, or entry point) is a design decision to be refined, but it must be reachable without navigating through the library.
- Item linking (deriving library items from lessons, inline quick-add) is explicitly deferred to a follow-up feature. This feature is pure capture.

## Future Work (out of scope)

- **Inline quick-add**: Type item names within a lesson to create minimal library items, with the option to expand and fill in details later. Separate issue.
- **Lesson–item linking**: Link existing library items to a lesson. Depends on inline quick-add or a dedicated linking flow.
- **Lesson link on item detail**: Show originating lesson on a library item's detail view.
