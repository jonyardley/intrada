use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};
use crate::domain::item::ItemKind;
use crate::domain::session::SessionStatus;
use crate::model::Model;
use crate::validation;

// ── Domain Types ───────────────────────────────────────────────────────

/// A named, reusable setlist template (ordered library-item references).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Set {
    pub id: String,
    pub name: String,
    pub entries: Vec<SetEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SetEntry {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
    pub position: usize,
}

// ── Events ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum SetEvent {
    SaveSummaryAsSet {
        name: String,
        request_id: String,
    },
    DeleteSet {
        id: String,
    },
    UpdateSet {
        id: String,
        name: String,
        entries: Vec<SetEntry>,
    },
}

// ── Handler ────────────────────────────────────────────────────────────

pub fn handle_set_event(event: SetEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        SetEvent::SaveSummaryAsSet { name, request_id } => {
            let summary = match &model.session_status {
                SessionStatus::Summary(s) => s,
                _ => {
                    model.last_error = Some("Can only save set from practice summary".to_string());
                    return crux_core::render::render();
                }
            };

            if let Err(e) = validation::validate_set_name(&name) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            if summary.entries.is_empty() {
                model.last_error = Some("Set must have at least one entry".to_string());
                return crux_core::render::render();
            }

            let now = Utc::now();
            let set = Set {
                id: ulid::Ulid::generate().to_string(),
                name: name.trim().to_string(),
                entries: summary
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(i, e)| SetEntry {
                        id: ulid::Ulid::generate().to_string(),
                        item_id: e.item_id.clone(),
                        item_title: e.item_title.clone(),
                        item_type: e.item_type.clone(),
                        position: i,
                    })
                    .collect(),
                created_at: now,
                updated_at: now,
            };

            model.sets.push(set.clone());
            model.last_error = None;

            Command::all([
                crate::http::create_set(&model.api_base_url, &set, request_id),
                crux_core::render::render(),
            ])
        }

        SetEvent::DeleteSet { id } => {
            model.sets.retain(|r| r.id != id);
            model.last_error = None;

            Command::all([
                crate::http::delete_set(&model.api_base_url, &id),
                crux_core::render::render(),
            ])
        }

        SetEvent::UpdateSet { id, name, entries } => {
            if let Err(e) = validation::validate_set_name(&name) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            if let Err(e) = validation::validate_entries_not_empty(&entries, "Set") {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let set = match model.sets.iter_mut().find(|r| r.id == id) {
                Some(r) => r,
                None => {
                    model.last_error = Some("Set not found".to_string());
                    return crux_core::render::render();
                }
            };

            set.name = name.trim().to_string();
            set.entries = entries;
            set.updated_at = Utc::now();

            for (i, entry) in set.entries.iter_mut().enumerate() {
                entry.position = i;
            }

            let updated = set.clone();
            model.last_error = None;

            Command::all([
                crate::http::update_set(&model.api_base_url, &updated),
                crux_core::render::render(),
            ])
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::session::{EntryStatus, SetlistEntry};

    fn sample_setlist_entries() -> Vec<SetlistEntry> {
        vec![
            SetlistEntry {
                id: "entry-1".to_string(),
                item_id: "item-a".to_string(),
                item_title: "Long Tones".to_string(),
                item_type: ItemKind::Exercise,
                position: 0,
                duration_secs: 0,
                status: EntryStatus::NotAttempted,
                notes: None,
                score: None,
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: None,
                group_id: None,
                variant_id: None,
            },
            SetlistEntry {
                id: "entry-2".to_string(),
                item_id: "item-b".to_string(),
                item_title: "C Major Scale".to_string(),
                item_type: ItemKind::Exercise,
                position: 1,
                duration_secs: 0,
                status: EntryStatus::NotAttempted,
                notes: None,
                score: None,
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: None,
                group_id: None,
                variant_id: None,
            },
        ]
    }

    fn sample_set() -> Set {
        Set {
            id: "set-1".to_string(),
            name: "Morning Warm-up".to_string(),
            entries: vec![
                SetEntry {
                    id: "re-1".to_string(),
                    item_id: "item-a".to_string(),
                    item_title: "Long Tones".to_string(),
                    item_type: ItemKind::Exercise,
                    position: 0,
                },
                SetEntry {
                    id: "re-2".to_string(),
                    item_id: "item-b".to_string(),
                    item_title: "C Major Scale".to_string(),
                    item_type: ItemKind::Exercise,
                    position: 1,
                },
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn save_summary_as_set_creates_set() {
        use crate::domain::session::{CompletionStatus, SummarySession};

        let mut model = Model {
            api_base_url: "http://localhost:3001".to_string(),
            session_status: SessionStatus::Summary(SummarySession {
                id: "session-1".to_string(),
                entries: sample_setlist_entries(),
                session_started_at: Utc::now(),
                session_ended_at: Utc::now(),
                session_notes: None,
                session_intention: None,
                completion_status: CompletionStatus::Completed,
                session_score: None,
                reflection_improved: None,
                reflection_still_rough: None,
                reflection_next_target: None,
            }),
            ..Default::default()
        };

        let _cmd = handle_set_event(
            SetEvent::SaveSummaryAsSet {
                name: "Post-Session Set".to_string(),
                request_id: "req-test".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.sets.len(), 1);
        assert_eq!(model.sets[0].name, "Post-Session Set");
        assert_eq!(model.sets[0].entries.len(), 2);
        assert!(model.last_error.is_none());
    }

    #[test]
    fn save_summary_wrong_status_fails() {
        let mut model = Model::test_default(); // Idle status
        let _cmd = handle_set_event(
            SetEvent::SaveSummaryAsSet {
                name: "Test".to_string(),
                request_id: "req-test".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.sets.len(), 0);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn delete_set_removes_from_model() {
        let mut model = Model::test_default();
        model.sets.push(sample_set());
        assert_eq!(model.sets.len(), 1);

        let _cmd = handle_set_event(
            SetEvent::DeleteSet {
                id: "set-1".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.sets.len(), 0);
        assert!(model.last_error.is_none());
    }

    #[test]
    fn update_set_changes_name_and_entries() {
        let mut model = Model::test_default();
        model.sets.push(sample_set());

        let new_entries = vec![SetEntry {
            id: ulid::Ulid::generate().to_string(),
            item_id: "item-c".to_string(),
            item_title: "New Item".to_string(),
            item_type: ItemKind::Piece,
            position: 0,
        }];

        let _cmd = handle_set_event(
            SetEvent::UpdateSet {
                id: "set-1".to_string(),
                name: "Updated Name".to_string(),
                entries: new_entries,
            },
            &mut model,
        );

        assert_eq!(model.sets.len(), 1);
        assert_eq!(model.sets[0].name, "Updated Name");
        assert_eq!(model.sets[0].entries.len(), 1);
        assert_eq!(model.sets[0].entries[0].item_title, "New Item");
        assert!(model.last_error.is_none());
    }

    #[test]
    fn update_set_invalid_name_fails() {
        let mut model = Model::test_default();
        model.sets.push(sample_set());

        let _cmd = handle_set_event(
            SetEvent::UpdateSet {
                id: "set-1".to_string(),
                name: "".to_string(),
                entries: vec![SetEntry {
                    id: "re-1".to_string(),
                    item_id: "item-a".to_string(),
                    item_title: "Long Tones".to_string(),
                    item_type: ItemKind::Exercise,
                    position: 0,
                }],
            },
            &mut model,
        );

        // Name should NOT have been changed
        assert_eq!(model.sets[0].name, "Morning Warm-up");
        assert!(model.last_error.is_some());
    }

    #[test]
    fn update_set_empty_entries_fails() {
        let mut model = Model::test_default();
        model.sets.push(sample_set());

        let _cmd = handle_set_event(
            SetEvent::UpdateSet {
                id: "set-1".to_string(),
                name: "Updated".to_string(),
                entries: vec![],
            },
            &mut model,
        );

        // Entries should NOT have been changed
        assert_eq!(model.sets[0].entries.len(), 2);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn update_set_not_found_fails() {
        let mut model = Model::test_default();

        let _cmd = handle_set_event(
            SetEvent::UpdateSet {
                id: "nonexistent".to_string(),
                name: "Updated".to_string(),
                entries: vec![SetEntry {
                    id: "re-1".to_string(),
                    item_id: "item-a".to_string(),
                    item_title: "Long Tones".to_string(),
                    item_type: ItemKind::Exercise,
                    position: 0,
                }],
            },
            &mut model,
        );

        assert!(model.last_error.is_some());
    }
}
