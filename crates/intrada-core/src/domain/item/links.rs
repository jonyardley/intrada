use super::*;
use crate::domain::link::{reconcile_links, same_links, ExerciseLink, WantedLink};

/// Links to `ids` after the piece's last, skipping any already live.
pub(super) fn append_links(piece: &mut Item, ids: &[String], now: chrono::DateTime<chrono::Utc>) {
    for id in ids {
        if !piece.has_live_link(id) {
            let position = piece.next_link_position();
            piece
                .exercise_links
                .push(ExerciseLink::new(id.clone(), position, now));
        }
    }
}

pub(super) fn commit_scaffold(
    model: &mut Model,
    piece_id: String,
    kinds: Vec<ScaffoldKind>,
) -> Command<Effect, Event> {
    if let Err(e) = validation::validate_chart_host(&piece_id, model) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    // Re-derive from the stored chart: deterministic, so the committed
    // exercises equal the previewed ones.
    let Some(chart) = model
        .items
        .iter()
        .find(|i| i.id == piece_id)
        .and_then(|p| p.chord_chart.clone())
    else {
        model.raise_error(
            LibraryError::Validation {
                field: "piece_id".to_string(),
                message: "This piece has no chord chart to build from".to_string(),
            }
            .to_string(),
        );
        return crux_core::render::render();
    };

    // Skip specs already linked (by reserved kind or hand-made title),
    // same predicate the preview's `already_linked` flag uses.
    let (linked_kinds, linked_titles) = linked_scaffold_state(model, &piece_id);

    let selected: std::collections::HashSet<ScaffoldKind> = kinds.into_iter().collect();
    let now = chrono::Utc::now();
    let new_exercises: Vec<Item> = crate::domain::chart::derive_scaffold(&chart)
        .into_iter()
        .filter(|s| selected.contains(&s.kind))
        .filter(|s| !scaffold_already_linked(&linked_kinds, &linked_titles, s.kind, &s.title))
        .map(|s| Item {
            id: ulid::Ulid::generate().to_string(),
            title: s.title,
            kind: ItemKind::Exercise,
            composer: None,
            key: s.key,
            tempo: None,
            notes: Some(s.rationale),
            tags: vec![s.kind.scaffold_tag()],
            exercise_links: vec![],
            created_at: now,
            updated_at: now,
            priority: false,
            chord_chart: None,
            variation_ids: vec![],
            keys: vec![],
            photo_id: None,
            metre: None,
        })
        .collect();

    if new_exercises.is_empty() {
        // Everything deselected or already linked: a benign no-op, not
        // an error, and nothing to persist.
        model.last_error = None;
        return crux_core::render::render();
    }

    let new_ids: Vec<String> = new_exercises.iter().map(|e| e.id.clone()).collect();

    let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
        model.raise_error(LibraryError::NotFound { id: piece_id }.to_string());
        return crux_core::render::render();
    };
    append_links(piece, &new_ids, now);
    piece.updated_at = now;
    let piece = piece.clone();

    model.items.extend(new_exercises.iter().cloned());

    model.clear_error();
    let mut batch = new_exercises;
    batch.push(piece);
    Command::all([
        crate::persistence::save_items(model, batch),
        crux_core::render::render(),
    ])
}

pub(super) fn set_piece_links(
    model: &mut Model,
    piece_id: String,
    links: Vec<ScaffoldEntry>,
) -> Command<Effect, Event> {
    if let Err(e) = validation::validate_piece_host(&piece_id, model) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let mut rows = Vec::with_capacity(links.len());
    for (index, exercise) in links.into_iter().enumerate() {
        let Some(entry) = super::create::validate_entry(model, index, exercise) else {
            return crux_core::render::render();
        };
        rows.push(entry);
    }

    let now = chrono::Utc::now();
    let mut created: Vec<Item> = Vec::new();
    let mut wanted: Vec<WantedLink> = Vec::new();
    for entry in rows {
        let exercise_id = match entry {
            super::create::Entry::New(input, tempo) => {
                let exercise = super::create::new_exercise(*input, tempo, now);
                let id = exercise.id.clone();
                created.push(exercise);
                id
            }
            super::create::Entry::Existing(id) => id,
        };
        if !wanted.iter().any(|w| w.exercise_id == exercise_id) {
            let position = Some(wanted.len());
            wanted.push(WantedLink {
                exercise_id,
                position,
            });
        }
    }

    let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
        model.raise_error(LibraryError::NotFound { id: piece_id }.to_string());
        return crux_core::render::render();
    };
    let stored: Vec<&str> = piece
        .live_links()
        .into_iter()
        .map(|l| l.exercise_id.as_str())
        .collect();
    let asked: Vec<&str> = wanted.iter().map(|w| w.exercise_id.as_str()).collect();
    if created.is_empty() && stored == asked {
        model.last_error = None;
        return crux_core::render::render();
    }
    piece.exercise_links = reconcile_links(&piece.exercise_links, |_| true, wanted, now);
    piece.updated_at = now;
    let piece = piece.clone();

    if created.is_empty() {
        return persist_item(model, piece);
    }
    model.items.extend(created.iter().cloned());
    model.clear_error();
    let mut batch = created;
    batch.push(piece);
    Command::all([
        crate::persistence::save_items(model, batch),
        crux_core::render::render(),
    ])
}

