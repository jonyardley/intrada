use super::*;

pub(super) fn link_exercise(
    model: &mut Model,
    piece_id: String,
    exercise_id: String,
) -> Command<Effect, Event> {
    if let Err(e) = validation::validate_link_exercise(&piece_id, &exercise_id, model) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
        model.raise_error(LibraryError::NotFound { id: piece_id }.to_string());
        return crux_core::render::render();
    };

    if !piece.linked_exercise_ids.contains(&exercise_id) {
        piece.linked_exercise_ids.push(exercise_id);
    }
    piece.updated_at = chrono::Utc::now();
    model.last_error = None;

    let piece = piece.clone();
    persist_item(model, piece)
}

pub(super) fn unlink_exercise(
    model: &mut Model,
    piece_id: String,
    exercise_id: String,
) -> Command<Effect, Event> {
    let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
        model.raise_error(LibraryError::NotFound { id: piece_id }.to_string());
        return crux_core::render::render();
    };

    piece.linked_exercise_ids.retain(|id| id != &exercise_id);
    piece.updated_at = chrono::Utc::now();
    model.last_error = None;

    let piece = piece.clone();
    persist_item(model, piece)
}

pub(super) fn reorder_linked_exercises(
    model: &mut Model,
    piece_id: String,
    ordered_ids: Vec<String>,
) -> Command<Effect, Event> {
    let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
        model.raise_error(LibraryError::NotFound { id: piece_id }.to_string());
        return crux_core::render::render();
    };

    let current = piece.linked_exercise_ids.clone();
    let current_set: std::collections::HashSet<&String> = current.iter().collect();
    let requested_set: std::collections::HashSet<&String> = ordered_ids.iter().collect();
    let mut seen: std::collections::HashSet<&String> = std::collections::HashSet::new();
    let mut next: Vec<String> = ordered_ids
        .iter()
        .filter(|id| current_set.contains(id) && seen.insert(id))
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
    persist_item(model, piece)
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

    // Re-derive from the stored chart — deterministic, so the committed
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

    // Skip specs already linked (by reserved kind or hand-made title) —
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
            photo_id: None,
            metre: None,
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
        model.raise_error(LibraryError::NotFound { id: piece_id }.to_string());
        return crux_core::render::render();
    };
    piece.linked_exercise_ids.extend(new_ids);
    piece.updated_at = now;
    let piece = piece.clone();

    model.items.extend(new_exercises.iter().cloned());

    model.clear_error();
    let mut batch = new_exercises;
    batch.push(piece);
    Command::all([
        crate::persistence::save_items(batch),
        crux_core::render::render(),
    ])
}
