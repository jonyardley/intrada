use super::*;

pub(super) fn update(model: &mut Model, id: String, input: UpdateItem) -> Command<Effect, Event> {
    let input = validation::normalize_update_item(input);
    let tempo = match validation::validate_update_item(&input) {
        Ok(tempo) => tempo,
        Err(e) => return refuse(model, &e),
    };

    let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    };
    apply_fields(item, input, tempo);
    item.updated_at = chrono::Utc::now();
    model.last_error = None;

    let item = item.clone();
    persist_item(model, item)
}

/// Only an exercise shows its variation rows, so a piece, including an
/// exercise this edit turns into a piece, keeps its set and the lists go unread.
/// A set the form sends back unchanged keeps the ids of deleted library rows,
/// which the form never sees.
pub(super) fn edit(
    model: &mut Model,
    id: String,
    input: UpdateItem,
    variation_ids: Vec<String>,
    new_labels: Vec<String>,
) -> Command<Effect, Event> {
    let input = validation::normalize_update_item(input);
    let tempo = match validation::validate_update_item(&input) {
        Ok(tempo) => tempo,
        Err(e) => return refuse(model, &e),
    };
    let Some(index) = model.items.iter().position(|i| i.id == id) else {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    };
    let now = chrono::Utc::now();
    let variations =
        if *input.kind.as_ref().unwrap_or(&model.items[index].kind) == ItemKind::Exercise {
            match super::variations::resolve_variations(model, variation_ids, new_labels, now) {
                Ok(set) => Some(set),
                Err(e) => return refuse(model, &e),
            }
        } else {
            None
        };

    let item = &mut model.items[index];
    apply_fields(item, input, tempo);
    let minted = match variations {
        Some((ids, minted)) => {
            if !minted.is_empty() || ids != live_ids(&model.variations, &item.variation_ids) {
                item.variation_ids = ids;
            }
            minted
        }
        None => vec![],
    };
    item.updated_at = now;
    model.last_error = None;

    let item = item.clone();
    super::variations::save_with_minted(model, item, minted)
}

fn live_ids(library: &[crate::domain::variation::Variation], ids: &[String]) -> Vec<String> {
    ids.iter()
        .filter(|id| crate::domain::variation::is_live(library, id))
        .cloned()
        .collect()
}

fn apply_fields(item: &mut Item, input: UpdateItem, tempo: Option<Option<Tempo>>) {
    if let Some(title) = input.title {
        item.title = title;
    }
    if let Some(kind) = input.kind {
        item.kind = kind;
    }
    if let Some(composer) = input.composer {
        item.composer = composer;
    }
    match input.key {
        KeyEdit::Keep => {}
        KeyEdit::Clear => item.key = None,
        KeyEdit::Set { key } => item.key = Some(key),
    }
    if let Some(tempo) = tempo {
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
}

pub(super) fn delete(model: &mut Model, id: String) -> Command<Effect, Event> {
    let Some(item) = model.items.iter().find(|i| i.id == id).cloned() else {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    };
    model.items.retain(|i| i.id != id);
    model.last_error = None;

    model.clear_error();
    Command::all([
        crate::persistence::delete_item(model, item, chrono::Utc::now()),
        crux_core::render::render(),
    ])
}

pub(super) fn set_chord_chart(
    model: &mut Model,
    piece_id: String,
    raw_chart: String,
) -> Command<Effect, Event> {
    if let Err(e) = validation::validate_chart_host(&piece_id, model) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    // The chart derives in the piece's key (default C major when unset).
    let key = model
        .items
        .iter()
        .find(|i| i.id == piece_id)
        .map(|p| p.key.unwrap_or(Key::C_MAJOR))
        .expect("validate_chart_host guarantees the piece exists");

    let chart = match crate::domain::chart::parse_chart(&raw_chart, key) {
        Ok(chart) => chart,
        Err(e) => {
            // Surface the parse error; store nothing (never a partial).
            model.raise_error(e.to_string());
            return crux_core::render::render();
        }
    };

    let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
        model.raise_error(LibraryError::NotFound { id: piece_id }.to_string());
        return crux_core::render::render();
    };
    piece.chord_chart = Some(chart);
    piece.updated_at = chrono::Utc::now();
    model.last_error = None;

    let piece = piece.clone();
    persist_item(model, piece)
}
