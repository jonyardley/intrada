use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::types::{CreateItem, Tempo, UpdateItem};
use crate::app::{Effect, Event};
use crate::error::LibraryError;
use crate::model::Model;
use crate::validation;

/// Discriminates between a piece (repertoire) and an exercise (technique drill).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Piece,
    Exercise,
}

impl fmt::Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemKind::Piece => write!(f, "piece"),
            ItemKind::Exercise => write!(f, "exercise"),
        }
    }
}

/// Major/minor tonality, paired with `Item.key` (the tonic, e.g. "F#").
/// Selection/spelling logic lives in the shell's key picker.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    Major,
    Minor,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Item {
    pub id: String,
    pub title: String,
    pub kind: ItemKind,
    pub composer: Option<String>,
    pub key: Option<String>,
    #[serde(default)]
    pub modality: Option<Modality>,
    pub tempo: Option<Tempo>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    // `#[serde(default)]` so absent fields (old clients / bincode) default to `[]`.
    #[serde(default)]
    pub linked_exercise_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub priority: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ItemEvent {
    Add(CreateItem),
    Update {
        id: String,
        input: UpdateItem,
    },
    Delete {
        id: String,
    },
    AddTags {
        id: String,
        tags: Vec<String>,
    },
    RemoveTags {
        id: String,
        tags: Vec<String>,
    },
    LinkExercise {
        piece_id: String,
        exercise_id: String,
    },
    UnlinkExercise {
        piece_id: String,
        exercise_id: String,
    },
    ReorderLinkedExercises {
        piece_id: String,
        ordered_ids: Vec<String>,
    },
    AddPieceWithScaffold {
        piece: CreateItem,
        scaffold: Vec<ScaffoldEntry>,
    },
}

/// One entry in a new piece's ordered exercise scaffold: create alongside the
/// piece, or link an exercise already in the library.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ScaffoldEntry {
    New(CreateItem),
    Existing { id: String },
}

/// Shared by Update / AddTags / RemoveTags.
fn save_or_put(model: &mut Model, item: Item) -> Command<Effect, Event> {
    if model.local_first {
        // No server callback to clear the dismiss-mute later (online does that
        // on ItemUpdated), so record the success here.
        model.record_success();
        Command::all([
            crate::persistence::save_item(item),
            crux_core::render::render(),
        ])
    } else {
        Command::all([
            crate::http::update_item(&model.api_base_url, &item),
            crux_core::render::render(),
        ])
    }
}

