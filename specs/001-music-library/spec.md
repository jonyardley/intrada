# Feature Specification: Music Library

**Feature Branch**: `001-music-library`
**Created**: 2026-02-08
**Status**: Draft
**Input**: User description: "A music library for managing pieces and exercises. The Rust core should contain all the shared business logic, designed to be consumed by multiple frontends (CLI first, later iOS/web). The library lets musicians build a personal catalogue of pieces they're working on (with metadata like composer, title, difficulty, key, tempo) and exercises (scales, arpeggios, technique drills etc). Items can be tagged, searched, and filtered. This is the foundational domain model that will later be used to track progress in practice sessions."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add a Piece to the Library (Priority: P1)

A musician wants to add a new piece to their personal library. They provide the title and composer as a minimum, and can optionally include additional metadata such as musical key, tempo marking, and any relevant notes. The piece is saved and immediately available in their library for future reference.

**Why this priority**: Adding pieces is the most fundamental action — without it, the library has no content. This is the core building block everything else depends on.

**Independent Test**: Can be fully tested by adding a piece with various combinations of metadata and verifying it persists and can be retrieved.

**Acceptance Scenarios**:

1. **Given** an empty library, **When** a musician adds a piece with title "Clair de Lune" and composer "Debussy", **Then** the piece is saved and appears in the library with those details.
2. **Given** an empty library, **When** a musician adds a piece with title, composer, key "Db Major", and tempo "Andante", **Then** all metadata is stored and retrievable.
3. **Given** a library with existing pieces, **When** a musician adds another piece, **Then** it is saved without affecting existing entries.
4. **Given** a musician adding a piece, **When** they omit the title, **Then** the system rejects the entry with a clear error message.

---

### User Story 2 - Add an Exercise to the Library (Priority: P1)

A musician wants to add an exercise to their library — such as a scale, arpeggio, or technique drill. Exercises have their own metadata: a title, optional composer (e.g. Hanon, Czerny), category (e.g. scales, arpeggios, technique, sight-reading), optional key, optional tempo, and descriptive notes.

**Why this priority**: Exercises are equally fundamental to pieces. Together they form the two core entity types in the library.

**Independent Test**: Can be fully tested by adding exercises of different categories and verifying they persist with correct metadata.

**Acceptance Scenarios**:

1. **Given** an empty library, **When** a musician adds an exercise with title "C Major Scale" and category "Scales", **Then** the exercise is saved and appears in the library.
2. **Given** a library, **When** a musician adds an exercise with title, category "Arpeggios", key "G Minor", and tempo "120 BPM", **Then** all metadata is stored and retrievable.
3. **Given** a musician adding an exercise, **When** they omit the title, **Then** the system rejects the entry with a clear error message.

---

### User Story 3 - Browse and View Library Contents (Priority: P1)

A musician wants to see everything in their library. They can list all items (pieces and exercises), and view the full details of any individual item. The listing provides enough information at a glance (title, type, composer/category) to identify items quickly.

**Why this priority**: Being able to see what's in the library is essential for it to be useful. Without browsing, the library is a write-only store.

**Independent Test**: Can be fully tested by populating a library with several items and verifying the list and detail views show correct information.

**Acceptance Scenarios**:

1. **Given** a library with 5 pieces and 3 exercises, **When** a musician lists all items, **Then** all 8 items are shown with their type, title, and key identifying information.
2. **Given** a library with items, **When** a musician views the details of a specific piece, **Then** all stored metadata for that piece is displayed.
3. **Given** an empty library, **When** a musician lists all items, **Then** they see a clear message indicating the library is empty.

---

### User Story 4 - Tag Library Items (Priority: P2)

A musician wants to organise their library by applying tags to pieces and exercises. Tags are freeform labels (e.g. "exam prep", "warm-up", "romantic era", "left hand") that help group and categorise items across both pieces and exercises.

**Why this priority**: Tags add meaningful organisation on top of the basic library. Important for usability but not required for the library to function.

**Independent Test**: Can be fully tested by adding tags to items and verifying they are stored, displayed, and can be removed.

**Acceptance Scenarios**:

1. **Given** a piece in the library, **When** a musician adds the tag "exam prep", **Then** the tag is associated with that piece and visible in its details.
2. **Given** an item with tags, **When** a musician removes a tag, **Then** the tag is no longer associated with that item.
3. **Given** a musician adding a tag, **When** they apply the same tag that already exists on the item, **Then** the system does not create a duplicate.
4. **Given** a musician creating an item, **When** they provide tags at creation time, **Then** the item is saved with those tags attached.

---

### User Story 5 - Search and Filter the Library (Priority: P2)

A musician wants to find specific items in their library quickly. They can search by text (matching against titles, composers, notes) and filter by type (piece/exercise), key, category (for exercises), and tags.

**Why this priority**: Search and filtering become essential as the library grows. Less critical for a small initial collection but key for ongoing usability.

**Independent Test**: Can be fully tested by populating a library with diverse items and verifying that searches and filters return correct results.

**Acceptance Scenarios**:

1. **Given** a library with pieces by various composers, **When** a musician searches for "Debussy", **Then** only pieces by Debussy are returned.
2. **Given** a library with pieces and exercises, **When** a musician filters by type "exercise", **Then** only exercises are shown.
3. **Given** a library with tagged items, **When** a musician filters by tag "warm-up", **Then** only items with that tag are returned.
4. **Given** a library, **When** a musician searches for a term that matches nothing, **Then** they see a clear "no results" message.
5. **Given** a library, **When** a musician combines a text search with a filter (e.g. search "scale" filtered to type "exercise"), **Then** results match both criteria.

