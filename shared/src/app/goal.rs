use crate::app::model::Model;
use serde::{Deserialize, Serialize};

// *************
// GOALS
// *************
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub enum Status {
    #[default]
    NotStarted,
    InProgress,
    Completed,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct PracticeGoal {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: Status,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub exercise_ids: Vec<String>,
}

impl PracticeGoal {
    pub fn new(name: String, description: Option<String>, status: Option<Status>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            status: status.unwrap_or(Status::NotStarted),
            start_date: None,
            end_date: None,
            exercise_ids: Vec::new(),
        }
    }
}

pub fn add_goal(goal: PracticeGoal, model: &mut Model) {
    model.goals.push(goal);
}

pub fn add_exercise_to_goal(goal_id: String, exercise_id: String, model: &mut Model) {
    if let Some(goal) = model.goals.iter_mut().find(|g| g.id == goal_id) {
        if !goal.exercise_ids.contains(&exercise_id) {
            goal.exercise_ids.push(exercise_id);
        }
    }
}
