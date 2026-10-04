use super::*;

pub(super) fn add(model: &mut Model, input: CreateItem) -> Command<Effect, Event> {
    let input = validation::normalize_create_item(input);
    let tempo = match validation::validate_create_item(&input) {
        Ok(tempo) => tempo,
        Err(e) => return refuse(model, &e),
    };

    let now = chrono::Utc::now();
    let (variation_ids, minted) =
        crate::domain::variation::ids_for_labels(&model.variations, &input.variation_labels, now);
    let item = Item {
        id: ulid::Ulid::generate().to_string(),
        title: input.title,
        kind: input.kind,
        composer: input.composer,
        key: input.key,
        tempo,
        notes: input.notes,
        tags: input.tags,
        exercise_links: vec![],
        created_at: now,
        updated_at: now,
        priority: false,
        chord_chart: None,
        photo_id: input.photo_id,
        metre: None,
        sections: vec![],
        variation_ids,
        keys: vec![],
    };

    model.items.push(item.clone());
    model.clear_error();
    let mut writes = Vec::new();
    if !minted.is_empty() {
        model.variations.extend(minted.iter().cloned());
        writes.push(crate::persistence::save_variations(model, minted));
    }
    writes.push(crate::persistence::save_item(model, item));
    writes.push(crux_core::render::render());
    Command::all(writes)
}

pub(super) fn add_linked_exercise(
    model: &mut Model,
    piece_id: String,
    input: CreateItem,
) -> Command<Effect, Event> {
    if let Err(e) = validation::validate_piece_host(&piece_id, model) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }
    if let Err(e) = validation::validate_no_variation_labels(&input) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    // Coerced before validation, not after: a piece linked as a related
    // exercise would break the link invariant, and the piece-shaped rules
    // must not run on the way past.
    let input = CreateItem {
        kind: ItemKind::Exercise,
        ..input
    };
    let input = validation::normalize_create_item(input);
    let tempo = match validation::validate_create_item(&input) {
        Ok(tempo) => tempo,
        Err(e) => {
            model.raise_error(e.to_string());
            return crux_core::render::render();
        }
    };

    let now = chrono::Utc::now();
    let exercise = Item {
        id: ulid::Ulid::generate().to_string(),
        title: input.title,
        kind: input.kind,
        composer: input.composer,
        key: input.key,
        tempo,
        notes: input.notes,
        tags: input.tags,
        exercise_links: vec![],
        created_at: now,
        updated_at: now,
        priority: false,
        chord_chart: None,
        sections: vec![],
        variation_ids: vec![],
        keys: vec![],
        // No scan surface writes an exercise yet, but a `CreateItem`
        // carrying one must not mean two different things by event.
        photo_id: input.photo_id,
        metre: None,
    };

    let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
        model.raise_error(LibraryError::NotFound { id: piece_id }.to_string());
        return crux_core::render::render();
    };
    super::links::append_whole_piece_links(piece, std::slice::from_ref(&exercise.id), now);
    piece.updated_at = now;
    let piece = piece.clone();

    model.items.push(exercise.clone());
    model.clear_error();

    Command::all([
        crate::persistence::save_items(model, vec![exercise, piece]),
        crux_core::render::render(),
    ])
}

