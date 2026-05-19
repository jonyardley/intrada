# Goals

> Evolve the unshipped lesson vertical slice into a Goals feature — the
> spine of the Plan pillar.

## Problem

The app handles mid-practice well (sessions, scoring, focus mode) but
lacks a "what should I work on and why" layer. There's no way to capture
intent — whether from a lesson, a masterclass, self-reflection, or an
upcoming performance — and turn it into structured practice planning.

A full lesson + photos vertical slice exists in core + API + DB (~610 LOC)
but has zero UI. Rather than building a UI for the narrow "lesson" concept,
we evolve it into Goals — a broader, forward-looking feature that serves
all musicians, not just those with a teacher.

## Design decisions

1. **Goals replace lessons.** A goal is anything you're working towards —
   "prep for next Tuesday's lesson" or "October recital." The lesson model
   is the starting point, not a greenfield rewrite.

2. **One surface, not two.** Quick capture and full goal planning are the
   same flow. You stop at whatever level of detail you have time for.
   A quick capture is just a goal where you only filled in the basics.

3. **Goals are the app's front door.** A new Goals tab sits first in the
   tab bar (Goals · Library · Practice · Analytics · Account) and is the
   default landing screen after sign-in.

4. **Items link to goals organisationally.** Users can link existing library
   items to a goal (or create new ones). The link is a reference — no
   session integration yet. That's the Plan → Practice bridge for a later
   phase.

5. **Tasks are deferred.** Subtasks under goals (practise bars 32–48,
   listen to recording, transcribe bridge) are a natural next step but
   not in scope for v1.

## Data model

### Goal (evolves from Lesson)

| Field          | Type              | Notes                                          |
|----------------|-------------------|-------------------------------------------------|
| `id`           | String (ULID)     | Existing                                        |
| `title`        | Option\<String\>  | New — short label. Optional for quick captures. |
| `notes`        | Option\<String\>  | Existing — free text, max 10k chars             |
| `deadline`     | Option\<String\>  | New — YYYY-MM-DD, optional                      |
| `status`       | GoalStatus        | New — Active (default) or Completed             |
| `date`         | String            | Existing — capture date (YYYY-MM-DD)            |
| `completed_at` | Option\<DateTime\>| New — when status flipped to Completed          |
| `items`        | Vec\<GoalItem\>   | New — linked library items                      |
| `photos`       | Vec\<GoalPhoto\>  | Existing (renamed from LessonPhoto)             |
| `created_at`   | DateTime          | Existing                                        |
| `updated_at`   | DateTime          | Existing                                        |

**GoalStatus** enum: `Active`, `Completed`.

**GoalItem** (lightweight reference, same pattern as set entries):
- `item_id: String`
- `item_title: String` (denormalised for display)
- `item_type: ItemKind` (Piece or Exercise)

**GoalPhoto** (renamed from LessonPhoto, unchanged shape):
- `id: String`
- `url: String`
- `created_at: DateTime`

### Validation

