# Data Model: Basic Goal Setting

**Feature**: 152-goal-setting
**Date**: 2026-02-24

## Entities

### Goal

The central persisted entity representing a user's practice goal.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | String (ULID) | Yes | Unique identifier, generated server-side |
| title | String | Yes | User-visible goal title (max 200 chars) |
| goal_type | Enum | Yes | Discriminant: `session_frequency`, `practice_time`, `item_mastery`, `milestone` |
| status | Enum | Yes | `active`, `completed`, `archived` (default: `active`) |
| target_days_per_week | Integer (1-7) | If frequency | Target practice days per ISO week |
| target_minutes_per_week | Integer (1-10080) | If time | Target practice minutes per ISO week |
| item_id | String | If mastery | Foreign key to library item |
| target_score | Integer (1-5) | If mastery | Target mastery score |
| milestone_description | String | If milestone | Milestone details (max 1000 chars) |
| deadline | DateTime (UTC) | No | Optional deadline for any goal type |
| created_at | DateTime (UTC) | Yes | Creation timestamp |
| updated_at | DateTime (UTC) | Yes | Last modification timestamp |
| completed_at | DateTime (UTC) | No | Set when status transitions to `completed` |
| user_id | String | Yes | Owner (from JWT `sub` claim) |

**Identity & uniqueness**: `id` is the primary key (ULID). No uniqueness constraint across goal_type + item_id — multiple mastery goals for the same item are permitted.

**Validation rules**:
- `title`: 1-200 characters, required
- `target_days_per_week`: 1-7 (only when goal_type = session_frequency)
- `target_minutes_per_week`: 1-10080 (only when goal_type = practice_time)
- `target_score`: 1-5 (only when goal_type = item_mastery)
- `milestone_description`: 0-1000 characters (only when goal_type = milestone)
- `goal_type`: immutable after creation
- `status`: valid transitions only (Active → Completed, Active → Archived, Archived → Active)

### GoalKind (Discriminated Union — not a separate table)

Represented as a tagged enum in the domain layer, stored as flat columns in the database.

| Variant | Discriminant | Type-specific fields |
|---------|-------------|---------------------|
| SessionFrequency | `session_frequency` | target_days_per_week |
| PracticeTime | `practice_time` | target_minutes_per_week |
| ItemMastery | `item_mastery` | item_id, target_score |
| Milestone | `milestone` | milestone_description |

### GoalProgress (Computed — not persisted)

Calculated at view-build time from Goal + Session data.

| Field | Type | Description |
|-------|------|-------------|
| current_value | Float | Current progress value (e.g., 3.0 days, 85.0 minutes, 3.0 score) |
| target_value | Float | Target value from goal kind |
| percentage | Float (0.0-100.0) | current / target * 100, capped at 100.0 |
| display_text | String | Human-readable progress (e.g., "3 of 5 days — great spacing for retention") |

**Computation rules by type**:
- **SessionFrequency**: Count distinct days with at least one session in current ISO week → current_value / target_days_per_week
- **PracticeTime**: Sum `total_duration_secs / 60` for sessions in current ISO week → current_value / target_minutes_per_week
- **ItemMastery**: Latest score from practice_summaries for the linked item → current_value / target_score
- **Milestone**: 0% if active, 100% if completed. Display: "In progress — mark complete when ready"

## State Transitions

```
           ┌──────────────┐
           │    Active     │
           └──────┬───────┘
                  │
          ┌───────┴────────┐
          ▼                ▼
  ┌──────────────┐  ┌──────────────┐
  │  Completed   │  │   Archived   │
  │   (final)    │  │ (reversible) │
  └──────────────┘  └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │    Active    │
                    │ (reactivated)│
                    └──────────────┘
```

- Active → Completed: Sets `completed_at`, final (no reversal)
- Active → Archived: Soft removal, can be reactivated
- Archived → Active: Reactivation, clears any archived state, progress recalculated

## Relationships

- **Goal → User**: Many-to-one via `user_id` (not a foreign key — user lives in Clerk)
- **Goal (item_mastery) → Item**: Many-to-one via `item_id`. If the referenced item is deleted, the goal remains with an orphaned reference (UI shows "Item no longer in library")

## Indexes

| Index | Columns | Purpose |
|-------|---------|---------|
| PRIMARY KEY | id | Unique goal lookup |
| idx_goals_user_id | user_id | All queries filter by user |
