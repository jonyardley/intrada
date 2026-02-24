use chrono::{DateTime, Utc};
use libsql::Connection;

use intrada_core::domain::goal::{Goal, GoalKind, GoalStatus};
use intrada_core::domain::types::CreateGoal;

use super::col;
use crate::error::ApiError;

/// Column list for SELECT queries — keeps positional indexing in one place.
const SELECT_COLUMNS: &str = "id, user_id, title, goal_type, status, target_days_per_week, target_minutes_per_week, item_id, target_score, milestone_description, deadline, created_at, updated_at, completed_at";

/// Helper: parse a goal_type string + nullable type-specific columns into GoalKind.
fn parse_goal_kind(
    goal_type: &str,
    target_days_per_week: Option<i64>,
    target_minutes_per_week: Option<i64>,
    item_id: Option<String>,
    target_score: Option<i64>,
    milestone_description: Option<String>,
) -> Result<GoalKind, ApiError> {
    match goal_type {
        "session_frequency" => Ok(GoalKind::SessionFrequency {
            target_days_per_week: target_days_per_week.unwrap_or(1) as u8,
        }),
        "practice_time" => Ok(GoalKind::PracticeTime {
            target_minutes_per_week: target_minutes_per_week.unwrap_or(1) as u32,
        }),
        "item_mastery" => Ok(GoalKind::ItemMastery {
            item_id: item_id.unwrap_or_default(),
            target_score: target_score.unwrap_or(1) as u8,
        }),
        "milestone" => Ok(GoalKind::Milestone {
            description: milestone_description.unwrap_or_default(),
        }),
        other => Err(ApiError::Internal(format!("Unknown goal type: {other}"))),
    }
}

/// Helper: get the goal_type discriminant string for a GoalKind.
fn goal_type_str(kind: &GoalKind) -> &'static str {
    match kind {
        GoalKind::SessionFrequency { .. } => "session_frequency",
        GoalKind::PracticeTime { .. } => "practice_time",
        GoalKind::ItemMastery { .. } => "item_mastery",
        GoalKind::Milestone { .. } => "milestone",
    }
}

/// Helper: parse a status string from the database into GoalStatus.
fn parse_status(s: &str) -> Result<GoalStatus, ApiError> {
    match s {
        "active" => Ok(GoalStatus::Active),
        "completed" => Ok(GoalStatus::Completed),
        "archived" => Ok(GoalStatus::Archived),
        other => Err(ApiError::Internal(format!("Unknown goal status: {other}"))),
    }
}

/// Helper: serialize GoalStatus to string for storage.
fn status_str(status: &GoalStatus) -> &'static str {
    match status {
        GoalStatus::Active => "active",
        GoalStatus::Completed => "completed",
        GoalStatus::Archived => "archived",
    }
}

/// Helper: parse a row from the goals table into a Goal.
fn row_to_goal(row: &libsql::Row) -> Result<Goal, ApiError> {
    let id: String = col!(row, 0)?;
    let _user_id: String = col!(row, 1)?;
    let title: String = col!(row, 2)?;
    let goal_type: String = col!(row, 3)?;
    let status_str_val: String = col!(row, 4)?;
    let target_days_per_week: Option<i64> = col!(row, 5)?;
    let target_minutes_per_week: Option<i64> = col!(row, 6)?;
    let item_id: Option<String> = col!(row, 7)?;
    let target_score: Option<i64> = col!(row, 8)?;
    let milestone_description: Option<String> = col!(row, 9)?;
    let deadline_str: Option<String> = col!(row, 10)?;
    let created_at_str: String = col!(row, 11)?;
    let updated_at_str: String = col!(row, 12)?;
    let completed_at_str: Option<String> = col!(row, 13)?;

    let kind = parse_goal_kind(
        &goal_type,
        target_days_per_week,
        target_minutes_per_week,
        item_id,
        target_score,
        milestone_description,
    )?;

    let status = parse_status(&status_str_val)?;

    let created_at: DateTime<Utc> = created_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid created_at: {e}")))?;
    let updated_at: DateTime<Utc> = updated_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid updated_at: {e}")))?;
    let deadline: Option<DateTime<Utc>> = deadline_str
        .map(|s| {
            s.parse()
                .map_err(|e| ApiError::Internal(format!("Invalid deadline: {e}")))
        })
        .transpose()?;
    let completed_at: Option<DateTime<Utc>> = completed_at_str
        .map(|s| {
            s.parse()
                .map_err(|e| ApiError::Internal(format!("Invalid completed_at: {e}")))
        })
        .transpose()?;

    Ok(Goal {
        id,
        title,
        kind,
        status,
        deadline,
        created_at,
        updated_at,
        completed_at,
    })
}

pub async fn list_goals(conn: &Connection, user_id: &str) -> Result<Vec<Goal>, ApiError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {SELECT_COLUMNS} FROM goals WHERE user_id = ?1 ORDER BY created_at DESC"
            ),
            libsql::params![user_id],
        )
        .await?;

    let mut goals = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        goals.push(row_to_goal(&row)?);
    }
    Ok(goals)
}

