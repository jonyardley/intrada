use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

use super::item::ItemKind;
use super::types::{CreateGoal, LinkGoalItem, UpdateGoal, UpdateGoalItem};
use crate::app::{Effect, Event};
use crate::model::Model;
use crate::validation;

/// Status of a goal.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum GoalStatus {
    #[default]
    Active,
    Completed,
}

/// A goal — a user-defined objective, optionally linked to library items.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Goal {
    pub id: String,
    pub title: Option<String>,
    pub date: String,
    pub notes: Option<String>,
    pub deadline: Option<String>,
    pub status: GoalStatus,
    pub completed_at: Option<DateTime<Utc>>,
    pub items: Vec<GoalItem>,
    pub photos: Vec<GoalPhoto>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_confidence: Option<u8>,
}

/// Photo metadata for a goal. Binary data lives in R2; core only sees metadata.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GoalPhoto {
    pub id: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

/// A library item linked to a goal.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GoalItem {
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_confidence: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GoalEvent {
    FetchGoals,
    FetchGoal {
        id: String,
    },
    Add(CreateGoal),
    Update {
        id: String,
        input: UpdateGoal,
    },
    Complete {
        id: String,
    },
    Reopen {
        id: String,
    },
    Delete {
        id: String,
    },
    LinkItem {
        goal_id: String,
        item: LinkGoalItem,
    },
    UnlinkItem {
        goal_id: String,
        item_id: String,
    },
    UpdateGoalItemTargets {
        goal_id: String,
        item_id: String,
        input: UpdateGoalItem,
    },
}

pub fn handle_goal_event(event: GoalEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        GoalEvent::FetchGoals => crate::http::fetch_goals(&model.api_base_url),
        GoalEvent::FetchGoal { id } => crate::http::fetch_goal(&model.api_base_url, &id),
        GoalEvent::Add(input) => {
            if let Err(e) = validation::validate_create_goal(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let now = chrono::Utc::now();
            let goal = Goal {
                id: ulid::Ulid::new().to_string(),
                title: input.title.clone(),
                date: input.date.clone(),
                notes: input.notes.clone(),
                deadline: input.deadline.clone(),
                status: GoalStatus::Active,
                completed_at: None,
                items: Vec::new(),
                photos: Vec::new(),
                created_at: now,
                updated_at: now,
                target_confidence: input.target_confidence,
            };

            let temp_id = goal.id.clone();
            model.goals.push(goal);
            model.last_error = None;

            Command::all([
                crate::http::create_goal(&model.api_base_url, &input, &temp_id),
                crux_core::render::render(),
            ])
        }
        GoalEvent::Update { id, input } => {
            if let Err(e) = validation::validate_update_goal(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(goal) = model.goals.iter_mut().find(|g| g.id == id) else {
                model.last_error = Some(format!("Goal not found: {id}"));
                return crux_core::render::render();
            };

            if let Some(date) = &input.date {
                goal.date = date.clone();
            }
            if let Some(title) = &input.title {
                goal.title = title.clone();
            }
            if let Some(notes) = &input.notes {
                goal.notes = notes.clone();
            }
            if let Some(deadline) = &input.deadline {
                goal.deadline = deadline.clone();
            }
            if let Some(status) = &input.status {
                goal.status = status.clone();
            }
            if let Some(target_confidence) = &input.target_confidence {
                goal.target_confidence = *target_confidence;
            }
            goal.updated_at = chrono::Utc::now();
            model.last_error = None;

            let goal_id = id.clone();
            Command::all([
                crate::http::update_goal(&model.api_base_url, &goal_id, &input),
                crux_core::render::render(),
            ])
        }
        GoalEvent::Complete { id } => {
            let Some(goal) = model.goals.iter_mut().find(|g| g.id == id) else {
                model.last_error = Some(format!("Goal not found: {id}"));
                return crux_core::render::render();
            };

            goal.status = GoalStatus::Completed;
            goal.completed_at = Some(chrono::Utc::now());
            goal.updated_at = chrono::Utc::now();
            model.last_error = None;

            Command::all([
                crate::http::complete_goal(&model.api_base_url, &id),
                crux_core::render::render(),
            ])
        }
        GoalEvent::Reopen { id } => {
            let Some(goal) = model.goals.iter_mut().find(|g| g.id == id) else {
                model.last_error = Some(format!("Goal not found: {id}"));
                return crux_core::render::render();
            };

            goal.status = GoalStatus::Active;
            goal.completed_at = None;
            goal.updated_at = chrono::Utc::now();
            model.last_error = None;

            Command::all([
                crate::http::reopen_goal(&model.api_base_url, &id),
                crux_core::render::render(),
            ])
        }
        GoalEvent::Delete { id } => {
            let len_before = model.goals.len();
            model.goals.retain(|g| g.id != id);
            if model.goals.len() == len_before {
                model.last_error = Some(format!("Goal not found: {id}"));
                return crux_core::render::render();
            }
            if model.current_goal.as_ref().is_some_and(|g| g.id == id) {
                model.current_goal = None;
            }
            model.last_error = None;

            Command::all([
                crate::http::delete_goal(&model.api_base_url, &id),
                crux_core::render::render(),
            ])
        }
        GoalEvent::LinkItem { goal_id, item } => {
            if let Err(e) = validation::validate_link_goal_item(&item) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }
            if let Some(goal) = model.goals.iter_mut().find(|g| g.id == goal_id) {
                let goal_item = GoalItem {
                    item_id: item.item_id.clone(),
                    item_title: item.item_title.clone(),
                    item_type: item.item_type.clone(),
                    target_date: item.target_date.clone(),
                    target_confidence: item.target_confidence,
                };
                if !goal.items.iter().any(|i| i.item_id == goal_item.item_id) {
                    goal.items.push(goal_item);
                }
                goal.updated_at = chrono::Utc::now();
            }
            model.last_error = None;

            Command::all([
                crate::http::link_goal_item(&model.api_base_url, &goal_id, &item),
                crux_core::render::render(),
            ])
        }
        GoalEvent::UpdateGoalItemTargets {
            goal_id,
            item_id,
            input,
        } => {
            if let Err(e) = validation::validate_update_goal_item(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }
            if let Some(goal) = model.goals.iter_mut().find(|g| g.id == goal_id) {
                if let Some(gi) = goal.items.iter_mut().find(|i| i.item_id == item_id) {
                    if let Some(d) = &input.target_date {
                        gi.target_date = d.clone();
                    }
                    if let Some(c) = &input.target_confidence {
                        gi.target_confidence = *c;
                    }
                    goal.updated_at = chrono::Utc::now();
                }
            }
            model.last_error = None;

            Command::all([
                crate::http::update_goal_item(&model.api_base_url, &goal_id, &item_id, &input),
                crux_core::render::render(),
            ])
        }
        GoalEvent::UnlinkItem { goal_id, item_id } => {
            if let Some(goal) = model.goals.iter_mut().find(|g| g.id == goal_id) {
                goal.items.retain(|i| i.item_id != item_id);
                goal.updated_at = chrono::Utc::now();
            }
            model.last_error = None;

            Command::all([
                crate::http::unlink_goal_item(&model.api_base_url, &goal_id, &item_id),
                crux_core::render::render(),
            ])
        }
    }
}
