use crux_core::capability::Operation;
use crux_core::render::RenderOperation;
use crux_core::{App, Command, Request};
use serde::{Deserialize, Serialize};

use crate::domain::exercise::{handle_exercise_event, Exercise, ExerciseEvent};
use crate::domain::piece::{handle_piece_event, Piece, PieceEvent};
use crate::domain::session::{
    handle_session_event, ActiveSession, PracticeSession, SessionEvent, SessionStatus,
};
use crate::domain::types::ListQuery;
use crate::model::{
    build_active_session_view, build_summary_view, entry_to_view, session_to_view,
    BuildingSetlistView, ItemPracticeSummary, LibraryItemView, Model, ViewModel,
};

/// Root Crux application for the music practice library.
#[derive(Default)]
pub struct Intrada;

/// All events the application can process.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Event {
    Piece(PieceEvent),
    Exercise(ExerciseEvent),
    Session(SessionEvent),
    DataLoaded {
        pieces: Vec<Piece>,
        exercises: Vec<Exercise>,
    },
    SessionsLoaded {
        sessions: Vec<PracticeSession>,
    },
    LoadFailed(String),
    ClearError,
    SetQuery(Option<ListQuery>),
}

/// Side effects the core requests from shells.
pub enum Effect {
    Render(Request<RenderOperation>),
    Storage(Box<Request<StorageEffect>>),
}

/// Storage operations handled by the shell.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StorageEffect {
    LoadAll,
    SavePiece(Piece),
    SaveExercise(Exercise),
    UpdatePiece(Piece),
    UpdateExercise(Exercise),
    DeleteItem { id: String },
    LoadSessions,
    SavePracticeSession(PracticeSession),
    DeletePracticeSession { id: String },
    SaveSessionInProgress(ActiveSession),
    ClearSessionInProgress,
}

impl Operation for StorageEffect {
    type Output = ();
}

impl crux_core::Effect for Effect {}

impl From<Request<RenderOperation>> for Effect {
    fn from(request: Request<RenderOperation>) -> Self {
        Effect::Render(request)
    }
}

impl From<Request<StorageEffect>> for Effect {
    fn from(request: Request<StorageEffect>) -> Self {
        Effect::Storage(Box::new(request))
    }
}

