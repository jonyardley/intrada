use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};
use crate::domain::item::ItemKind;
use crate::domain::session::{EntryStatus, SessionStatus, SetlistEntry};
use crate::model::Model;
use crate::validation;

// ── Domain Types ───────────────────────────────────────────────────────

/// A named, reusable setlist template containing an ordered list of library item references.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Set {
    pub id: String,
    pub name: String,
    pub entries: Vec<SetEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single item within a set, representing a library piece or exercise.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SetEntry {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
    pub position: usize,
}

// ── Events ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SetEvent {
    SaveBuildingAsSet {
        name: String,
    },
    SaveSummaryAsSet {
        name: String,
    },
    LoadSetIntoSetlist {
        set_id: String,
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
        SetEvent::SaveBuildingAsSet { name } => {
            // Precondition: must be in Building status
            let building = match &model.session_status {
                SessionStatus::Building(b) => b,
                _ => {
                    model.last_error = Some("Can only save set during building phase".to_string());
                    return crux_core::render::render();
                }
            };

            // Validate name
            if let Err(e) = validation::validate_set_name(&name) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            // Validate entries non-empty
            if building.entries.is_empty() {
                model.last_error = Some("Set must have at least one entry".to_string());
                return crux_core::render::render();
            }

            let now = Utc::now();
            let set = Set {
                id: ulid::Ulid::new().to_string(),
                name: name.trim().to_string(),
                entries: building
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(i, e)| SetEntry {
                        id: ulid::Ulid::new().to_string(),
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
                crate::http::create_set(&model.api_base_url, &set),
                crux_core::render::render(),
            ])
        }

        SetEvent::SaveSummaryAsSet { name } => {
            // Precondition: must be in Summary status
            let summary = match &model.session_status {
                SessionStatus::Summary(s) => s,
                _ => {
                    model.last_error = Some("Can only save set from practice summary".to_string());
                    return crux_core::render::render();
                }
            };

            // Validate name
            if let Err(e) = validation::validate_set_name(&name) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            // Validate entries non-empty
            if summary.entries.is_empty() {
                model.last_error = Some("Set must have at least one entry".to_string());
                return crux_core::render::render();
            }

            let now = Utc::now();
            let set = Set {
                id: ulid::Ulid::new().to_string(),
                name: name.trim().to_string(),
                entries: summary
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(i, e)| SetEntry {
                        id: ulid::Ulid::new().to_string(),
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
                crate::http::create_set(&model.api_base_url, &set),
                crux_core::render::render(),
            ])
        }

        SetEvent::LoadSetIntoSetlist { set_id } => {
            // Precondition: must be in Building status
            let building = match &mut model.session_status {
                SessionStatus::Building(b) => b,
                _ => {
                    model.last_error = Some("Can only load set during building phase".to_string());
                    return crux_core::render::render();
                }
            };

            // Find set by ID
            let set = match model.sets.iter().find(|r| r.id == set_id) {
                Some(r) => r.clone(),
                None => {
                    model.last_error = Some("Set not found".to_string());
                    return crux_core::render::render();
                }
            };

            // Create new SetlistEntry objects from set entries (fresh ULIDs)
            for entry in &set.entries {
                building.entries.push(SetlistEntry {
                    id: ulid::Ulid::new().to_string(),
                    item_id: entry.item_id.clone(),
                    item_title: entry.item_title.clone(),
                    item_type: entry.item_type.clone(),
                    position: 0, // will be reindexed below
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
                });
            }

            // Reindex all positions
            for (i, entry) in building.entries.iter_mut().enumerate() {
                entry.position = i;
            }

            model.last_error = None;
            crux_core::render::render()
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
            // Validate name
            if let Err(e) = validation::validate_set_name(&name) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            // Validate entries non-empty
            if let Err(e) = validation::validate_entries_not_empty(&entries, "Set") {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            // Find and update set
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

            // Reindex positions
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
    use crate::domain::session::BuildingSession;

    fn model_with_building(entries: Vec<SetlistEntry>) -> Model {
        Model {
            api_base_url: "http://localhost:3001".to_string(),
            session_status: SessionStatus::Building(BuildingSession {
                entries,
                session_intention: None,
                target_duration_mins: None,
            }),
            ..Default::default()
        }
    }

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
    fn save_building_as_set_creates_set() {
        let mut model = model_with_building(sample_setlist_entries());
        let _cmd = handle_set_event(
            SetEvent::SaveBuildingAsSet {
                name: "Morning Warm-up".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.sets.len(), 1);
        assert_eq!(model.sets[0].name, "Morning Warm-up");
        assert_eq!(model.sets[0].entries.len(), 2);
        assert_eq!(model.sets[0].entries[0].item_title, "Long Tones");
        assert_eq!(model.sets[0].entries[1].item_title, "C Major Scale");
        assert!(model.last_error.is_none());
    }

    #[test]
    fn save_building_preserves_building_state() {
        let entries = sample_setlist_entries();
        let mut model = model_with_building(entries.clone());
        let _cmd = handle_set_event(
            SetEvent::SaveBuildingAsSet {
                name: "Test".to_string(),
            },
            &mut model,
        );

        // Building state should be preserved
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries.len(), 2);
        } else {
            panic!("Expected building status to be preserved");
        }
    }

    #[test]
    fn save_building_empty_name_fails() {
        let mut model = model_with_building(sample_setlist_entries());
        let _cmd = handle_set_event(
            SetEvent::SaveBuildingAsSet {
                name: "".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.sets.len(), 0);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn save_building_whitespace_only_name_fails() {
        let mut model = model_with_building(sample_setlist_entries());
        let _cmd = handle_set_event(
            SetEvent::SaveBuildingAsSet {
                name: "   ".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.sets.len(), 0);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn save_building_name_too_long_fails() {
        let mut model = model_with_building(sample_setlist_entries());
        let _cmd = handle_set_event(
            SetEvent::SaveBuildingAsSet {
                name: "x".repeat(201),
            },
            &mut model,
        );

        assert_eq!(model.sets.len(), 0);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn save_building_name_at_limit_succeeds() {
        let mut model = model_with_building(sample_setlist_entries());
        let _cmd = handle_set_event(
            SetEvent::SaveBuildingAsSet {
                name: "x".repeat(200),
            },
            &mut model,
        );

        assert_eq!(model.sets.len(), 1);
        assert!(model.last_error.is_none());
    }

    #[test]
    fn save_building_empty_setlist_fails() {
        let mut model = model_with_building(vec![]);
        let _cmd = handle_set_event(
            SetEvent::SaveBuildingAsSet {
                name: "Test".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.sets.len(), 0);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn save_building_wrong_status_fails() {
        let mut model = Model::test_default(); // Idle status
        let _cmd = handle_set_event(
            SetEvent::SaveBuildingAsSet {
                name: "Test".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.sets.len(), 0);
        assert!(model.last_error.is_some());
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
            }),
            ..Default::default()
        };

        let _cmd = handle_set_event(
            SetEvent::SaveSummaryAsSet {
                name: "Post-Session Set".to_string(),
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
            },
            &mut model,
        );

        assert_eq!(model.sets.len(), 0);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn load_set_into_setlist_appends_entries() {
        let set = sample_set();
        let mut model = model_with_building(vec![SetlistEntry {
            id: "existing-1".to_string(),
            item_id: "item-x".to_string(),
            item_title: "Existing Item".to_string(),
            item_type: ItemKind::Piece,
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
        }]);
        model.sets.push(set);

        let _cmd = handle_set_event(
            SetEvent::LoadSetIntoSetlist {
                set_id: "set-1".to_string(),
            },
            &mut model,
        );

        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries.len(), 3); // 1 existing + 2 from set
            assert_eq!(b.entries[0].item_title, "Existing Item");
            assert_eq!(b.entries[1].item_title, "Long Tones");
            assert_eq!(b.entries[2].item_title, "C Major Scale");
            // Positions should be reindexed
            assert_eq!(b.entries[0].position, 0);
            assert_eq!(b.entries[1].position, 1);
            assert_eq!(b.entries[2].position, 2);
            // New entries should have fresh IDs (not matching set entry IDs)
            assert_ne!(b.entries[1].id, "re-1");
            assert_ne!(b.entries[2].id, "re-2");
            // New entries should have default values
            assert_eq!(b.entries[1].duration_secs, 0);
            assert_eq!(b.entries[1].status, EntryStatus::NotAttempted);
            assert!(b.entries[1].notes.is_none());
            assert!(b.entries[1].score.is_none());
        } else {
            panic!("Expected building status");
        }
        assert!(model.last_error.is_none());
    }

    #[test]
    fn load_set_not_building_fails() {
        let mut model = Model::test_default();
        model.sets.push(sample_set());

        let _cmd = handle_set_event(
            SetEvent::LoadSetIntoSetlist {
                set_id: "set-1".to_string(),
            },
            &mut model,
        );

        assert!(model.last_error.is_some());
    }

    #[test]
    fn load_set_not_found_fails() {
        let mut model = model_with_building(vec![]);
        let _cmd = handle_set_event(
            SetEvent::LoadSetIntoSetlist {
                set_id: "nonexistent".to_string(),
            },
            &mut model,
        );

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
            id: ulid::Ulid::new().to_string(),
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

    #[test]
    fn load_set_twice_is_additive() {
        let set = sample_set();
        let mut model = model_with_building(vec![]);
        model.sets.push(set);

        // Load once
        let _cmd = handle_set_event(
            SetEvent::LoadSetIntoSetlist {
                set_id: "set-1".to_string(),
            },
            &mut model,
        );

        // Load again
        let _cmd = handle_set_event(
            SetEvent::LoadSetIntoSetlist {
                set_id: "set-1".to_string(),
            },
            &mut model,
        );

        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries.len(), 4); // 2 + 2
                                            // All entries should have unique IDs
            let ids: Vec<&str> = b.entries.iter().map(|e| e.id.as_str()).collect();
            let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
            assert_eq!(ids.len(), unique.len(), "All entry IDs should be unique");
            // Positions should be reindexed
            for (i, entry) in b.entries.iter().enumerate() {
                assert_eq!(entry.position, i);
            }
        } else {
            panic!("Expected building status");
        }
    }
}
