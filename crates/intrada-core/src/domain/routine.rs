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
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Routine {
    pub id: String,
    pub name: String,
    pub entries: Vec<RoutineEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single item within a routine, representing a library piece or exercise.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct RoutineEntry {
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
pub enum RoutineEvent {
    SaveBuildingAsRoutine {
        name: String,
    },
    SaveSummaryAsRoutine {
        name: String,
    },
    LoadRoutineIntoSetlist {
        routine_id: String,
    },
    DeleteRoutine {
        id: String,
    },
    UpdateRoutine {
        id: String,
        name: String,
        entries: Vec<RoutineEntry>,
    },
}

// ── Handler ────────────────────────────────────────────────────────────

pub fn handle_routine_event(event: RoutineEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        RoutineEvent::SaveBuildingAsRoutine { name } => {
            // Precondition: must be in Building status
            let building = match &model.session_status {
                SessionStatus::Building(b) => b,
                _ => {
                    model.last_error =
                        Some("Can only save routine during building phase".to_string());
                    return crux_core::render::render();
                }
            };

            // Validate name
            if let Err(e) = validation::validate_routine_name(&name) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            // Validate entries non-empty
            if building.entries.is_empty() {
                model.last_error = Some("Routine must have at least one entry".to_string());
                return crux_core::render::render();
            }

            let now = Utc::now();
            let routine = Routine {
                id: ulid::Ulid::new().to_string(),
                name: name.trim().to_string(),
                entries: building
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(i, e)| RoutineEntry {
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

            model.routines.push(routine.clone());
            model.last_error = None;

            Command::all([
                crate::http::create_routine(&model.api_base_url, &routine),
                crux_core::render::render(),
            ])
        }

        RoutineEvent::SaveSummaryAsRoutine { name } => {
            // Precondition: must be in Summary status
            let summary = match &model.session_status {
                SessionStatus::Summary(s) => s,
                _ => {
                    model.last_error =
                        Some("Can only save routine from practice summary".to_string());
                    return crux_core::render::render();
                }
            };

            // Validate name
            if let Err(e) = validation::validate_routine_name(&name) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            // Validate entries non-empty
            if summary.entries.is_empty() {
                model.last_error = Some("Routine must have at least one entry".to_string());
                return crux_core::render::render();
            }

            let now = Utc::now();
            let routine = Routine {
                id: ulid::Ulid::new().to_string(),
                name: name.trim().to_string(),
                entries: summary
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(i, e)| RoutineEntry {
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

            model.routines.push(routine.clone());
            model.last_error = None;

            Command::all([
                crate::http::create_routine(&model.api_base_url, &routine),
                crux_core::render::render(),
            ])
        }

        RoutineEvent::LoadRoutineIntoSetlist { routine_id } => {
            // Precondition: must be in Building status
            let building = match &mut model.session_status {
                SessionStatus::Building(b) => b,
                _ => {
                    model.last_error =
                        Some("Can only load routine during building phase".to_string());
                    return crux_core::render::render();
                }
            };

            // Find routine by ID
            let routine = match model.routines.iter().find(|r| r.id == routine_id) {
                Some(r) => r.clone(),
                None => {
                    model.last_error = Some("Routine not found".to_string());
                    return crux_core::render::render();
                }
            };

            // Create new SetlistEntry objects from routine entries (fresh ULIDs)
            for entry in &routine.entries {
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

        RoutineEvent::DeleteRoutine { id } => {
            model.routines.retain(|r| r.id != id);
            model.last_error = None;

            Command::all([
                crate::http::delete_routine(&model.api_base_url, &id),
                crux_core::render::render(),
            ])
        }

        RoutineEvent::UpdateRoutine { id, name, entries } => {
            // Validate name
            if let Err(e) = validation::validate_routine_name(&name) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            // Validate entries non-empty
            if let Err(e) = validation::validate_entries_not_empty(&entries, "Routine") {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            // Find and update routine
            let routine = match model.routines.iter_mut().find(|r| r.id == id) {
                Some(r) => r,
                None => {
                    model.last_error = Some("Routine not found".to_string());
                    return crux_core::render::render();
                }
            };

            routine.name = name.trim().to_string();
            routine.entries = entries;
            routine.updated_at = Utc::now();

            // Reindex positions
            for (i, entry) in routine.entries.iter_mut().enumerate() {
                entry.position = i;
            }

            let updated = routine.clone();
            model.last_error = None;

            Command::all([
                crate::http::update_routine(&model.api_base_url, &updated),
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

    fn sample_routine() -> Routine {
        Routine {
            id: "routine-1".to_string(),
            name: "Morning Warm-up".to_string(),
            entries: vec![
                RoutineEntry {
                    id: "re-1".to_string(),
                    item_id: "item-a".to_string(),
                    item_title: "Long Tones".to_string(),
                    item_type: ItemKind::Exercise,
                    position: 0,
                },
                RoutineEntry {
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
    fn save_building_as_routine_creates_routine() {
        let mut model = model_with_building(sample_setlist_entries());
        let _cmd = handle_routine_event(
            RoutineEvent::SaveBuildingAsRoutine {
                name: "Morning Warm-up".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.routines.len(), 1);
        assert_eq!(model.routines[0].name, "Morning Warm-up");
        assert_eq!(model.routines[0].entries.len(), 2);
        assert_eq!(model.routines[0].entries[0].item_title, "Long Tones");
        assert_eq!(model.routines[0].entries[1].item_title, "C Major Scale");
        assert!(model.last_error.is_none());
    }

    #[test]
    fn save_building_preserves_building_state() {
        let entries = sample_setlist_entries();
        let mut model = model_with_building(entries.clone());
        let _cmd = handle_routine_event(
            RoutineEvent::SaveBuildingAsRoutine {
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
        let _cmd = handle_routine_event(
            RoutineEvent::SaveBuildingAsRoutine {
                name: "".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.routines.len(), 0);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn save_building_whitespace_only_name_fails() {
        let mut model = model_with_building(sample_setlist_entries());
        let _cmd = handle_routine_event(
            RoutineEvent::SaveBuildingAsRoutine {
                name: "   ".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.routines.len(), 0);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn save_building_name_too_long_fails() {
        let mut model = model_with_building(sample_setlist_entries());
        let _cmd = handle_routine_event(
            RoutineEvent::SaveBuildingAsRoutine {
                name: "x".repeat(201),
            },
            &mut model,
        );

        assert_eq!(model.routines.len(), 0);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn save_building_name_at_limit_succeeds() {
        let mut model = model_with_building(sample_setlist_entries());
        let _cmd = handle_routine_event(
            RoutineEvent::SaveBuildingAsRoutine {
                name: "x".repeat(200),
            },
            &mut model,
        );

        assert_eq!(model.routines.len(), 1);
        assert!(model.last_error.is_none());
    }

    #[test]
    fn save_building_empty_setlist_fails() {
        let mut model = model_with_building(vec![]);
        let _cmd = handle_routine_event(
            RoutineEvent::SaveBuildingAsRoutine {
                name: "Test".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.routines.len(), 0);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn save_building_wrong_status_fails() {
        let mut model = Model::test_default(); // Idle status
        let _cmd = handle_routine_event(
            RoutineEvent::SaveBuildingAsRoutine {
                name: "Test".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.routines.len(), 0);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn save_summary_as_routine_creates_routine() {
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

        let _cmd = handle_routine_event(
            RoutineEvent::SaveSummaryAsRoutine {
                name: "Post-Session Routine".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.routines.len(), 1);
        assert_eq!(model.routines[0].name, "Post-Session Routine");
        assert_eq!(model.routines[0].entries.len(), 2);
        assert!(model.last_error.is_none());
    }

    #[test]
    fn save_summary_wrong_status_fails() {
        let mut model = Model::test_default(); // Idle status
        let _cmd = handle_routine_event(
            RoutineEvent::SaveSummaryAsRoutine {
                name: "Test".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.routines.len(), 0);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn load_routine_into_setlist_appends_entries() {
        let routine = sample_routine();
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
        model.routines.push(routine);

        let _cmd = handle_routine_event(
            RoutineEvent::LoadRoutineIntoSetlist {
                routine_id: "routine-1".to_string(),
            },
            &mut model,
        );

        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries.len(), 3); // 1 existing + 2 from routine
            assert_eq!(b.entries[0].item_title, "Existing Item");
            assert_eq!(b.entries[1].item_title, "Long Tones");
            assert_eq!(b.entries[2].item_title, "C Major Scale");
            // Positions should be reindexed
            assert_eq!(b.entries[0].position, 0);
            assert_eq!(b.entries[1].position, 1);
            assert_eq!(b.entries[2].position, 2);
            // New entries should have fresh IDs (not matching routine entry IDs)
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
    fn load_routine_not_building_fails() {
        let mut model = Model::test_default();
        model.routines.push(sample_routine());

        let _cmd = handle_routine_event(
            RoutineEvent::LoadRoutineIntoSetlist {
                routine_id: "routine-1".to_string(),
            },
            &mut model,
        );

        assert!(model.last_error.is_some());
    }

    #[test]
    fn load_routine_not_found_fails() {
        let mut model = model_with_building(vec![]);
        let _cmd = handle_routine_event(
            RoutineEvent::LoadRoutineIntoSetlist {
                routine_id: "nonexistent".to_string(),
            },
            &mut model,
        );

        assert!(model.last_error.is_some());
    }

    #[test]
    fn delete_routine_removes_from_model() {
        let mut model = Model::test_default();
        model.routines.push(sample_routine());
        assert_eq!(model.routines.len(), 1);

        let _cmd = handle_routine_event(
            RoutineEvent::DeleteRoutine {
                id: "routine-1".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.routines.len(), 0);
        assert!(model.last_error.is_none());
    }

    #[test]
    fn update_routine_changes_name_and_entries() {
        let mut model = Model::test_default();
        model.routines.push(sample_routine());

        let new_entries = vec![RoutineEntry {
            id: ulid::Ulid::new().to_string(),
            item_id: "item-c".to_string(),
            item_title: "New Item".to_string(),
            item_type: ItemKind::Piece,
            position: 0,
        }];

        let _cmd = handle_routine_event(
            RoutineEvent::UpdateRoutine {
                id: "routine-1".to_string(),
                name: "Updated Name".to_string(),
                entries: new_entries,
            },
            &mut model,
        );

        assert_eq!(model.routines.len(), 1);
        assert_eq!(model.routines[0].name, "Updated Name");
        assert_eq!(model.routines[0].entries.len(), 1);
        assert_eq!(model.routines[0].entries[0].item_title, "New Item");
        assert!(model.last_error.is_none());
    }

    #[test]
    fn update_routine_invalid_name_fails() {
        let mut model = Model::test_default();
        model.routines.push(sample_routine());

        let _cmd = handle_routine_event(
            RoutineEvent::UpdateRoutine {
                id: "routine-1".to_string(),
                name: "".to_string(),
                entries: vec![RoutineEntry {
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
        assert_eq!(model.routines[0].name, "Morning Warm-up");
        assert!(model.last_error.is_some());
    }

    #[test]
    fn update_routine_empty_entries_fails() {
        let mut model = Model::test_default();
        model.routines.push(sample_routine());

        let _cmd = handle_routine_event(
            RoutineEvent::UpdateRoutine {
                id: "routine-1".to_string(),
                name: "Updated".to_string(),
                entries: vec![],
            },
            &mut model,
        );

        // Entries should NOT have been changed
        assert_eq!(model.routines[0].entries.len(), 2);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn update_routine_not_found_fails() {
        let mut model = Model::test_default();

        let _cmd = handle_routine_event(
            RoutineEvent::UpdateRoutine {
                id: "nonexistent".to_string(),
                name: "Updated".to_string(),
                entries: vec![RoutineEntry {
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
    fn load_routine_twice_is_additive() {
        let routine = sample_routine();
        let mut model = model_with_building(vec![]);
        model.routines.push(routine);

        // Load once
        let _cmd = handle_routine_event(
            RoutineEvent::LoadRoutineIntoSetlist {
                routine_id: "routine-1".to_string(),
            },
            &mut model,
        );

        // Load again
        let _cmd = handle_routine_event(
            RoutineEvent::LoadRoutineIntoSetlist {
                routine_id: "routine-1".to_string(),
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