impl App for Intrada {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(
        &self,
        event: Self::Event,
        model: &mut Self::Model,
    ) -> Command<Self::Effect, Self::Event> {
        match event {
            Event::Piece(piece_event) => handle_piece_event(piece_event, model),
            Event::Exercise(exercise_event) => handle_exercise_event(exercise_event, model),
            Event::Session(session_event) => handle_session_event(session_event, model),
            Event::DataLoaded { pieces, exercises } => {
                model.pieces = pieces;
                model.exercises = exercises;
                model.last_error = None;
                crux_core::render::render()
            }
            Event::SessionsLoaded { sessions } => {
                model.sessions = sessions;
                crux_core::render::render()
            }
            Event::LoadFailed(msg) => {
                model.last_error = Some(msg);
                crux_core::render::render()
            }
            Event::ClearError => {
                model.last_error = None;
                crux_core::render::render()
            }
            Event::SetQuery(query) => {
                model.active_query = query;
                crux_core::render::render()
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let mut items: Vec<LibraryItemView> = Vec::new();

        for piece in &model.pieces {
            let practice = compute_practice_summary(&model.sessions, &piece.id);
            items.push(LibraryItemView {
                id: piece.id.clone(),
                item_type: "piece".to_string(),
                title: piece.title.clone(),
                subtitle: piece.composer.clone(),
                category: None,
                key: piece.key.clone(),
                tempo: piece
                    .tempo
                    .as_ref()
                    .map(|t| t.format_display())
                    .filter(|s| !s.is_empty()),
                notes: piece.notes.clone(),
                tags: piece.tags.clone(),
                created_at: piece.created_at.to_rfc3339(),
                updated_at: piece.updated_at.to_rfc3339(),
                practice,
            });
        }

        for exercise in &model.exercises {
            let practice = compute_practice_summary(&model.sessions, &exercise.id);
            items.push(LibraryItemView {
                id: exercise.id.clone(),
                item_type: "exercise".to_string(),
                title: exercise.title.clone(),
                subtitle: exercise
                    .category
                    .clone()
                    .or_else(|| exercise.composer.clone())
                    .unwrap_or_default(),
                category: exercise.category.clone(),
                key: exercise.key.clone(),
                tempo: exercise
                    .tempo
                    .as_ref()
                    .map(|t| t.format_display())
                    .filter(|s| !s.is_empty()),
                notes: exercise.notes.clone(),
                tags: exercise.tags.clone(),
                created_at: exercise.created_at.to_rfc3339(),
                updated_at: exercise.updated_at.to_rfc3339(),
                practice,
            });
        }

        // Apply active query filter
        if let Some(ref query) = model.active_query {
            items = apply_query_filter(items, query);
        }

        // Sort by created_at descending (newest first)
        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Build completed session views sorted newest-first
        let mut sessions: Vec<_> = model.sessions.iter().map(session_to_view).collect();
        sessions.sort_by(|a, b| b.finished_at.cmp(&a.finished_at));

        // Build active session / building / summary views from session_status
        let (active_session, building_setlist, summary) = match &model.session_status {
            SessionStatus::Idle => (None, None, None),
            SessionStatus::Building(building) => {
                let entries: Vec<_> = building.entries.iter().map(entry_to_view).collect();
                let item_count = entries.len();
                (
                    None,
                    Some(BuildingSetlistView {
                        entries,
                        item_count,
                    }),
                    None,
                )
            }
            SessionStatus::Active(active) => (Some(build_active_session_view(active)), None, None),
            SessionStatus::Summary(summary_session) => {
                (None, None, Some(build_summary_view(summary_session)))
            }
        };

        let session_status = match &model.session_status {
            SessionStatus::Idle => "idle".to_string(),
            SessionStatus::Building(_) => "building".to_string(),
            SessionStatus::Active(_) => "active".to_string(),
            SessionStatus::Summary(_) => "summary".to_string(),
        };

        ViewModel {
            items,
            sessions,
            active_session,
            building_setlist,
            summary,
            session_status,
            error: model.last_error.clone(),
        }
    }
}

fn compute_practice_summary(
    sessions: &[PracticeSession],
    item_id: &str,
) -> Option<ItemPracticeSummary> {
    let mut session_count = 0usize;
    let mut total_secs = 0u64;

    for session in sessions {
        for entry in &session.entries {
            if entry.item_id == item_id {
                session_count += 1;
                total_secs += entry.duration_secs;
            }
        }
    }

    if session_count == 0 {
        return None;
    }

    Some(ItemPracticeSummary {
        session_count,
        total_minutes: (total_secs / 60) as u32,
    })
}

fn apply_query_filter(items: Vec<LibraryItemView>, query: &ListQuery) -> Vec<LibraryItemView> {
    items
        .into_iter()
        .filter(|item| {
            // Filter by item type
            if let Some(ref item_type) = query.item_type {
                if item.item_type != *item_type {
                    return false;
                }
            }

            // Filter by key
            if let Some(ref key) = query.key {
                if item.key.as_deref() != Some(key.as_str()) {
                    return false;
                }
            }

            // Filter by category (exercises only)
            if let Some(ref category) = query.category {
                if item.category.as_deref() != Some(category.as_str()) {
                    return false;
                }
            }

            // Filter by tags (all must match, case-insensitive)
            if let Some(ref tags) = query.tags {
                for tag in tags {
                    let tag_lower = tag.to_lowercase();
                    if !item.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                        return false;
                    }
                }
            }

            // Filter by text search (case-insensitive substring match)
            if let Some(ref text) = query.text {
                let text_lower = text.to_lowercase();
                let matches = item.title.to_lowercase().contains(&text_lower)
                    || item.subtitle.to_lowercase().contains(&text_lower)
                    || item
                        .notes
                        .as_ref()
                        .is_some_and(|n| n.to_lowercase().contains(&text_lower))
                    || item
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&text_lower))
                    || item
                        .category
                        .as_ref()
                        .is_some_and(|c| c.to_lowercase().contains(&text_lower));
                if !matches {
                    return false;
                }
            }

            true
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_loaded_populates_model() {
        let app = Intrada;
        let mut model = Model::default();

        let now = chrono::Utc::now();
        let pieces = vec![Piece {
            id: "piece1".to_string(),
            title: "Clair de Lune".to_string(),
            composer: "Debussy".to_string(),
            key: Some("Db Major".to_string()),
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        }];
        let exercises = vec![Exercise {
            id: "ex1".to_string(),
            title: "C Major Scale".to_string(),
            composer: None,
            category: Some("Scales".to_string()),
            key: Some("C Major".to_string()),
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        }];

        let _cmd = app.update(
            Event::DataLoaded {
                pieces: pieces.clone(),
                exercises: exercises.clone(),
            },
            &mut model,
        );

        assert_eq!(model.pieces.len(), 1);
        assert_eq!(model.exercises.len(), 1);
        assert_eq!(model.pieces[0].title, "Clair de Lune");
        assert_eq!(model.exercises[0].title, "C Major Scale");
        assert!(model.last_error.is_none());
    }

    #[test]
    fn test_clear_error() {
        let app = Intrada;
        let mut model = Model {
            last_error: Some("some error".to_string()),
            ..Default::default()
        };

        let _cmd = app.update(Event::ClearError, &mut model);

        assert!(model.last_error.is_none());
    }

    #[test]
    fn test_view_empty_model() {
        let app = Intrada;
        let model = Model::default();
        let vm = app.view(&model);

        assert!(vm.items.is_empty());
        assert_eq!(vm.items.len(), 0);
        assert!(vm.error.is_none());
        assert_eq!(vm.session_status, "idle");
    }

    #[test]
    fn test_view_with_items() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let model = Model {
            pieces: vec![Piece {
                id: "p1".to_string(),
                title: "Sonata".to_string(),
                composer: "Beethoven".to_string(),
                key: None,
                tempo: Some(crate::domain::types::Tempo {
                    marking: Some("Allegro".to_string()),
                    bpm: Some(132),
                }),
                notes: None,
                tags: vec!["classical".to_string()],
                created_at: now,
                updated_at: now,
            }],
            exercises: vec![Exercise {
                id: "e1".to_string(),
                title: "Scales".to_string(),
                composer: None,
                category: Some("Technique".to_string()),
                key: None,
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
            }],
            ..Default::default()
        };

        let vm = app.view(&model);

        assert_eq!(vm.items.len(), 2);

        // Check piece
        let piece_view = vm.items.iter().find(|i| i.id == "p1").unwrap();
        assert_eq!(piece_view.item_type, "piece");
        assert_eq!(piece_view.title, "Sonata");
        assert_eq!(piece_view.subtitle, "Beethoven");
        assert_eq!(piece_view.tempo, Some("Allegro (132 BPM)".to_string()));
        assert_eq!(piece_view.tags, vec!["classical".to_string()]);

        // Check exercise
        let ex_view = vm.items.iter().find(|i| i.id == "e1").unwrap();
        assert_eq!(ex_view.item_type, "exercise");
        assert_eq!(ex_view.title, "Scales");
        assert_eq!(ex_view.subtitle, "Technique");
        assert_eq!(ex_view.category, Some("Technique".to_string()));
    }