pub fn handle_item_event(event: ItemEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        ItemEvent::Add(input) => {
            let input = validation::normalize_create_item(input);
            if let Err(e) = validation::validate_create_item(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let now = chrono::Utc::now();
            let item = Item {
                id: ulid::Ulid::gen().to_string(),
                title: input.title,
                kind: input.kind,
                composer: input.composer,
                key: input.key,
                modality: input.modality,
                tempo: input.tempo,
                notes: input.notes,
                tags: input.tags,
                linked_exercise_ids: vec![],
                created_at: now,
                updated_at: now,
                priority: false,
            };

            model.items.push(item.clone());
            model.last_error = None;

            if model.local_first {
                // Client ulid is canonical, no temp-id replacement. No
                // ItemCreated callback to clear the dismiss-mute, so do it here.
                model.record_success();
                Command::all([
                    crate::persistence::save_item(item),
                    crux_core::render::render(),
                ])
            } else {
                let temp_id = item.id.clone();
                Command::all([
                    crate::http::create_item(&model.api_base_url, &item, &temp_id),
                    crux_core::render::render(),
                ])
            }
        }
        ItemEvent::Update { id, input } => {
            let input = validation::normalize_update_item(input);
            if let Err(e) = validation::validate_update_item(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            if let Some(title) = input.title {
                item.title = title;
            }
            if let Some(kind) = input.kind {
                item.kind = kind;
            }
            if let Some(composer) = input.composer {
                item.composer = composer;
            }
            if let Some(key) = input.key {
                item.key = key;
            }
            if let Some(modality) = input.modality {
                item.modality = modality;
            }
            if let Some(tempo) = input.tempo {
                item.tempo = tempo;
            }
            if let Some(notes) = input.notes {
                item.notes = notes;
            }
            if let Some(tags) = input.tags {
                item.tags = tags;
            }
            if let Some(priority) = input.priority {
                item.priority = priority;
            }
            item.updated_at = chrono::Utc::now();
            model.last_error = None;

            let item = item.clone();
            save_or_put(model, item)
        }
        ItemEvent::Delete { id } => {
            let len_before = model.items.len();
            model.items.retain(|i| i.id != id);
            if model.items.len() == len_before {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            }
            model.last_error = None;

            if model.local_first {
                model.record_success();
                Command::all([
                    crate::persistence::delete_item(id, chrono::Utc::now()),
                    crux_core::render::render(),
                ])
            } else {
                Command::all([
                    crate::http::delete_item(&model.api_base_url, &id),
                    crux_core::render::render(),
                ])
            }
        }
        ItemEvent::AddTags { id, tags } => {
            if let Err(e) = validation::validate_tags(&tags) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            for tag in tags {
                let tag_lower = tag.to_lowercase();
                if !item.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                    item.tags.push(tag);
                }
            }
            item.updated_at = chrono::Utc::now();
            model.last_error = None;

            let item = item.clone();
            save_or_put(model, item)
        }
        ItemEvent::RemoveTags { id, tags } => {
            let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            let tags_lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
            item.tags
                .retain(|t| !tags_lower.contains(&t.to_lowercase()));
            item.updated_at = chrono::Utc::now();
            model.last_error = None;

            let item = item.clone();
            save_or_put(model, item)
        }
        ItemEvent::LinkExercise {
            piece_id,
            exercise_id,
        } => {
            if let Err(e) = validation::validate_link_exercise(&piece_id, &exercise_id, model) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
                model.last_error = Some(LibraryError::NotFound { id: piece_id }.to_string());
                return crux_core::render::render();
            };

            if !piece.linked_exercise_ids.contains(&exercise_id) {
                piece.linked_exercise_ids.push(exercise_id);
            }
            piece.updated_at = chrono::Utc::now();
            model.last_error = None;

            let piece = piece.clone();
            save_or_put(model, piece)
        }
        ItemEvent::UnlinkExercise {
            piece_id,
            exercise_id,
        } => {
            let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
                model.last_error = Some(LibraryError::NotFound { id: piece_id }.to_string());
                return crux_core::render::render();
            };

            piece.linked_exercise_ids.retain(|id| id != &exercise_id);
            piece.updated_at = chrono::Utc::now();
            model.last_error = None;

            let piece = piece.clone();
            save_or_put(model, piece)
        }
        ItemEvent::ReorderLinkedExercises {
            piece_id,
            ordered_ids,
        } => {
            let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
                model.last_error = Some(LibraryError::NotFound { id: piece_id }.to_string());
                return crux_core::render::render();
            };

            let current = piece.linked_exercise_ids.clone();
            let current_set: std::collections::HashSet<&String> = current.iter().collect();
            let requested_set: std::collections::HashSet<&String> = ordered_ids.iter().collect();
            let mut next: Vec<String> = ordered_ids
                .iter()
                .filter(|id| current_set.contains(id))
                .cloned()
                .collect();
            for id in &current {
                if !requested_set.contains(id) {
                    next.push(id.clone());
                }
            }
            piece.linked_exercise_ids = next;
            piece.updated_at = chrono::Utc::now();
            model.last_error = None;

            let piece = piece.clone();
            save_or_put(model, piece)
        }
        ItemEvent::AddPieceWithScaffold { piece, scaffold } => {
            // Validate everything before mutating anything — the event is
            // atomic in the model: on any invalid part, nothing is applied.
            let piece_input = validation::normalize_create_item(piece);
            if let Err(e) = validation::validate_create_item(&piece_input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }
            if piece_input.kind != ItemKind::Piece {
                model.last_error = Some(
                    LibraryError::Validation {
                        field: "kind".to_string(),
                        message: "A lesson starts from a piece, not an exercise".to_string(),
                    }
                    .to_string(),
                );
                return crux_core::render::render();
            }

            let mut entries = Vec::with_capacity(scaffold.len());
            for entry in scaffold {
                match entry {
                    ScaffoldEntry::New(input) => {
                        let input = validation::normalize_create_item(input);
                        if let Err(e) = validation::validate_create_item(&input) {
                            model.last_error = Some(e.to_string());
                            return crux_core::render::render();
                        }
                        if input.kind != ItemKind::Exercise {
                            model.last_error = Some(
                                LibraryError::Validation {
                                    field: "scaffold".to_string(),
                                    message: "Scaffold entries must be exercises".to_string(),
                                }
                                .to_string(),
                            );
                            return crux_core::render::render();
                        }
                        entries.push(ScaffoldEntry::New(input));
                    }
                    ScaffoldEntry::Existing { id } => {
                        if let Err(e) = validation::validate_scaffold_link_target(&id, model) {
                            model.last_error = Some(e.to_string());
                            return crux_core::render::render();
                        }
                        entries.push(ScaffoldEntry::Existing { id });
                    }
                }
            }

            if !model.local_first {
                // Online items use the temp-id dance (server-assigned ids), which
                // can't remap a piece's linked_exercise_ids minted client-side —
                // the composite is local-first-only for now (#1080).
                model.last_error = Some(
                    LibraryError::Validation {
                        field: "scaffold".to_string(),
                        message: "Adding a lesson is not available online yet".to_string(),
                    }
                    .to_string(),
                );
                return crux_core::render::render();
            }

            let now = chrono::Utc::now();
            let mut new_items: Vec<Item> = Vec::new();
            let mut linked_ids: Vec<String> = Vec::new();
            for entry in entries {
                match entry {
                    ScaffoldEntry::New(input) => {
                        let item = Item {
                            id: ulid::Ulid::gen().to_string(),
                            title: input.title,
                            kind: input.kind,
                            composer: input.composer,
                            key: input.key,
                            modality: input.modality,
                            tempo: input.tempo,
                            notes: input.notes,
                            tags: input.tags,
                            linked_exercise_ids: vec![],
                            created_at: now,
                            updated_at: now,
                            priority: false,
                        };
                        linked_ids.push(item.id.clone());
                        new_items.push(item);
                    }
                    ScaffoldEntry::Existing { id } => {
                        if !linked_ids.contains(&id) {
                            linked_ids.push(id);
                        }
                    }
                }
            }

            let piece_item = Item {
                id: ulid::Ulid::gen().to_string(),
                title: piece_input.title,
                kind: ItemKind::Piece,
                composer: piece_input.composer,
                key: piece_input.key,
                modality: piece_input.modality,
                tempo: piece_input.tempo,
                notes: piece_input.notes,
                tags: piece_input.tags,
                linked_exercise_ids: linked_ids,
                created_at: now,
                updated_at: now,
                priority: false,
            };

            model.items.extend(new_items.iter().cloned());
            model.items.push(piece_item.clone());
            model.last_error = None;
            model.record_success();

            let mut cmds: Vec<Command<Effect, Event>> = new_items
                .into_iter()
                .map(crate::persistence::save_item)
                .collect();
            cmds.push(crate::persistence::save_item(piece_item));
            cmds.push(crux_core::render::render());
            Command::all(cmds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Intrada;
    use crate::model::Model;
    use crux_core::App;

    fn make_piece(id: &str) -> Item {
        let now = chrono::Utc::now();
        Item {
            id: id.to_string(),
            title: "Moonlight Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            linked_exercise_ids: vec![],
            created_at: now,
            updated_at: now,
            priority: false,
        }
    }

    fn make_exercise(id: &str) -> Item {
        let now = chrono::Utc::now();
        Item {
            id: id.to_string(),
            title: "C Major Scale".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            linked_exercise_ids: vec![],
            created_at: now,
            updated_at: now,
            priority: false,
        }
    }

    fn model_with_piece_and_exercise() -> Model {
        Model {
            items: vec![make_piece("piece-1"), make_exercise("ex-1")],
            local_first: true,
            api_base_url: "http://localhost:3001".to_string(),
            ..Default::default()
        }
    }

    fn send(model: &mut Model, event: ItemEvent) {
        let app = Intrada;
        let _cmd = app.update(crate::app::Event::Item(event), model);
    }

    // ── LinkExercise ──

    #[test]
    fn link_exercise_adds_id_and_bumps_updated_at() {
        let mut model = model_with_piece_and_exercise();
        let before = model
            .items
            .iter()
            .find(|i| i.id == "piece-1")
            .unwrap()
            .updated_at;

        send(
            &mut model,
            ItemEvent::LinkExercise {
                piece_id: "piece-1".to_string(),
                exercise_id: "ex-1".to_string(),
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert_eq!(piece.linked_exercise_ids, vec!["ex-1".to_string()]);
        assert!(piece.updated_at >= before);
        assert!(model.last_error.is_none());
    }

    #[test]
    fn link_exercise_rejects_nonexistent_exercise() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::LinkExercise {
                piece_id: "piece-1".to_string(),
                exercise_id: "no-such-id".to_string(),
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert!(piece.linked_exercise_ids.is_empty());
        assert!(model.last_error.is_some());
    }

    #[test]
    fn link_exercise_rejects_non_exercise_target() {
        let mut model = model_with_piece_and_exercise();
        // Add a second piece to try linking as exercise.
        model.items.push(make_piece("piece-2"));

        send(
            &mut model,
            ItemEvent::LinkExercise {
                piece_id: "piece-1".to_string(),
                exercise_id: "piece-2".to_string(),
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert!(piece.linked_exercise_ids.is_empty());
        assert!(model.last_error.is_some());
    }

    #[test]
    fn link_exercise_rejects_non_piece_host() {
        let mut model = model_with_piece_and_exercise();
        model.items.push(make_exercise("ex-2"));

        send(
            &mut model,
            ItemEvent::LinkExercise {
                piece_id: "ex-1".to_string(),
                exercise_id: "ex-2".to_string(),
            },
        );

        let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert!(ex.linked_exercise_ids.is_empty());
        assert!(model.last_error.is_some());
    }

    #[test]
    fn link_exercise_rejects_duplicate() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::LinkExercise {
                piece_id: "piece-1".to_string(),
                exercise_id: "ex-1".to_string(),
            },
        );
        assert!(model.last_error.is_none());

        send(
            &mut model,
            ItemEvent::LinkExercise {
                piece_id: "piece-1".to_string(),
                exercise_id: "ex-1".to_string(),
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert_eq!(piece.linked_exercise_ids.len(), 1);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn link_exercise_rejects_self_link() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::LinkExercise {
                piece_id: "piece-1".to_string(),
                exercise_id: "piece-1".to_string(),
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert!(piece.linked_exercise_ids.is_empty());
        assert!(model.last_error.is_some());
    }

    // ── UnlinkExercise ──

    #[test]
    fn unlink_exercise_removes_id() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::LinkExercise {
                piece_id: "piece-1".to_string(),
                exercise_id: "ex-1".to_string(),
            },
        );
        assert!(model.last_error.is_none());

        send(
            &mut model,
            ItemEvent::UnlinkExercise {
                piece_id: "piece-1".to_string(),
                exercise_id: "ex-1".to_string(),
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert!(piece.linked_exercise_ids.is_empty());
        assert!(model.last_error.is_none());
    }

    // ── ReorderLinkedExercises ──

    #[test]
    fn reorder_linked_exercises_sets_new_order() {
        let mut model = model_with_piece_and_exercise();
        model.items.push(make_exercise("ex-2"));

        send(
            &mut model,
            ItemEvent::LinkExercise {
                piece_id: "piece-1".to_string(),
                exercise_id: "ex-1".to_string(),
            },
        );
        send(
            &mut model,
            ItemEvent::LinkExercise {
                piece_id: "piece-1".to_string(),
                exercise_id: "ex-2".to_string(),
            },
        );

        send(
            &mut model,
            ItemEvent::ReorderLinkedExercises {
                piece_id: "piece-1".to_string(),
                ordered_ids: vec!["ex-2".to_string(), "ex-1".to_string()],
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert_eq!(
            piece.linked_exercise_ids,
            vec!["ex-2".to_string(), "ex-1".to_string()]
        );
        assert!(model.last_error.is_none());
    }

    #[test]
    fn reorder_linked_exercises_preserves_omitted_ids() {
        let mut model = model_with_piece_and_exercise();
        model.items.push(make_exercise("ex-2"));
        model.items.push(make_exercise("ex-3"));

        for ex in ["ex-1", "ex-2", "ex-3"] {
            send(
                &mut model,
                ItemEvent::LinkExercise {
                    piece_id: "piece-1".to_string(),
                    exercise_id: ex.to_string(),
                },
            );
        }

        send(
            &mut model,
            ItemEvent::ReorderLinkedExercises {
                piece_id: "piece-1".to_string(),
                ordered_ids: vec!["ex-3".to_string(), "ex-1".to_string()],
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert_eq!(
            piece.linked_exercise_ids,
            vec!["ex-3".to_string(), "ex-1".to_string(), "ex-2".to_string()]
        );
        assert!(model.last_error.is_none());
    }

    #[test]
    fn reorder_linked_exercises_ignores_foreign_ids() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::LinkExercise {
                piece_id: "piece-1".to_string(),
                exercise_id: "ex-1".to_string(),
            },
        );

        send(
            &mut model,
            ItemEvent::ReorderLinkedExercises {
                piece_id: "piece-1".to_string(),
                ordered_ids: vec!["ex-1".to_string(), "foreign-id".to_string()],
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert_eq!(piece.linked_exercise_ids, vec!["ex-1".to_string()]);
        assert!(model.last_error.is_none());
    }

    // ── AddPieceWithScaffold ──

    fn create_piece_input(title: &str) -> CreateItem {
        CreateItem {
            title: title.to_string(),
            kind: ItemKind::Piece,
            composer: Some("Roy Hargrove".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
        }
    }

    fn create_exercise_input(title: &str) -> CreateItem {
        CreateItem {
            title: title.to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
        }
    }

    fn send_cmd(
        model: &mut Model,
        event: ItemEvent,
    ) -> crux_core::Command<crate::app::Effect, crate::app::Event> {
        let app = Intrada;
        app.update(crate::app::Event::Item(event), model)
    }

    #[test]
    fn add_piece_with_scaffold_creates_piece_and_new_exercises_linked_in_order() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::AddPieceWithScaffold {
                piece: create_piece_input("Strasbourg / St. Denis "),
                scaffold: vec![
                    ScaffoldEntry::New(create_exercise_input("Learn the melody")),
                    ScaffoldEntry::Existing {
                        id: "ex-1".to_string(),
                    },
                    ScaffoldEntry::New(create_exercise_input("Enclosures")),
                ],
            },
        );

        assert!(model.last_error.is_none());
        // 2 pre-existing + piece + 2 new exercises
        assert_eq!(model.items.len(), 5);

        let piece = model
            .items
            .iter()
            .find(|i| i.title == "Strasbourg / St. Denis")
            .expect("piece created with normalized (trimmed) title");
        assert_eq!(piece.kind, ItemKind::Piece);

        let melody = model
            .items
            .iter()
            .find(|i| i.title == "Learn the melody")
            .expect("new exercise created");
        assert_eq!(melody.kind, ItemKind::Exercise);
        let enclosures = model
            .items
            .iter()
            .find(|i| i.title == "Enclosures")
            .expect("new exercise created");

        assert_eq!(
            piece.linked_exercise_ids,
            vec![melody.id.clone(), "ex-1".to_string(), enclosures.id.clone()],
            "scaffold order is the teacher's assignment order"
        );
    }

    #[test]
    fn add_piece_with_scaffold_rejects_invalid_new_entry_and_applies_nothing() {
        let mut model = model_with_piece_and_exercise();
        let count_before = model.items.len();

        send(
            &mut model,
            ItemEvent::AddPieceWithScaffold {
                piece: create_piece_input("Strasbourg / St. Denis"),
                scaffold: vec![
                    ScaffoldEntry::New(create_exercise_input("Valid one")),
                    ScaffoldEntry::New(create_exercise_input("   ")),
                ],
            },
        );

        assert!(model.last_error.is_some());
        assert_eq!(model.items.len(), count_before, "nothing applied");
    }

    #[test]
    fn add_piece_with_scaffold_rejects_unknown_existing_id_and_applies_nothing() {
        let mut model = model_with_piece_and_exercise();
        let count_before = model.items.len();

        send(
            &mut model,
            ItemEvent::AddPieceWithScaffold {
                piece: create_piece_input("Strasbourg / St. Denis"),
                scaffold: vec![ScaffoldEntry::Existing {
                    id: "no-such-id".to_string(),
                }],
            },
        );

        assert!(model.last_error.is_some());
        assert_eq!(model.items.len(), count_before);
    }

    #[test]
    fn add_piece_with_scaffold_rejects_existing_id_that_is_a_piece() {
        let mut model = model_with_piece_and_exercise();
        let count_before = model.items.len();

        send(
            &mut model,
            ItemEvent::AddPieceWithScaffold {
                piece: create_piece_input("Strasbourg / St. Denis"),
                scaffold: vec![ScaffoldEntry::Existing {
                    id: "piece-1".to_string(),
                }],
            },
        );

        assert!(model.last_error.is_some());
        assert_eq!(model.items.len(), count_before);
    }

    #[test]
    fn add_piece_with_scaffold_rejects_non_piece_kind_input() {
        let mut model = model_with_piece_and_exercise();
        let count_before = model.items.len();

        send(
            &mut model,
            ItemEvent::AddPieceWithScaffold {
                piece: create_exercise_input("Not a piece"),
                scaffold: vec![],
            },
        );

        assert!(model.last_error.is_some());
        assert_eq!(model.items.len(), count_before);
    }

    #[test]
    fn add_piece_with_scaffold_rejects_new_entry_with_piece_kind() {
        let mut model = model_with_piece_and_exercise();
        let count_before = model.items.len();

        send(
            &mut model,
            ItemEvent::AddPieceWithScaffold {
                piece: create_piece_input("Strasbourg / St. Denis"),
                scaffold: vec![ScaffoldEntry::New(create_piece_input("Nested piece"))],
            },
        );

        assert!(model.last_error.is_some());
        assert_eq!(model.items.len(), count_before);
    }

    #[test]
    fn add_piece_with_scaffold_dedupes_repeated_existing_ids() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::AddPieceWithScaffold {
                piece: create_piece_input("Strasbourg / St. Denis"),
                scaffold: vec![
                    ScaffoldEntry::Existing {
                        id: "ex-1".to_string(),
                    },
                    ScaffoldEntry::Existing {
                        id: "ex-1".to_string(),
                    },
                ],
            },
        );

        assert!(model.last_error.is_none());
        let piece = model
            .items
            .iter()
            .find(|i| i.title == "Strasbourg / St. Denis")
            .unwrap();
        assert_eq!(piece.linked_exercise_ids, vec!["ex-1".to_string()]);
    }

    #[test]
    fn add_piece_with_scaffold_errors_in_online_mode_and_applies_nothing() {
        let mut model = model_with_piece_and_exercise();
        model.local_first = false;
        let count_before = model.items.len();

        let mut cmd = send_cmd(
            &mut model,
            ItemEvent::AddPieceWithScaffold {
                piece: create_piece_input("Strasbourg / St. Denis"),
                scaffold: vec![ScaffoldEntry::New(create_exercise_input("Melody"))],
            },
        );

        assert!(model.last_error.is_some());
        assert_eq!(model.items.len(), count_before);
        assert!(
            !cmd.effects()
                .any(|e| matches!(e, crate::app::Effect::Http(_))),
            "online rejection must not fire HTTP"
        );
    }

    #[test]
    fn add_piece_with_scaffold_local_first_saves_all_items_piece_last_no_http() {
        let mut model = model_with_piece_and_exercise();

        let mut cmd = send_cmd(
            &mut model,
            ItemEvent::AddPieceWithScaffold {
                piece: create_piece_input("Strasbourg / St. Denis"),
                scaffold: vec![
                    ScaffoldEntry::New(create_exercise_input("Learn the melody")),
                    ScaffoldEntry::Existing {
                        id: "ex-1".to_string(),
                    },
                ],
            },
        );

        let mut saved_ids: Vec<String> = vec![];
        let mut saw_http = false;
        for effect in cmd.effects() {
            match effect {
                crate::app::Effect::Http(_) => saw_http = true,
                crate::app::Effect::Persistence(req) => {
                    if let crate::persistence::PersistenceOperation::SaveItem(item) = &req.operation
                    {
                        saved_ids.push(item.id.clone());
                    }
                }
                _ => {}
            }
        }

        assert!(
            !saw_http,
            "local-first path must be HTTP-free (invariant 1)"
        );
        let piece_id = model
            .items
            .iter()
            .find(|i| i.title == "Strasbourg / St. Denis")
            .unwrap()
            .id
            .clone();
        assert_eq!(saved_ids.len(), 2, "one new exercise + the piece");
        assert_eq!(
            saved_ids.last(),
            Some(&piece_id),
            "piece saved after its exercises"
        );
    }

    // FFI bincode round-trip (#846): mirrors `assert_round_trips` in types.rs —
    // ScaffoldEntry is a new bridge-crossing type.
    #[test]
    fn add_piece_with_scaffold_event_round_trips_on_ffi_bincode_wire() {
        use crux_core::bridge::{BincodeFfiFormat, FfiFormat};

        let event = ItemEvent::AddPieceWithScaffold {
            piece: create_piece_input("Strasbourg / St. Denis"),
            scaffold: vec![
                ScaffoldEntry::New(create_exercise_input("Learn the melody")),
                ScaffoldEntry::Existing {
                    id: "ex-1".to_string(),
                },
            ],
        };

        let mut bytes = Vec::new();
        BincodeFfiFormat::serialize(&mut bytes, &event).expect("serialize");
        let back: ItemEvent =
            BincodeFfiFormat::deserialize(&bytes).expect("must decode on the FFI wire (#846)");
        assert_eq!(event, back, "round-trip changed the value");
    }
}
