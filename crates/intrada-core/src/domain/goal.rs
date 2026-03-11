use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

use super::types::{CreateGoal, UpdateGoal};
use crate::app::{AppEffect, Effect, Event};
use crate::error::LibraryError;
use crate::model::Model;
use crate::validation;

/// The three lifecycle states a goal can be in.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum GoalStatus {
    Active,
    Completed,
    Archived,
}

/// Discriminated union of goal types. Uses internally-tagged serde so
/// the JSON includes `"type": "session_frequency"` etc.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[serde(rename_all = "snake_case", tag = "type")]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum GoalKind {
    SessionFrequency { target_days_per_week: u8 },
    PracticeTime { target_minutes_per_week: u32 },
    ItemMastery { item_id: String, target_score: u8 },
    Milestone { description: String },
}

/// A goal set by the musician.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub kind: GoalKind,
    pub status: GoalStatus,
    pub deadline: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Events that can modify goals.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum GoalEvent {
    Add(CreateGoal),
    Update { id: String, input: UpdateGoal },
    Complete { id: String },
    Archive { id: String },
    Reactivate { id: String },
    Delete { id: String },
}

pub fn handle_goal_event(event: GoalEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        GoalEvent::Add(input) => {
            if let Err(e) = validation::validate_create_goal(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let now = Utc::now();
            let goal = Goal {
                id: ulid::Ulid::new().to_string(),
                title: input.title,
                kind: input.kind,
                status: GoalStatus::Active,
                deadline: input.deadline,
                created_at: now,
                updated_at: now,
                completed_at: None,
            };

            model.goals.push(goal.clone());
            model.last_error = None;

            Command::all([
                Command::notify_shell(AppEffect::SaveGoal(goal)).into(),
                crux_core::render::render(),
            ])
        }
        GoalEvent::Update { id, input } => {
            if let Err(e) = validation::validate_update_goal(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(goal) = model.goals.iter_mut().find(|g| g.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id: id.to_string() }.to_string());
                return crux_core::render::render();
            };

            if let Some(title) = input.title {
                goal.title = title;
            }
            if let Some(deadline) = input.deadline {
                goal.deadline = deadline;
            }
            goal.updated_at = Utc::now();
            model.last_error = None;

            let goal = goal.clone();
            Command::all([
                Command::notify_shell(AppEffect::UpdateGoal(goal)).into(),
                crux_core::render::render(),
            ])
        }
        GoalEvent::Complete { id } => {
            let Some(goal) = model.goals.iter_mut().find(|g| g.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            if goal.status != GoalStatus::Active {
                model.last_error = Some("Only active goals can be completed".to_string());
                return crux_core::render::render();
            }

            let now = Utc::now();
            goal.status = GoalStatus::Completed;
            goal.completed_at = Some(now);
            goal.updated_at = now;
            model.last_error = None;

            let goal = goal.clone();
            Command::all([
                Command::notify_shell(AppEffect::UpdateGoal(goal)).into(),
                crux_core::render::render(),
            ])
        }
        GoalEvent::Archive { id } => {
            let Some(goal) = model.goals.iter_mut().find(|g| g.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            if goal.status != GoalStatus::Active {
                model.last_error = Some("Only active goals can be archived".to_string());
                return crux_core::render::render();
            }

            goal.status = GoalStatus::Archived;
            goal.updated_at = Utc::now();
            model.last_error = None;

            let goal = goal.clone();
            Command::all([
                Command::notify_shell(AppEffect::UpdateGoal(goal)).into(),
                crux_core::render::render(),
            ])
        }
        GoalEvent::Reactivate { id } => {
            let Some(goal) = model.goals.iter_mut().find(|g| g.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            if goal.status != GoalStatus::Archived {
                model.last_error = Some("Only archived goals can be reactivated".to_string());
                return crux_core::render::render();
            }

            goal.status = GoalStatus::Active;
            goal.updated_at = Utc::now();
            model.last_error = None;

            let goal = goal.clone();
            Command::all([
                Command::notify_shell(AppEffect::UpdateGoal(goal)).into(),
                crux_core::render::render(),
            ])
        }
        GoalEvent::Delete { id } => {
            let len_before = model.goals.len();
            model.goals.retain(|g| g.id != id);
            if model.goals.len() == len_before {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            }
            model.last_error = None;

            Command::all([
                Command::notify_shell(AppEffect::DeleteGoal { id }).into(),
                crux_core::render::render(),
            ])
        }
    }
}