    #[test]
    fn test_view_shows_error() {
        let app = Intrada;
        let model = Model {
            last_error: Some("Something went wrong".to_string()),
            ..Default::default()
        };

        let vm = app.view(&model);
        assert_eq!(vm.error, Some("Something went wrong".to_string()));
    }

    // --- Query filtering in core ---

    #[test]
    fn test_set_query_filters_by_type() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();

        model.pieces.push(Piece {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            composer: "Beethoven".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });
        model.exercises.push(Exercise {
            id: "e1".to_string(),
            title: "Scales".to_string(),
            composer: None,
            category: Some("Technique".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });

        // No filter — both items
        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 2);

        // Filter to pieces only
        let _cmd = app.update(
            Event::SetQuery(Some(ListQuery {
                item_type: Some("piece".to_string()),
                ..Default::default()
            })),
            &mut model,
        );
        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].item_type, "piece");

        // Clear filter
        let _cmd = app.update(Event::SetQuery(None), &mut model);
        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 2);
    }

    #[test]
    fn test_set_query_filters_by_text() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();

        model.pieces.push(Piece {
            id: "p1".to_string(),
            title: "Moonlight Sonata".to_string(),
            composer: "Beethoven".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });
        model.pieces.push(Piece {
            id: "p2".to_string(),
            title: "Clair de Lune".to_string(),
            composer: "Debussy".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });

        model.active_query = Some(ListQuery {
            text: Some("beethoven".to_string()),
            ..Default::default()
        });

        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "Moonlight Sonata");
    }

    #[test]
    fn test_set_query_filters_by_category() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();

        model.exercises.push(Exercise {
            id: "e1".to_string(),
            title: "C Scale".to_string(),
            composer: Some("Hanon".to_string()),
            category: Some("Scales".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });
        model.exercises.push(Exercise {
            id: "e2".to_string(),
            title: "Chord Inversions".to_string(),
            composer: None,
            category: Some("Chords".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });

        model.active_query = Some(ListQuery {
            category: Some("Scales".to_string()),
            ..Default::default()
        });

        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "C Scale");
    }

    #[test]
    fn test_set_query_filters_by_tags() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();

        model.pieces.push(Piece {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            composer: "Beethoven".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec!["classical".to_string(), "piano".to_string()],
            created_at: now,
            updated_at: now,
        });
        model.pieces.push(Piece {
            id: "p2".to_string(),
            title: "Etude".to_string(),
            composer: "Chopin".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec!["romantic".to_string(), "piano".to_string()],
            created_at: now,
            updated_at: now,
        });

        model.active_query = Some(ListQuery {
            tags: Some(vec!["classical".to_string()]),
            ..Default::default()
        });

        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "Sonata");
    }

    // --- T042: Unicode handling in core ---

    #[test]
    fn test_unicode_in_piece_add() {
        let app = Intrada;
        let mut model = Model::default();

        let _cmd = app.update(
            Event::Piece(crate::domain::piece::PieceEvent::Add(
                crate::domain::types::CreatePiece {
                    title: "Ménuet en Sol".to_string(),
                    composer: "Dvořák".to_string(),
                    key: Some("ré mineur".to_string()),
                    tempo: None,
                    notes: Some("Pièce très jolie — «superbe»".to_string()),
                    tags: vec!["日本語タグ".to_string()],
                },
            )),
            &mut model,
        );

        assert!(model.last_error.is_none());
        assert_eq!(model.pieces.len(), 1);
        assert_eq!(model.pieces[0].title, "Ménuet en Sol");
        assert_eq!(model.pieces[0].composer, "Dvořák");
        assert_eq!(model.pieces[0].key, Some("ré mineur".to_string()));
        assert_eq!(
            model.pieces[0].notes,
            Some("Pièce très jolie — «superbe»".to_string())
        );
        assert_eq!(model.pieces[0].tags, vec!["日本語タグ".to_string()]);

        // Verify ViewModel preserves Unicode
        let vm = app.view(&model);
        assert_eq!(vm.items[0].title, "Ménuet en Sol");
        assert_eq!(vm.items[0].subtitle, "Dvořák");
    }

    // --- T045: Performance benchmark ---

    #[test]
    fn test_performance_10k_items() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();

        // Populate 10,000 items (5k pieces + 5k exercises)
        let start = std::time::Instant::now();
        for i in 0..5000 {
            model.pieces.push(Piece {
                id: format!("p{i:05}"),
                title: format!("Piece {i}"),
                composer: format!("Composer {}", i % 100),
                key: if i % 3 == 0 {
                    Some("C Major".to_string())
                } else {
                    None
                },
                tempo: if i % 5 == 0 {
                    Some(crate::domain::types::Tempo {
                        marking: Some("Allegro".to_string()),
                        bpm: Some(120),
                    })
                } else {
                    None
                },
                notes: if i % 7 == 0 {
                    Some(format!("Notes for piece {i}"))
                } else {
                    None
                },
                tags: vec![format!("tag{}", i % 10)],
                created_at: now,
                updated_at: now,
            });
        }
        for i in 0..5000 {
            model.exercises.push(Exercise {
                id: format!("e{i:05}"),
                title: format!("Exercise {i}"),
                composer: None,
                category: Some(format!("Category {}", i % 20)),
                key: if i % 4 == 0 {
                    Some("G Major".to_string())
                } else {
                    None
                },
                tempo: None,
                notes: None,
                tags: vec![format!("etag{}", i % 10)],
                created_at: now,
                updated_at: now,
            });
        }
        let populate_time = start.elapsed();
        assert!(
            populate_time.as_millis() < 100,
            "Populating 10k items took {}ms (target: <100ms)",
            populate_time.as_millis()
        );

        // Benchmark: view() with 10k items
        let start = std::time::Instant::now();
        let vm = app.view(&model);
        let view_time = start.elapsed();
        assert_eq!(vm.items.len(), 10_000);
        assert!(
            view_time.as_millis() < 200,
            "view() with 10k items took {}ms (target: <200ms)",
            view_time.as_millis()
        );

        // Benchmark: add one more item with 10k existing
        let start = std::time::Instant::now();
        let _cmd = app.update(
            Event::Piece(crate::domain::piece::PieceEvent::Add(
                crate::domain::types::CreatePiece {
                    title: "New Piece".to_string(),
                    composer: "New Composer".to_string(),
                    key: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                },
            )),
            &mut model,
        );
        let add_time = start.elapsed();
        assert_eq!(model.pieces.len(), 5001);
        assert!(
            add_time.as_millis() < 100,
            "Adding item with 10k existing took {}ms (target: <100ms)",
            add_time.as_millis()
        );

        // Benchmark: delete item with 10k existing
        let start = std::time::Instant::now();
        let _cmd = app.update(
            Event::Piece(crate::domain::piece::PieceEvent::Delete {
                id: "p00042".to_string(),
            }),
            &mut model,
        );
        let delete_time = start.elapsed();
        assert_eq!(model.pieces.len(), 5000);
        assert!(
            delete_time.as_millis() < 100,
            "Deleting item with 10k existing took {}ms (target: <100ms)",
            delete_time.as_millis()
        );
    }

    // --- Practice summary with new setlist sessions ---

    #[test]
    fn test_view_practice_summary_with_setlist_sessions() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model::default();

        let p1 = Piece {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            composer: "Beethoven".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        };
        let p2 = Piece {
            id: "p2".to_string(),
            title: "Etude".to_string(),
            composer: "Chopin".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        };
        model.pieces = vec![p1, p2];

        // Create a completed session with two entries
        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };
        model.sessions.push(PracticeSession {
            id: "sess1".to_string(),
            started_at: now - chrono::Duration::minutes(60),
            completed_at: now,
            total_duration_secs: 2700,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            entries: vec![
                SetlistEntry {
                    id: "e1".to_string(),
                    item_id: "p1".to_string(),
                    item_title: "Sonata".to_string(),
                    item_type: "piece".to_string(),
                    position: 0,
                    duration_secs: 1800, // 30 min
                    status: EntryStatus::Completed,
                    notes: None,
                },
                SetlistEntry {
                    id: "e2".to_string(),
                    item_id: "p1".to_string(),
                    item_title: "Sonata".to_string(),
                    item_type: "piece".to_string(),
                    position: 1,
                    duration_secs: 900, // 15 min
                    status: EntryStatus::Completed,
                    notes: None,
                },
            ],
        });

        let vm = app.view(&model);
        let p1_view = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let p2_view = vm.items.iter().find(|i| i.id == "p2").unwrap();

        // p1 has 2 entries totalling 45 minutes
        assert_eq!(
            p1_view.practice,
            Some(ItemPracticeSummary {
                session_count: 2,
                total_minutes: 45,
            })
        );
        // p2 has no entries
        assert_eq!(p2_view.practice, None);
    }

    #[test]
    fn test_view_empty_sessions() {
        let app = Intrada;
        let model = Model::default();
        let vm = app.view(&model);
        assert!(vm.sessions.is_empty());
    }

    #[test]
    fn test_tempo_format_display() {
        use crate::domain::types::Tempo;

        // None tempo — map returns None
        let none_tempo: Option<Tempo> = None;
        assert_eq!(none_tempo.as_ref().map(|t| t.format_display()), None);

        // Both None — empty string
        let tempo = Tempo {
            marking: None,
            bpm: None,
        };
        assert_eq!(tempo.format_display(), "");

        // Marking only
        let tempo = Tempo {
            marking: Some("Adagio".to_string()),
            bpm: None,
        };
        assert_eq!(tempo.format_display(), "Adagio");

        // BPM only
        let tempo = Tempo {
            marking: None,
            bpm: Some(120),
        };
        assert_eq!(tempo.format_display(), "120 BPM");

        // Both
        let tempo = Tempo {
            marking: Some("Allegro".to_string()),
            bpm: Some(132),
        };
        assert_eq!(tempo.format_display(), "Allegro (132 BPM)");
    }
}