pub(super) fn add_piece_in_full(
    model: &mut Model,
    piece: CreateItem,
    chart: Option<String>,
    exercises: Vec<ScaffoldEntry>,
) -> Command<Effect, Event> {
    // Everything is validated before anything is written: no half-made
    // piece, no orphan exercise.
    let piece_input = validation::normalize_create_item(CreateItem {
        kind: ItemKind::Piece,
        ..piece
    });
    let piece_tempo = match validation::validate_create_item(&piece_input) {
        Ok(tempo) => tempo,
        Err(e) => {
            model.last_error_target = form_field(&e).map(|field| FormErrorTarget::Piece { field });
            model.raise_error(e.to_string());
            return crux_core::render::render();
        }
    };

    let mut entries = Vec::with_capacity(exercises.len());
    for (index, entry) in exercises.into_iter().enumerate() {
        let Some(entry) = validate_entry(model, index, entry) else {
            return crux_core::render::render();
        };
        entries.push(entry);
    }

    // No piece exists yet, so the chart derives against the key the form
    // is carrying.
    let chart = match chart.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
        Some(raw) => {
            let key = piece_input.key.unwrap_or(Key::C_MAJOR);
            match crate::domain::chart::parse_chart(raw, key) {
                Ok(chart) => Some(chart),
                Err(e) => {
                    model.last_error_target = Some(if e.bar == 0 {
                        FormErrorTarget::Chart
                    } else {
                        FormErrorTarget::ChartBar {
                            bar_number: e.bar,
                            token: e.token.clone(),
                        }
                    });
                    model.raise_error(e.to_string());
                    return crux_core::render::render();
                }
            }
        }
        None => None,
    };

    let now = chrono::Utc::now();
    let mut created: Vec<Item> = Vec::new();
    let mut linked_ids: Vec<String> = Vec::new();
    for entry in entries {
        match entry {
            Entry::New(input, tempo) => {
                let exercise = new_exercise(*input, tempo, now);
                linked_ids.push(exercise.id.clone());
                created.push(exercise);
            }
            Entry::Existing(id) => {
                if !linked_ids.contains(&id) {
                    linked_ids.push(id);
                }
            }
        }
    }

    let piece_item = Item {
        id: ulid::Ulid::generate().to_string(),
        title: piece_input.title,
        kind: ItemKind::Piece,
        composer: piece_input.composer,
        key: piece_input.key,
        tempo: piece_tempo,
        notes: piece_input.notes,
        tags: piece_input.tags,
        exercise_links: crate::domain::link::whole_piece_links(
            &linked_ids.iter().map(String::as_str).collect::<Vec<_>>(),
            now,
        ),
        created_at: now,
        updated_at: now,
        priority: false,
        chord_chart: chart,
        variation_ids: vec![],
        keys: vec![],
        sections: vec![],
        photo_id: piece_input.photo_id,
        metre: None,
    };

    model.items.extend(created.iter().cloned());
    model.items.push(piece_item.clone());
    model.clear_error();

    // One batch, exercises before the piece: the shell writes it in a
    // single transaction, so the piece never lands without them.
    let mut to_save = created;
    to_save.push(piece_item);
    Command::all([
        crate::persistence::save_items(model, to_save),
        crux_core::render::render(),
    ])
}

/// One exercise row of a create-and-link write, validated.
pub(super) enum Entry {
    New(Box<CreateItem>, Option<Tempo>),
    Existing(String),
}

/// Validates one row, coercing a written one to an exercise. `None` is a
/// refusal, already raised against the row it came from.
pub(super) fn validate_entry(
    model: &mut Model,
    index: usize,
    entry: ScaffoldEntry,
) -> Option<Entry> {
    let refuse_row = |model: &mut Model, e: LibraryError, field: Option<FormErrorField>| {
        model.last_error_target = Some(FormErrorTarget::Exercise { index, field });
        model.raise_error(e.to_string());
        None
    };
    match entry {
        ScaffoldEntry::New(input) => {
            if let Err(e) = validation::validate_no_variation_labels(&input) {
                let field = form_field(&e);
                return refuse_row(model, e, field);
            }
            let input = validation::normalize_create_item(CreateItem {
                kind: ItemKind::Exercise,
                ..input
            });
            match validation::validate_create_item(&input) {
                Ok(tempo) => Some(Entry::New(Box::new(input), tempo)),
                Err(e) => {
                    let field = form_field(&e);
                    refuse_row(model, e, field)
                }
            }
        }
        ScaffoldEntry::Existing { id } => {
            match validation::validate_exercise_link_target(&id, model) {
                Ok(()) => Some(Entry::Existing(id)),
                Err(e) => refuse_row(model, e, None),
            }
        }
    }
}

pub(super) fn new_exercise(
    input: CreateItem,
    tempo: Option<Tempo>,
    now: chrono::DateTime<chrono::Utc>,
) -> Item {
    Item {
        id: ulid::Ulid::generate().to_string(),
        title: input.title,
        kind: input.kind,
        composer: input.composer,
        key: input.key,
        tempo,
        notes: input.notes,
        tags: input.tags,
        created_at: now,
        updated_at: now,
        priority: false,
        chord_chart: None,
        variation_ids: vec![],
        keys: vec![],
        sections: vec![],
        photo_id: input.photo_id,
        metre: None,
        exercise_links: vec![],
    }
}
