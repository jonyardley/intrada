use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::chart::{ChordChart, ScaffoldKind};
use super::types::{CreateItem, Tempo, UpdateItem};
use super::variant::Variant;
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
    /// Parsed chord changes (pieces only). Additive + local-first; rides the
    /// piece's `updated_at`. `None` for exercises and un-charted pieces.
    #[serde(default)]
    pub chord_chart: Option<ChordChart>,
    /// The exercise's step ladder (exercises only), tombstones included —
    /// persisted to the `variant` child table, not an item column (#1083).
    #[serde(default)]
    pub variants: Vec<Variant>,
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
    /// Parse `raw_chart` and store it on the piece. A parse error surfaces on
    /// `last_error` and stores nothing — never a partial chart.
    SetChordChart {
        piece_id: String,
        raw_chart: String,
    },
    ClearChordChart {
        piece_id: String,
    },
    /// Materialise the selected scaffold `kinds` into real exercises linked to
    /// the piece. The core re-derives from the stored chart (deterministic), so
    /// only the ticked `kind`s cross the wire — never spec content (#1106).
    /// Dedups by title against the piece's already-linked exercises; a batch
    /// with nothing new to add is a no-op.
    CommitScaffold {
        piece_id: String,
        kinds: Vec<ScaffoldKind>,
    },
    /// Define an exercise's whole step ladder from ordered `labels`,
    /// reconciled by case-insensitive label: matching variants keep their id
    /// (and score history), removed labels tombstone, re-added labels
    /// resurrect. Empty = clear the ladder. Local-first only until sync
    /// (#1083 C1).
    SetVariants {
        id: String,
        labels: Vec<String>,
    },
}

/// Shared by Update / AddTags / RemoveTags.
/// The reconciliation key shared by `CommitScaffold` and the preview's
/// `already_linked` flag: kinds (from the reserved tag, rename-robust) + titles
/// (guarding hand-made exercises) already linked to the piece.
pub(crate) fn linked_scaffold_state(
    model: &Model,
    piece_id: &str,
) -> (
    std::collections::HashSet<ScaffoldKind>,
    std::collections::HashSet<String>,
) {
    let linked_ids: std::collections::HashSet<&String> = model
        .items
        .iter()
        .find(|i| i.id == piece_id)
        .map(|p| p.linked_exercise_ids.iter().collect())
        .unwrap_or_default();
    let linked = model.items.iter().filter(|i| linked_ids.contains(&i.id));

    let mut kinds = std::collections::HashSet::new();
    let mut titles = std::collections::HashSet::new();
    for item in linked {
        titles.insert(item.title.to_lowercase());
        for tag in &item.tags {
            if let Some(kind) = ScaffoldKind::from_scaffold_tag(tag) {
                kinds.insert(kind);
            }
        }
    }
    (kinds, titles)
}