- `title`: max 200 characters (when present)
- `notes`: max 10,000 characters (existing)
- `deadline`: YYYY-MM-DD format, no past-date restriction (a goal for
  last week's lesson is valid)
- `date`: YYYY-MM-DD format, not future-dated (existing)
- Photos: JPEG/PNG only, max 5 MB each (existing)

## API

### Routes

| Method   | Path                                | Purpose                        |
|----------|-------------------------------------|--------------------------------|
| `GET`    | `/api/goals`                        | List goals (filter by status)  |
| `POST`   | `/api/goals`                        | Create goal                    |
| `GET`    | `/api/goals/{id}`                   | Get goal with photos and items |
| `PUT`    | `/api/goals/{id}`                   | Update goal                    |
| `DELETE` | `/api/goals/{id}`                   | Delete goal + photos + items   |
| `POST`   | `/api/goals/{id}/photos`            | Upload photo (existing R2)     |
| `DELETE` | `/api/goals/{id}/photos/{photo_id}` | Delete photo                   |
| `POST`   | `/api/goals/{id}/items`             | Link library item              |
| `DELETE` | `/api/goals/{id}/items/{item_id}`   | Unlink library item            |

### List filtering

`GET /api/goals?status=active` (default), `?status=completed`, `?status=all`.

Active goals sorted by deadline ascending (soonest first, no-deadline
last), then by creation date descending. Completed goals sorted by
`completed_at` descending.

### Create request (minimal)

```json
{ "date": "2026-05-15", "title": "...", "notes": "...", "deadline": "2026-05-20" }
```

Only `date` is required (shell auto-sets to today). At least one of
`title`, `notes`, or a photo (uploaded after creation) should be present
for a meaningful goal, but the API doesn't enforce this — the UI guides
it.

### Link item request

```json
{ "item_id": "01J...", "item_title": "Bach Prelude in C", "item_type": "piece" }
```

## Core (Crux) events

### Events in

| Event                                | Purpose                              |
|--------------------------------------|--------------------------------------|
| `Goal(GoalEvent::FetchGoals)`        | Load active goals                    |
| `Goal(GoalEvent::FetchGoal { id })`  | Load single goal with photos + items |
| `Goal(GoalEvent::Add(CreateGoal))`   | Quick capture or full create         |
| `Goal(GoalEvent::Update { id, input })` | Edit any field including status   |
| `Goal(GoalEvent::Complete { id })`   | Mark done (sets status + completed_at) |
| `Goal(GoalEvent::Delete { id })`     | Remove goal                          |
| `Goal(GoalEvent::LinkItem { goal_id, item })` | Add item reference          |
| `Goal(GoalEvent::UnlinkItem { goal_id, item_id })` | Remove item reference  |

### Events back

| Event                        | Purpose                |
|------------------------------|------------------------|
| `GoalsLoaded { goals }`     | Replace model.goals    |
| `GoalLoaded { goal }`       | Set model.current_goal |
| `RefetchGoals`               | Trigger fresh list     |

### Model state

Replaces `lessons: Vec<Lesson>` and `current_lesson: Option<Lesson>`:
- `goals: Vec<Goal>`
- `current_goal: Option<Goal>`

### ViewModel

`GoalView` replaces `LessonView`:
- All existing LessonView fields (renamed)
- `title: Option<String>`
- `deadline: Option<String>`
- `status: GoalStatus`
- `completed_at: Option<String>`
- `is_overdue: bool` (computed: has deadline in past, still active)
- `items: Vec<GoalItemView>`

## Database

### New tables

**goals** (replaces `lessons`):
```sql
CREATE TABLE goals (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL DEFAULT '',
    title TEXT,
    date TEXT NOT NULL,
    notes TEXT,
    deadline TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    completed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_goals_user_status ON goals(user_id, status, deadline);
```

**goal_photos** (replaces `lesson_photos`):
```sql
CREATE TABLE goal_photos (
    id TEXT PRIMARY KEY,
    goal_id TEXT NOT NULL,
    user_id TEXT NOT NULL DEFAULT '',
    storage_key TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_goal_photos_goal ON goal_photos(goal_id);
```

**goal_items** (new):
```sql
CREATE TABLE goal_items (
    goal_id TEXT NOT NULL,
    item_id TEXT NOT NULL,
    item_title TEXT NOT NULL,
    item_type TEXT NOT NULL,
    user_id TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    PRIMARY KEY (goal_id, item_id)
);
CREATE INDEX idx_goal_items_goal ON goal_items(goal_id);
```

### Migration plan

No user data exists in the lessons tables (no UI was ever shipped), so
this is a clean break:
- Drop `lesson_photos` table
- Drop `lessons` table
- Create `goals`, `goal_photos`, `goal_items` tables

## UI

### Navigation

Tab bar order: **Goals** · Library · Practice · Analytics · Account.
Goals is the default landing screen after sign-in.

Icon: target/bullseye style (Heroicons outline/solid pair).

### Screens

**Goal List** (landing):
- Active/Completed toggle tabs at top
- Goal cards showing: title (or notes preview if no title), deadline
  badge (relative: "Due Tue", or absolute for distant dates), photo
  count indicator
- Goals without titles display notes in italics as the primary text
- FAB (+) button for new goal
- Empty state for new users

**Quick Capture / Create Goal** (same form):
- Title (optional) — text input
- Notes — multiline text area
- Deadline (optional) — date picker
- Photos (optional) — camera/gallery picker
- "Save Goal" button
- Minimal friction: notes or a photo is enough to save

**Goal Detail**:
- Status badge + deadline
- Notes (full text)
- Photos section with add capability
- Library Items section — browse/search to link, or create new item
  inline. Same picker pattern as the session builder.
- "Mark Complete" action button
- Edit button
- Captured date footer
- Space reserved for Tasks section in future phases

### Design system

Uses existing primitives: `Card`, `GroupedList`, `TypeBadge`,
`BottomSheet` (for item picker), `Button`, `EmptyState`. New goal cards
follow the `AccentRow` pattern. Active/Completed toggle uses
`LibraryTypeTabs` pattern.

## Right to erasure

- **Goal deletion** (`DELETE /api/goals/{id}`): delete R2 photo objects,
  then `goal_photos` rows, then `goal_items` rows, then `goals` row.
- **Account deletion** (`DELETE /api/account`): delete all `goal_items`,
  `goal_photos`, `goals` rows for the user. R2 cleanup via existing
  `delete_user_photos` (operates on user-level prefix, table-name
  agnostic).
- **UI copy**: update account deletion notice from "Your lessons and
  lesson photos" to "Your goals and goal photos."
- **Core model reset**: `Model::goals` and `Model::current_goal` cleared
  on sign-out.

## Phasing

### Phase A — Core + API + DB
Rename lesson → goal across core and API. Add new fields (title,
deadline, status, completed_at). Create new DB tables and drop old ones.
Add goal_items table and endpoints. Tests for all new endpoints.

### Phase B — Web UI
Goals tab in tab bar (first position). Goal list screen with
Active/Completed toggle. Quick capture / create form. Goal detail screen
with item linking (picker). Update account deletion copy.

### Phase C — Polish
Overdue indicators (red badge for past-deadline active goals). Empty
states for new users. Photo management UX. Haptics on iOS (success on
complete, light on tap). View transitions.

## Deferred (future phases)

- **Tasks**: subtasks under goals (practise, listen, transcribe, etc.)
- **Session integration**: suggest goal items when starting a session
- **Create set from goal**: bundle a goal's linked items into a set
- **Goal → session linking**: track which sessions contributed to a goal
- **Archiving**: auto-archive completed goals after a period
