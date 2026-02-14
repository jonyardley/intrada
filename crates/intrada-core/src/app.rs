use crux_core::capability::Operation;
use crux_core::render::RenderOperation;
use crux_core::{App, Command, Request};
use serde::{Deserialize, Serialize};

use crate::domain::exercise::{handle_exercise_event, Exercise, ExerciseEvent};
use crate::domain::piece::{handle_piece_event, Piece, PieceEvent};
use crate::model::{LibraryItemView, Model, ViewModel};

#[derive(Default)]
pub struct Intrada;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Event {
    Piece(PieceEvent),
    Exercise(ExerciseEvent),
    DataLoaded {
        pieces: Vec<Piece>,
        exercises: Vec<Exercise>,
    },
    LoadFailed(String),
    ClearError,
}

pub enum Effect {
    Render(Request<RenderOperation>),
    Storage(Box<Request<StorageEffect>>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StorageEffect {
    LoadAll,
    SavePiece(Piece),
    SaveExercise(Exercise),
    UpdatePiece(Piece),
    UpdateExercise(Exercise),
    DeleteItem { id: String },
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
            Event::DataLoaded { pieces, exercises } => {
                model.pieces = pieces;
                model.exercises = exercises;
                model.last_error = None;
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
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let mut items: Vec<LibraryItemView> = Vec::new();

        for piece in &model.pieces {
            items.push(LibraryItemView {
                id: piece.id.clone(),
                item_type: "piece".to_string(),
                title: piece.title.clone(),
                subtitle: piece.composer.clone(),
                key: piece.key.clone(),
                tempo: format_tempo(&piece.tempo),
                notes: piece.notes.clone(),
                tags: piece.tags.clone(),
                created_at: piece.created_at.to_rfc3339(),
                updated_at: piece.updated_at.to_rfc3339(),
            });
        }

        for exercise in &model.exercises {
            items.push(LibraryItemView {
                id: exercise.id.clone(),
                item_type: "exercise".to_string(),
                title: exercise.title.clone(),
                subtitle: exercise.category.clone().or_else(|| exercise.composer.clone()).unwrap_or_default(),
                key: exercise.key.clone(),
                tempo: format_tempo(&exercise.tempo),
                notes: exercise.notes.clone(),
                tags: exercise.tags.clone(),
                created_at: exercise.created_at.to_rfc3339(),
                updated_at: exercise.updated_at.to_rfc3339(),
            });
        }

        // Sort by created_at descending (newest first)
        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let item_count = items.len();

        ViewModel {
            items,
            item_count,
            error: model.last_error.clone(),
            status: None,
        }
    }
}

fn format_tempo(tempo: &Option<crate::domain::types::Tempo>) -> Option<String> {
    let tempo = tempo.as_ref()?;
    match (&tempo.marking, tempo.bpm) {
        (Some(marking), Some(bpm)) => Some(format!("{marking} ({bpm} BPM)")),
        (Some(marking), None) => Some(marking.clone()),
        (None, Some(bpm)) => Some(format!("{bpm} BPM")),
        (None, None) => None,
    }
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
        let mut model = Model::default();
        model.last_error = Some("some error".to_string());

        let _cmd = app.update(Event::ClearError, &mut model);

        assert!(model.last_error.is_none());
    }

    #[test]
    fn test_view_empty_model() {
        let app = Intrada;
        let model = Model::default();
        let vm = app.view(&model);

        assert!(vm.items.is_empty());
        assert_eq!(vm.item_count, 0);
        assert!(vm.error.is_none());
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
            last_error: None,
        };

        let vm = app.view(&model);

        assert_eq!(vm.item_count, 2);
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
    }

    #[test]
    fn test_view_shows_error() {
        let app = Intrada;
        let model = Model {
            pieces: vec![],
            exercises: vec![],
            last_error: Some("Something went wrong".to_string()),
        };

        let vm = app.view(&model);
        assert_eq!(vm.error, Some("Something went wrong".to_string()));
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
        assert_eq!(model.pieces[0].notes, Some("Pièce très jolie — «superbe»".to_string()));
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
                key: if i % 3 == 0 { Some("C Major".to_string()) } else { None },
                tempo: if i % 5 == 0 {
                    Some(crate::domain::types::Tempo {
                        marking: Some("Allegro".to_string()),
                        bpm: Some(120),
                    })
                } else {
                    None
                },
                notes: if i % 7 == 0 { Some(format!("Notes for piece {i}")) } else { None },
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
                key: if i % 4 == 0 { Some("G Major".to_string()) } else { None },
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
        assert_eq!(vm.item_count, 10_000);
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

    #[test]
    fn test_format_tempo() {
        assert_eq!(format_tempo(&None), None);
        assert_eq!(
            format_tempo(&Some(crate::domain::types::Tempo {
                marking: Some("Adagio".to_string()),
                bpm: None
            })),
            Some("Adagio".to_string())
        );
        assert_eq!(
            format_tempo(&Some(crate::domain::types::Tempo {
                marking: None,
                bpm: Some(120)
            })),
            Some("120 BPM".to_string())
        );
        assert_eq!(
            format_tempo(&Some(crate::domain::types::Tempo {
                marking: Some("Allegro".to_string()),
                bpm: Some(132)
            })),
            Some("Allegro (132 BPM)".to_string())
        );
    }
}