/// Whether a spec is already linked — by reserved kind (rename-robust) or a
/// hand-made title collision.
pub(crate) fn scaffold_already_linked(
    kinds: &std::collections::HashSet<ScaffoldKind>,
    titles: &std::collections::HashSet<String>,
    kind: ScaffoldKind,
    title: &str,
) -> bool {
    kinds.contains(&kind) || titles.contains(&title.to_lowercase())
}

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
                chord_chart: None,
                variants: vec![],
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
        ItemEvent::SetChordChart {
            piece_id,
            raw_chart,
        } => {
            if let Err(e) = validation::validate_chart_host(&piece_id, model) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            // The chart derives in the piece's key (default C major when unset).
            let (key, modality) = model
                .items
                .iter()
                .find(|i| i.id == piece_id)
                .map(|p| {
                    (
                        p.key.clone().unwrap_or_else(|| "C".to_string()),
                        p.modality.unwrap_or(Modality::Major),
                    )
                })
                .expect("validate_chart_host guarantees the piece exists");

            let chart = match super::chart::parse_chart(&raw_chart, &key, modality) {
                Ok(chart) => chart,
                Err(e) => {
                    // Surface the parse error; store nothing (never a partial).
                    model.last_error = Some(e.to_string());
                    return crux_core::render::render();
                }
            };

            let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
                model.last_error = Some(LibraryError::NotFound { id: piece_id }.to_string());
                return crux_core::render::render();
            };
            piece.chord_chart = Some(chart);
            piece.updated_at = chrono::Utc::now();
            model.last_error = None;

            let piece = piece.clone();
            save_or_put(model, piece)
        }
        ItemEvent::ClearChordChart { piece_id } => {
            let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
                model.last_error = Some(LibraryError::NotFound { id: piece_id }.to_string());
                return crux_core::render::render();
            };
            piece.chord_chart = None;
            piece.updated_at = chrono::Utc::now();
            model.last_error = None;

            let piece = piece.clone();
            save_or_put(model, piece)
        }
        ItemEvent::SetVariants { id, labels } => {
            if !model.local_first {
                // Steps are local-first-only until sync (#1083; invariant 6
                // consciously scoped) — surfaced, never a silent no-op.
                model.last_error = Some("Steps aren't available online yet".to_string());
                return crux_core::render::render();
            }

            let labels = validation::normalize_variant_labels(labels);
            if let Err(e) = validation::validate_variant_host(&id, model)
                .and_then(|()| validation::validate_variant_labels(&labels))
            {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            let now = chrono::Utc::now();
            item.variants = super::variant::reconcile_variants(
                std::mem::take(&mut item.variants),
                &labels,
                now,
            );
            item.updated_at = now;
            model.last_error = None;

            let item = item.clone();
            save_or_put(model, item)
        }
        ItemEvent::CommitScaffold { piece_id, kinds } => {
            if let Err(e) = validation::validate_chart_host(&piece_id, model) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            // Re-derive from the stored chart — deterministic, so the committed
            // exercises equal the previewed ones.
            let Some(chart) = model
                .items
                .iter()
                .find(|i| i.id == piece_id)
                .and_then(|p| p.chord_chart.clone())
            else {
                model.last_error = Some(
                    LibraryError::Validation {
                        field: "piece_id".to_string(),
                        message: "This piece has no chord chart to build from".to_string(),
                    }
                    .to_string(),
                );
                return crux_core::render::render();
            };

            // Skip specs already linked (by reserved kind or hand-made title) —
            // same predicate the preview's `already_linked` flag uses.
            let (linked_kinds, linked_titles) = linked_scaffold_state(model, &piece_id);

            let selected: std::collections::HashSet<ScaffoldKind> = kinds.into_iter().collect();
            let now = chrono::Utc::now();
            let new_exercises: Vec<Item> = super::chart::derive_scaffold(&chart)
                .into_iter()
                .filter(|s| selected.contains(&s.kind))
                .filter(|s| {
                    !scaffold_already_linked(&linked_kinds, &linked_titles, s.kind, &s.title)
                })
                .map(|s| Item {
                    id: ulid::Ulid::gen().to_string(),
                    title: s.title,
                    kind: ItemKind::Exercise,
                    composer: None,
                    key: Some(s.key),
                    modality: None,
                    tempo: None,
                    notes: Some(s.rationale),
                    tags: vec![s.kind.scaffold_tag()],
                    linked_exercise_ids: vec![],
                    created_at: now,
                    updated_at: now,
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                })
                .collect();

            if new_exercises.is_empty() {
                // Everything deselected or already linked — a benign no-op, not
                // an error, and nothing to persist.
                model.last_error = None;
                return crux_core::render::render();
            }

            let new_ids: Vec<String> = new_exercises.iter().map(|e| e.id.clone()).collect();

            let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
                model.last_error = Some(LibraryError::NotFound { id: piece_id }.to_string());
                return crux_core::render::render();
            };
            piece.linked_exercise_ids.extend(new_ids);
            piece.updated_at = now;
            let piece = piece.clone();
            model.last_error = None;

            model.items.extend(new_exercises.iter().cloned());

            if model.local_first {
                // No server callback to clear the dismiss-mute, so record here.
                model.record_success();
                let mut batch = new_exercises;
                batch.push(piece);
                Command::all([
                    crate::persistence::save_items(batch),
                    crux_core::render::render(),
                ])
            } else {
                // Online batch is deferred with the web/API work (invariant 6):
                // compiles against existing plumbing, non-atomic, untested path.
                // FIXME(#1108): these links use the client ulid, but create_item
                // is the temp-id path where the server reassigns the id — the
                // online links would dangle. Reconcile ids before wiring web.
                let mut cmds: Vec<Command<Effect, Event>> = new_exercises
                    .iter()
                    .map(|ex| crate::http::create_item(&model.api_base_url, ex, &ex.id))
                    .collect();
                cmds.push(crate::http::update_item(&model.api_base_url, &piece));
                cmds.push(crux_core::render::render());
                Command::all(cmds)
            }
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
            chord_chart: None,
            variants: vec![],
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
            chord_chart: None,
            variants: vec![],
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

    fn send_cmd(
        model: &mut Model,
        event: ItemEvent,
    ) -> crux_core::Command<crate::app::Effect, crate::app::Event> {
        let app = Intrada;
        app.update(crate::app::Event::Item(event), model)
    }

    fn emits_http(cmd: &mut crux_core::Command<crate::app::Effect, crate::app::Event>) -> bool {
        cmd.effects()
            .any(|e| matches!(e, crate::app::Effect::Http(_)))
    }

    fn emits_save(
        cmd: &mut crux_core::Command<crate::app::Effect, crate::app::Event>,
        id: &str,
    ) -> bool {
        cmd.effects().any(|e| {
            matches!(e, crate::app::Effect::Persistence(req)
                if matches!(&req.operation, crate::persistence::PersistenceOperation::SaveItem(item) if item.id == id))
        })
    }

    // ── SetChordChart / ClearChordChart ──

    #[test]
    fn set_chord_chart_parses_stores_and_persists_without_http() {
        let mut model = model_with_piece_and_exercise();
        let before = model
            .items
            .iter()
            .find(|i| i.id == "piece-1")
            .unwrap()
            .updated_at;

        let mut cmd = send_cmd(
            &mut model,
            ItemEvent::SetChordChart {
                piece_id: "piece-1".to_string(),
                raw_chart: "| Cm7 | F7 | Bbmaj7 |".to_string(),
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        let chart = piece.chord_chart.as_ref().expect("chart stored");
        assert_eq!(chart.changes().len(), 3);
        assert!(piece.updated_at >= before);
        assert!(model.last_error.is_none());
        assert!(
            emits_save(&mut cmd, "piece-1"),
            "local-first persists the piece"
        );
        assert!(
            !emits_http(&mut cmd),
            "local-first stores no HTTP (invariant 1)"
        );
    }

    #[test]
    fn set_chord_chart_parse_error_surfaces_and_stores_nothing() {
        let mut model = model_with_piece_and_exercise();

        let mut cmd = send_cmd(
            &mut model,
            ItemEvent::SetChordChart {
                piece_id: "piece-1".to_string(),
                raw_chart: "| Cm7 | Hm7b5 |".to_string(),
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert!(
            piece.chord_chart.is_none(),
            "no partial chart on parse error"
        );
        let err = model.last_error.as_deref().expect("parse error surfaced");
        assert!(
            err.contains("Bar 2"),
            "error names the offending bar: {err}"
        );
        assert!(
            !emits_save(&mut cmd, "piece-1"),
            "nothing persisted on error"
        );
    }

    #[test]
    fn set_chord_chart_rejects_a_non_piece_host() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::SetChordChart {
                piece_id: "ex-1".to_string(),
                raw_chart: "| Cm7 |".to_string(),
            },
        );

        let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert!(ex.chord_chart.is_none());
        assert!(model.last_error.is_some());
    }

    #[test]
    fn set_chord_chart_uses_the_piece_key() {
        let mut model = model_with_piece_and_exercise();
        if let Some(p) = model.items.iter_mut().find(|i| i.id == "piece-1") {
            p.key = Some("G".to_string());
            p.modality = Some(Modality::Minor);
        }

        send(
            &mut model,
            ItemEvent::SetChordChart {
                piece_id: "piece-1".to_string(),
                raw_chart: "| Cm7 |".to_string(),
            },
        );

        let chart = model
            .items
            .iter()
            .find(|i| i.id == "piece-1")
            .unwrap()
            .chord_chart
            .as_ref()
            .unwrap();
        assert_eq!(chart.key, "G");
        assert_eq!(chart.modality, Modality::Minor);
    }

    #[test]
    fn clear_chord_chart_removes_it() {
        let mut model = model_with_piece_and_exercise();
        send(
            &mut model,
            ItemEvent::SetChordChart {
                piece_id: "piece-1".to_string(),
                raw_chart: "| Cm7 |".to_string(),
            },
        );
        assert!(model.items[0].chord_chart.is_some());

        send(
            &mut model,
            ItemEvent::ClearChordChart {
                piece_id: "piece-1".to_string(),
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert!(piece.chord_chart.is_none());
        assert!(model.last_error.is_none());
    }

    // ── CommitScaffold ──

    use super::ScaffoldKind;

    fn emits_save_items(
        cmd: &mut crux_core::Command<crate::app::Effect, crate::app::Event>,
    ) -> Option<Vec<String>> {
        cmd.effects().find_map(|e| match e {
            crate::app::Effect::Persistence(req) => match req.operation {
                crate::persistence::PersistenceOperation::SaveItems(items) => {
                    Some(items.iter().map(|i| i.id.clone()).collect())
                }
                _ => None,
            },
            _ => None,
        })
    }

    fn charted_model() -> Model {
        let mut model = model_with_piece_and_exercise();
        send(
            &mut model,
            ItemEvent::SetChordChart {
                piece_id: "piece-1".to_string(),
                raw_chart: "| Cm7 | F7 | Bbmaj7 |".to_string(),
            },
        );
        model
    }

    fn exercise_titles(model: &Model) -> Vec<String> {
        model
            .items
            .iter()
            .filter(|i| i.kind == ItemKind::Exercise)
            .map(|i| i.title.clone())
            .collect()
    }

    #[test]
    fn commit_scaffold_creates_selected_exercises_links_them_and_persists_a_batch() {
        let mut model = charted_model();

        let mut cmd = send_cmd(
            &mut model,
            ItemEvent::CommitScaffold {
                piece_id: "piece-1".to_string(),
                kinds: vec![ScaffoldKind::Shells, ScaffoldKind::GuideToneLines],
            },
        );

        let new: Vec<&Item> = model
            .items
            .iter()
            .filter(|i| i.kind == ItemKind::Exercise && i.id != "ex-1")
            .collect();
        assert_eq!(new.len(), 2, "two ticked kinds create two exercises");
        let titles: std::collections::HashSet<&str> =
            new.iter().map(|e| e.title.as_str()).collect();
        assert!(titles.contains("Shells") && titles.contains("Guide-tone lines"));
        assert!(
            new.iter().all(|e| e.key.as_deref() == Some("C")),
            "exercises carry the chart's key"
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        for e in &new {
            assert!(
                piece.linked_exercise_ids.contains(&e.id),
                "each new exercise is linked to the piece"
            );
        }
        assert!(model.last_error.is_none());

        let batch = emits_save_items(&mut cmd).expect("a SaveItems batch is persisted");
        assert_eq!(batch.len(), 3, "two exercises + the piece, one transaction");
        assert!(batch.contains(&"piece-1".to_string()));
        assert!(
            !emits_http(&mut cmd),
            "local-first commit makes no HTTP (invariant 1)"
        );
    }

    #[test]
    fn commit_scaffold_dedups_on_rerun_no_duplicates() {
        let mut model = charted_model();
        let kinds = vec![ScaffoldKind::Shells];

        send(
            &mut model,
            ItemEvent::CommitScaffold {
                piece_id: "piece-1".to_string(),
                kinds: kinds.clone(),
            },
        );
        let after_first = exercise_titles(&model).len();

        let mut cmd = send_cmd(
            &mut model,
            ItemEvent::CommitScaffold {
                piece_id: "piece-1".to_string(),
                kinds,
            },
        );

        assert_eq!(
            exercise_titles(&model).len(),
            after_first,
            "re-committing the same kind adds no duplicate"
        );
        assert!(
            emits_save_items(&mut cmd).is_none(),
            "a no-op commit persists nothing"
        );
        assert!(model.last_error.is_none(), "a no-op commit is not an error");
    }

    #[test]
    fn commit_scaffold_does_not_clobber_a_handmade_exercise_of_the_same_title() {
        let mut model = charted_model();
        // A hand-made "Shells" already linked to the piece.
        let mut handmade = make_exercise("handmade-shells");
        handmade.title = "Shells".to_string();
        handmade.notes = Some("my own".to_string());
        model.items.push(handmade);
        if let Some(piece) = model.items.iter_mut().find(|i| i.id == "piece-1") {
            piece
                .linked_exercise_ids
                .push("handmade-shells".to_string());
        }

        send(
            &mut model,
            ItemEvent::CommitScaffold {
                piece_id: "piece-1".to_string(),
                kinds: vec![ScaffoldKind::Shells],
            },
        );

        let shells: Vec<&Item> = model.items.iter().filter(|i| i.title == "Shells").collect();
        assert_eq!(shells.len(), 1, "no duplicate 'Shells' created");
        assert_eq!(
            shells[0].id, "handmade-shells",
            "the hand-made exercise is untouched"
        );
        assert_eq!(shells[0].notes.as_deref(), Some("my own"));
    }

    #[test]
    fn commit_scaffold_reconciles_after_rename_via_reserved_tag() {
        // Regenerate-on-edit robustness: a generated exercise the user renamed
        // still carries its reserved scaffold tag, so re-committing reconciles by
        // kind and doesn't create a second copy (title-only dedup would duplicate).
        let mut model = charted_model();
        send(
            &mut model,
            ItemEvent::CommitScaffold {
                piece_id: "piece-1".to_string(),
                kinds: vec![ScaffoldKind::Shells],
            },
        );
        let shells_id = model
            .items
            .iter()
            .find(|i| i.tags.contains(&ScaffoldKind::Shells.scaffold_tag()))
            .expect("a tagged Shells exercise was created")
            .id
            .clone();
        model
            .items
            .iter_mut()
            .find(|i| i.id == shells_id)
            .unwrap()
            .title = "3rds & 7ths".to_string();

        send(
            &mut model,
            ItemEvent::CommitScaffold {
                piece_id: "piece-1".to_string(),
                kinds: vec![ScaffoldKind::Shells],
            },
        );

        let tagged = model
            .items
            .iter()
            .filter(|i| i.tags.contains(&ScaffoldKind::Shells.scaffold_tag()))
            .count();
        assert_eq!(tagged, 1, "the renamed generated exercise isn't duplicated");
    }

    #[test]
    fn committed_exercise_tag_is_hidden_from_the_view_and_vocabulary() {
        let mut model = charted_model();
        send(
            &mut model,
            ItemEvent::CommitScaffold {
                piece_id: "piece-1".to_string(),
                kinds: vec![ScaffoldKind::Shells],
            },
        );
        let shells = model
            .items
            .iter()
            .find(|i| i.tags.contains(&ScaffoldKind::Shells.scaffold_tag()))
            .expect("the committed exercise carries the reserved tag");

        let vm = Intrada.view(&model);
        let shells_view = vm.items.iter().find(|v| v.id == shells.id).unwrap();
        assert!(
            shells_view.tags.iter().all(|t| !t.starts_with("scaffold:")),
            "no reserved tag reaches the item view"
        );
        assert!(
            vm.available_tags
                .iter()
                .all(|t| !t.starts_with("scaffold:")),
            "no reserved tag reaches the tag vocabulary"
        );
    }

    #[test]
    fn commit_scaffold_without_a_chart_surfaces_an_error() {
        let mut model = model_with_piece_and_exercise(); // no chart set

        let mut cmd = send_cmd(
            &mut model,
            ItemEvent::CommitScaffold {
                piece_id: "piece-1".to_string(),
                kinds: vec![ScaffoldKind::Shells],
            },
        );

        assert!(
            model.last_error.is_some(),
            "no chart is surfaced, not silent"
        );
        assert!(emits_save_items(&mut cmd).is_none(), "nothing persisted");
        assert_eq!(exercise_titles(&model), vec!["C Major Scale".to_string()]);
    }

    #[test]
    fn commit_scaffold_rejects_a_non_piece_host() {
        let mut model = charted_model();

        send(
            &mut model,
            ItemEvent::CommitScaffold {
                piece_id: "ex-1".to_string(),
                kinds: vec![ScaffoldKind::Shells],
            },
        );

        assert!(model.last_error.is_some());
    }

    #[test]
    fn commit_scaffold_empty_selection_is_a_benign_noop() {
        let mut model = charted_model();
        let before = exercise_titles(&model).len();

        let mut cmd = send_cmd(
            &mut model,
            ItemEvent::CommitScaffold {
                piece_id: "piece-1".to_string(),
                kinds: vec![],
            },
        );

        assert_eq!(exercise_titles(&model).len(), before, "nothing created");
        assert!(emits_save_items(&mut cmd).is_none());
        assert!(model.last_error.is_none());
    }

    // ── SetVariants ──

    #[test]
    fn set_variants_creates_a_ladder_and_persists_locally() {
        let mut model = model_with_piece_and_exercise();

        let mut cmd = send_cmd(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string(), "F".to_string(), "Bb".to_string()],
            },
        );

        let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(ex.variants.len(), 3);
        assert!(ex.variants.iter().all(|v| v.deleted_at.is_none()));
        assert_eq!(
            ex.variants
                .iter()
                .map(|v| v.label.as_str())
                .collect::<Vec<_>>(),
            vec!["C", "F", "Bb"]
        );
        assert_eq!(
            ex.variants.iter().map(|v| v.position).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert!(model.last_error.is_none());
        assert!(
            emits_save(&mut cmd, "ex-1"),
            "local-first persists the exercise"
        );
        assert!(!emits_http(&mut cmd), "no HTTP (invariant 1)");
    }

    #[test]
    fn set_variants_reorder_preserves_ids_by_label() {
        let mut model = model_with_piece_and_exercise();
        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string(), "F".to_string(), "Bb".to_string()],
            },
        );
        let ids_by_label: std::collections::HashMap<String, String> = model
            .items
            .iter()
            .find(|i| i.id == "ex-1")
            .unwrap()
            .variants
            .iter()
            .map(|v| (v.label.clone(), v.id.clone()))
            .collect();

        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["Bb".to_string(), "C".to_string(), "F".to_string()],
            },
        );

        let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(ex.variants.len(), 3);
        for v in &ex.variants {
            assert_eq!(
                ids_by_label.get(&v.label),
                Some(&v.id),
                "reordering keeps each step's id (and so its history)"
            );
        }
        assert_eq!(
            ex.variants
                .iter()
                .map(|v| (v.label.as_str(), v.position))
                .collect::<Vec<_>>(),
            vec![("Bb", 0), ("C", 1), ("F", 2)]
        );
    }

    #[test]
    fn set_variants_removing_a_label_tombstones_never_hard_deletes() {
        let mut model = model_with_piece_and_exercise();
        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string(), "F".to_string()],
            },
        );

        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string()],
            },
        );

        let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
        let live: Vec<_> = ex
            .variants
            .iter()
            .filter(|v| v.deleted_at.is_none())
            .collect();
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].label, "C");
        let dead: Vec<_> = ex
            .variants
            .iter()
            .filter(|v| v.deleted_at.is_some())
            .collect();
        assert_eq!(dead.len(), 1, "the removed step is kept as a tombstone");
        assert_eq!(dead[0].label, "F");
    }

    #[test]
    fn set_variants_readding_a_label_resurrects_its_tombstone() {
        let mut model = model_with_piece_and_exercise();
        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string(), "F".to_string()],
            },
        );
        let f_id = model
            .items
            .iter()
            .find(|i| i.id == "ex-1")
            .unwrap()
            .variants
            .iter()
            .find(|v| v.label == "F")
            .unwrap()
            .id
            .clone();

        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string()],
            },
        );
        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string(), "F".to_string()],
            },
        );

        let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(
            ex.variants.len(),
            2,
            "no duplicate row for the re-added label"
        );
        let f = ex.variants.iter().find(|v| v.label == "F").unwrap();
        assert_eq!(
            f.id, f_id,
            "the re-added step resurrects its old id (score history intact)"
        );
        assert!(f.deleted_at.is_none());
        assert_eq!(f.position, 1);
    }

    fn exercise_variants(model: &Model) -> &Vec<crate::domain::variant::Variant> {
        &model
            .items
            .iter()
            .find(|i| i.id == "ex-1")
            .unwrap()
            .variants
    }

    #[test]
    fn set_variants_rejects_a_piece_host() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "piece-1".to_string(),
                labels: vec!["C".to_string()],
            },
        );

        let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert!(piece.variants.is_empty(), "a piece never gets a ladder");
        assert!(model.last_error.is_some());
    }

    #[test]
    fn set_variants_rejects_duplicate_labels_case_insensitively() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string(), "c".to_string()],
            },
        );

        assert!(exercise_variants(&model).is_empty());
        assert!(model.last_error.is_some());
    }

    #[test]
    fn set_variants_rejects_a_blank_label() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string(), "   ".to_string()],
            },
        );

        assert!(exercise_variants(&model).is_empty());
        assert!(model.last_error.is_some());
    }

    #[test]
    fn set_variants_trims_labels() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["  C  ".to_string()],
            },
        );

        assert_eq!(exercise_variants(&model)[0].label, "C");
        assert!(model.last_error.is_none());
    }

    #[test]
    fn set_variants_rejects_more_than_the_cap() {
        let mut model = model_with_piece_and_exercise();
        let labels: Vec<String> = (0..=validation::MAX_VARIANTS)
            .map(|i| i.to_string())
            .collect();

        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels,
            },
        );

        assert!(exercise_variants(&model).is_empty());
        assert!(model.last_error.is_some());
    }

    #[test]
    fn set_variants_rejects_an_overlong_label() {
        let mut model = model_with_piece_and_exercise();

        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["x".repeat(validation::MAX_VARIANT_LABEL + 1)],
            },
        );

        assert!(exercise_variants(&model).is_empty());
        assert!(model.last_error.is_some());
    }

    #[test]
    fn set_variants_empty_list_clears_the_ladder() {
        let mut model = model_with_piece_and_exercise();
        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string(), "F".to_string()],
            },
        );

        let mut cmd = send_cmd(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec![],
            },
        );

        let variants = exercise_variants(&model);
        assert_eq!(variants.len(), 2, "cleared steps remain as tombstones");
        assert!(variants.iter().all(|v| v.deleted_at.is_some()));
        assert!(model.last_error.is_none());
        assert!(emits_save(&mut cmd, "ex-1"), "the clear persists");
    }

    #[test]
    fn set_variants_untouched_step_keeps_its_updated_at() {
        // Per-row LWW hygiene: only rows that changed get a new timestamp.
        let mut model = model_with_piece_and_exercise();
        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string(), "F".to_string()],
            },
        );
        let c_updated_at = exercise_variants(&model)[0].updated_at;

        send(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string(), "G".to_string()],
            },
        );

        let variants = exercise_variants(&model);
        let c = variants.iter().find(|v| v.label == "C").unwrap();
        assert_eq!(
            c.updated_at, c_updated_at,
            "an untouched step keeps its LWW timestamp"
        );
        let f = variants.iter().find(|v| v.label == "F").unwrap();
        assert!(
            f.updated_at > c_updated_at,
            "the tombstoned step is stamped"
        );
    }

    #[test]
    fn set_variants_online_mode_is_scoped_out_gracefully() {
        // Invariant 6 consciously scoped (#1083): steps are local-first-only
        // until sync. Online must surface that — never mutate, never POST.
        let mut model = model_with_piece_and_exercise();
        model.local_first = false;

        let mut cmd = send_cmd(
            &mut model,
            ItemEvent::SetVariants {
                id: "ex-1".to_string(),
                labels: vec!["C".to_string()],
            },
        );

        assert!(exercise_variants(&model).is_empty(), "model unchanged");
        assert!(
            model.last_error.is_some(),
            "scope-out is surfaced, not silent"
        );
        assert!(!emits_http(&mut cmd), "no HTTP");
        assert!(!emits_save(&mut cmd, "ex-1"), "nothing persisted");
    }

    // ── Bridge round-trip for the write events (#846) ──

    #[test]
    fn chord_chart_events_round_trip_on_the_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(crate::app::Event::Item(
            ItemEvent::SetChordChart {
                piece_id: "P".to_string(),
                raw_chart: "| Cm7 | F7 |".to_string(),
            },
        ));
        crate::domain::types::assert_round_trips(crate::app::Event::Item(
            ItemEvent::ClearChordChart {
                piece_id: "P".to_string(),
            },
        ));
        crate::domain::types::assert_round_trips(crate::app::Event::Item(
            ItemEvent::CommitScaffold {
                piece_id: "P".to_string(),
                kinds: vec![ScaffoldKind::Shells, ScaffoldKind::ScalesToChordTones],
            },
        ));
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
}
