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
    item.updated_at = chrono::Utc::now();
    model.last_error = None;

    let item = item.clone();
    persist_item(model, item)
}

pub(super) fn delete(model: &mut Model, id: String) -> Command<Effect, Event> {
    let len_before = model.items.len();
    model.items.retain(|i| i.id != id);
    if model.items.len() == len_before {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    }
    model.last_error = None;

    model.clear_error();
    Command::all([
        crate::persistence::delete_item(model, id, chrono::Utc::now()),
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