pub(super) fn choose_exercise_pieces(
    model: &mut Model,
    exercise_id: String,
    piece_ids: Vec<String>,
) -> Command<Effect, Event> {
    if let Err(e) = validation::validate_exercise_link_target(&exercise_id, model) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }
    let mut wanted: Vec<String> = Vec::with_capacity(piece_ids.len());
    for piece_id in piece_ids {
        if let Err(e) = validation::validate_piece_host(&piece_id, model) {
            model.raise_error(e.to_string());
            return crux_core::render::render();
        }
        if !wanted.contains(&piece_id) {
            wanted.push(piece_id);
        }
    }

    let now = chrono::Utc::now();
    let mut changed: Vec<Item> = Vec::new();
    for piece in model.items.iter_mut().filter(|i| i.kind == ItemKind::Piece) {
        let mine: Vec<WantedLink> = wanted
            .iter()
            .filter(|id| **id == piece.id)
            .map(|_| WantedLink {
                exercise_id: exercise_id.clone(),
                position: None,
            })
            .collect();
        let next = reconcile_links(
            &piece.exercise_links,
            |l| l.exercise_id == exercise_id,
            mine,
            now,
        );
        if !same_links(&piece.exercise_links, &next) {
            piece.exercise_links = next;
            piece.updated_at = now;
            changed.push(piece.clone());
        }
    }

    if changed.is_empty() {
        model.last_error = None;
        return crux_core::render::render();
    }
    model.clear_error();
    Command::all([
        crate::persistence::save_items(model, changed),
        crux_core::render::render(),
    ])
}

pub(super) fn choose_piece_exercises(
    model: &mut Model,
    piece_id: String,
    exercise_ids: Vec<String>,
    written: Vec<CreateItem>,
) -> Command<Effect, Event> {
    let stored: Vec<String> = model
        .items
        .iter()
        .find(|i| i.id == piece_id)
        .map(Item::linked_exercise_ids)
        .unwrap_or_default();
    let kept = stored
        .iter()
        .filter(|id| exercise_ids.contains(id))
        .cloned();
    let added = exercise_ids
        .iter()
        .filter(|id| !stored.contains(id))
        .cloned();
    let links = kept
        .chain(added)
        .map(|id| ScaffoldEntry::Existing { id })
        .chain(written.into_iter().map(ScaffoldEntry::New))
        .collect();
    set_piece_links(model, piece_id, links)
}

/// Works on the card: each linked exercise still in the library, in link order.
pub(super) fn change_piece_link(
    model: &mut Model,
    piece_id: String,
    change: LinkChange,
) -> Command<Effect, Event> {
    let Some(piece) = model.items.iter().find(|i| i.id == piece_id) else {
        model.raise_error(LibraryError::NotFound { id: piece_id }.to_string());
        return crux_core::render::render();
    };
    // A deleted exercise leaves its link row behind; the card the musician
    // sees skips it, so the rebuilt set drops it too.
    let in_library = |id: &str| {
        model
            .items
            .iter()
            .any(|i| i.id == id && i.kind == ItemKind::Exercise)
    };
    let mut card: Vec<String> = piece
        .linked_exercise_ids()
        .into_iter()
        .filter(|id| in_library(id))
        .collect();
    let place = |card: &[String], exercise_id: &str| card.iter().position(|id| id == exercise_id);

    match change {
        LinkChange::Move { exercise_id, to } => {
            if let Some(from) = place(&card, &exercise_id) {
                let moved = card.remove(from);
                card.insert(to.min(card.len()), moved);
            }
        }
        LinkChange::Unlink { exercise_id } => {
            if let Some(at) = place(&card, &exercise_id) {
                card.remove(at);
            }
        }
    }

    let links = card
        .into_iter()
        .map(|id| ScaffoldEntry::Existing { id })
        .collect();
    set_piece_links(model, piece_id, links)
}