pub async fn get_goal(
    conn: &Connection,
    id: &str,
    user_id: &str,
) -> Result<Option<Goal>, ApiError> {
    let mut rows = conn
        .query(
            &format!("SELECT {SELECT_COLUMNS} FROM goals WHERE id = ?1 AND user_id = ?2"),
            libsql::params![id, user_id],
        )
        .await?;

    match rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(row) => Ok(Some(row_to_goal(&row)?)),
        None => Ok(None),
    }
}

pub async fn insert_goal(
    conn: &Connection,
    user_id: &str,
    input: &CreateGoal,
) -> Result<Goal, ApiError> {
    let id = ulid::Ulid::new().to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let goal_type = goal_type_str(&input.kind);
    let status = "active";

    let (
        target_days_per_week,
        target_minutes_per_week,
        item_id,
        target_score,
        milestone_description,
    ) = match &input.kind {
        GoalKind::SessionFrequency {
            target_days_per_week,
        } => (Some(*target_days_per_week as i64), None, None, None, None),
        GoalKind::PracticeTime {
            target_minutes_per_week,
        } => (
            None,
            Some(*target_minutes_per_week as i64),
            None,
            None,
            None,
        ),
        GoalKind::ItemMastery {
            item_id,
            target_score,
        } => (
            None,
            None,
            Some(item_id.as_str()),
            Some(*target_score as i64),
            None,
        ),
        GoalKind::Milestone { description } => (None, None, None, None, Some(description.as_str())),
    };

    let deadline_str = input.deadline.map(|d| d.to_rfc3339());

    conn.execute(
        "INSERT INTO goals (id, user_id, title, goal_type, status, target_days_per_week, target_minutes_per_week, item_id, target_score, milestone_description, deadline, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        libsql::params![
            id.as_str(),
            user_id,
            input.title.as_str(),
            goal_type,
            status,
            target_days_per_week,
            target_minutes_per_week,
            item_id,
            target_score,
            milestone_description,
            deadline_str.as_deref(),
            now_str.as_str(),
            now_str.as_str()
        ],
    )
    .await?;

    Ok(Goal {
        id,
        title: input.title.clone(),
        kind: input.kind.clone(),
        status: GoalStatus::Active,
        deadline: input.deadline,
        created_at: now,
        updated_at: now,
        completed_at: None,
    })
}

pub async fn update_goal(conn: &Connection, user_id: &str, goal: &Goal) -> Result<(), ApiError> {
    let goal_type = goal_type_str(&goal.kind);
    let status = status_str(&goal.status);
    let now_str = goal.updated_at.to_rfc3339();
    let deadline_str = goal.deadline.map(|d| d.to_rfc3339());
    let completed_at_str = goal.completed_at.map(|d| d.to_rfc3339());

    conn.execute(
        "UPDATE goals SET title = ?1, status = ?2, deadline = ?3, updated_at = ?4, completed_at = ?5 WHERE id = ?6 AND user_id = ?7",
        libsql::params![
            goal.title.as_str(),
            status,
            deadline_str.as_deref(),
            now_str.as_str(),
            completed_at_str.as_deref(),
            goal.id.as_str(),
            user_id
        ],
    )
    .await?;

    // Also update goal_type-specific columns in case kind was modified at a higher level
    let (
        target_days_per_week,
        target_minutes_per_week,
        item_id,
        target_score,
        milestone_description,
    ) = match &goal.kind {
        GoalKind::SessionFrequency {
            target_days_per_week,
        } => (Some(*target_days_per_week as i64), None, None, None, None),
        GoalKind::PracticeTime {
            target_minutes_per_week,
        } => (
            None,
            Some(*target_minutes_per_week as i64),
            None,
            None,
            None,
        ),
        GoalKind::ItemMastery {
            item_id,
            target_score,
        } => (
            None,
            None,
            Some(item_id.as_str()),
            Some(*target_score as i64),
            None,
        ),
        GoalKind::Milestone { description } => (None, None, None, None, Some(description.as_str())),
    };

    conn.execute(
        "UPDATE goals SET goal_type = ?1, target_days_per_week = ?2, target_minutes_per_week = ?3, item_id = ?4, target_score = ?5, milestone_description = ?6 WHERE id = ?7 AND user_id = ?8",
        libsql::params![
            goal_type,
            target_days_per_week,
            target_minutes_per_week,
            item_id,
            target_score,
            milestone_description,
            goal.id.as_str(),
            user_id
        ],
    )
    .await?;

    Ok(())
}

pub async fn delete_goal(conn: &Connection, id: &str, user_id: &str) -> Result<bool, ApiError> {
    let rows_affected = conn
        .execute(
            "DELETE FROM goals WHERE id = ?1 AND user_id = ?2",
            libsql::params![id, user_id],
        )
        .await?;

    Ok(rows_affected > 0)
}