---

### User Story 6 - Edit and Delete Library Items (Priority: P2)

A musician wants to update the details of existing items (fix a typo, add notes, change metadata) and remove items they no longer need.

**Why this priority**: Editing and deleting complete the CRUD lifecycle. Important for a usable library but not the first thing needed.

**Independent Test**: Can be fully tested by modifying item metadata and verifying changes persist, and by deleting items and verifying they are removed.

**Acceptance Scenarios**:

1. **Given** a piece in the library, **When** a musician updates its title, **Then** the new title is shown in listings and detail views.
2. **Given** an item in the library, **When** a musician deletes it, **Then** it is permanently removed from the library.
3. **Given** a musician deleting an item, **When** they confirm the deletion, **Then** the item and its associated tags are removed.

---

### Edge Cases

- What happens when a musician tries to add a piece with extremely long field values (e.g. a 10,000-character title)?
- How does the system handle special characters and Unicode in titles, composers, and notes (e.g. accented characters like "Dvořák", "Ménuet")?
- What happens when a musician searches with an empty query?
- How does the system behave when the library contains a very large number of items (e.g. 10,000+)?
- What happens when a musician tries to delete an item that has already been deleted?
- How does the system handle concurrent modifications if multiple frontends access the same library?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow musicians to add pieces with a mandatory title and composer, and optional metadata (key, tempo, notes)
- **FR-002**: System MUST allow musicians to add exercises with a mandatory title, and optional metadata (composer, category, key, tempo, notes)
- **FR-003**: System MUST persist all library items so they survive application restarts
- **FR-004**: System MUST allow musicians to list all items in their library with summary information
- **FR-005**: System MUST allow musicians to view the full details of any individual item
- **FR-006**: System MUST allow musicians to update any metadata field on an existing item
- **FR-007**: System MUST allow musicians to delete items from their library permanently
- **FR-008**: System MUST support freeform text tags that can be applied to both pieces and exercises
- **FR-009**: System MUST support text search across titles, composers (on both pieces and exercises), categories, and notes
- **FR-010**: System MUST support filtering by item type, key, category, and tags
- **FR-011**: System MUST support combining text search with filters
- **FR-012**: System MUST validate required fields and provide clear error messages when validation fails
- **FR-013**: System MUST handle Unicode text correctly in all fields
- **FR-014**: System MUST enforce reasonable field length limits and reject values that exceed them
- **FR-015**: The core business logic MUST be implemented as a shared library independent of any specific frontend or persistence mechanism

### Key Entities

- **Piece**: A musical composition a musician is working on. Key attributes: title (required), composer (required), key, tempo marking, notes, tags. Represents repertoire the musician is learning or maintaining.
- **Exercise**: A practice drill or technique study. Key attributes: title (required), composer (optional), category (freeform, e.g. scales, arpeggios, technique, sight-reading), key, tempo, notes, tags. Represents structured practice activities.
- **Tag**: A freeform label applied to pieces or exercises for organisation. A single tag can be applied to multiple items, and an item can have multiple tags.

### Assumptions

- This is a single-user, local-first application. Multi-user and sync features are out of scope for this feature.
- Persistence details (file format, database) are implementation decisions, not specification concerns. The core library will define a storage trait/interface.
- Tempo can be stored as either a textual marking (e.g. "Andante", "Allegro") or a numeric BPM value, or both. The representation will be flexible.
- Category for exercises is freeform text. Suggested defaults (Scales, Arpeggios, Technique, Sight-Reading, Etudes, Rhythm) are offered in the UI but not enforced.
- Items are identified by a unique, system-generated identifier — not by title alone. Duplicate titles/composers/names are allowed; no uniqueness constraints beyond the system ID.

## Clarifications

### Session 2026-02-08

- Q: What difficulty level system should be used? → A: Deferred — difficulty level removed from this feature scope entirely. To be revisited as a separate feature later.
- Q: Are duplicate items (same title+composer or same name) allowed? → A: Yes — no uniqueness constraints. Musicians may have different editions or arrangements of the same piece. Each item has a unique system-generated ID.
- Q: Should exercise categories be a fixed list or freeform? → A: Freeform — musicians can type any category. Suggested defaults (Scales, Arpeggios, Technique, Sight-Reading, Etudes, Rhythm) are offered in the UI but not enforced.
- Q: Should pieces and exercises use the same field name for their primary label? → A: Yes — both use "title" for consistency across the data model and API.
- Q: Should exercises have an optional composer field? → A: Yes — exercises can optionally have a composer (e.g. Hanon, Czerny, Kreutzer).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Musicians can add a new piece or exercise to their library in under 30 seconds
- **SC-002**: Musicians can find any item in a library of 1,000+ items in under 5 seconds using search or filters
- **SC-003**: All library operations (add, view, edit, delete, search) complete without perceptible delay for libraries up to 10,000 items
- **SC-004**: 100% of required field validations produce clear, actionable error messages
- **SC-005**: The core library can be consumed by a CLI frontend with zero changes to business logic
- **SC-006**: All library data persists correctly across application restarts with no data loss
