use super::variations::migrate_key_into_labels;
use super::*;

pub(super) fn add(model: &mut Model, input: CreateItem) -> Command<Effect, Event> {
    let input = validation::normalize_create_item(input);
    if let Err(e) = validation::validate_create_item(&input) {
        return refuse(model, &e);
    }

    // A brand new item has no earlier variation to have already
    // migrated, so this is always eligible.
    let (variant_labels, key) = migrate_key_into_labels(input.key, input.variant_labels, true);
    if let Err(e) = validation::validate_variant_labels(&variant_labels) {
        return refuse(model, &e);
    }

    let now = chrono::Utc::now();
    let variants = crate::domain::variant::reconcile_variants(vec![], &variant_labels, now);
    let item = Item {
        id: ulid::Ulid::generate().to_string(),
        title: input.title,
        kind: input.kind,
        composer: input.composer,
        key,
        modality: input.modality,
        tempo: input.tempo,
        notes: input.notes,
        tags: input.tags,
        linked_exercise_ids: vec![],
        created_at: now,
        updated_at: now,
        priority: false,
        chord_chart: None,
        variants,
        photo_id: input.photo_id,
        metre: None,
    };

    model.items.push(item.clone());
    model.last_error = None;

    model.clear_error();
    Command::all([
        crate::persistence::save_item(item),
        crux_core::render::render(),
    ])
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
    if let Err(e) = validation::validate_no_variant_labels(&input) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    // Coerced before validation, not after: a piece linked as a related
    // exercise would break the link invariant `validate_link_exercise`
    // guards, and the piece-shaped rules must not run on the way past.
    let input = CreateItem {
        kind: ItemKind::Exercise,
        ..input
    };
    let input = validation::normalize_create_item(input);
    if let Err(e) = validation::validate_create_item(&input) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let now = chrono::Utc::now();
    let exercise = Item {
        id: ulid::Ulid::generate().to_string(),
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
        // No scan surface writes an exercise yet, but a `CreateItem`
        // carrying one must not mean two different things by event.
        photo_id: input.photo_id,
        metre: None,
    };

    let Some(piece) = model.items.iter_mut().find(|i| i.id == piece_id) else {
        model.raise_error(LibraryError::NotFound { id: piece_id }.to_string());
        return crux_core::render::render();
    };
    piece.linked_exercise_ids.push(exercise.id.clone());
    piece.updated_at = now;
    let piece = piece.clone();

    model.items.push(exercise.clone());
    model.clear_error();

    Command::all([
        crate::persistence::save_items(vec![exercise, piece]),
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
    if let Err(e) = validation::validate_create_item(&piece_input) {
        model.last_error_target = form_field(&e).map(|field| FormErrorTarget::Piece { field });
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let mut entries = Vec::with_capacity(exercises.len());
    for (index, entry) in exercises.into_iter().enumerate() {
        match entry {
            ScaffoldEntry::New(input) => {
                if let Err(e) = validation::validate_no_variant_labels(&input) {
                    model.last_error_target = Some(FormErrorTarget::Exercise {
                        index,
                        field: form_field(&e),
                    });
                    model.raise_error(e.to_string());
                    return crux_core::render::render();
                }
                let input = validation::normalize_create_item(CreateItem {
                    kind: ItemKind::Exercise,
                    ..input
                });
                if let Err(e) = validation::validate_create_item(&input) {
                    model.last_error_target = Some(FormErrorTarget::Exercise {
                        index,
                        field: form_field(&e),
                    });
                    model.raise_error(e.to_string());
                    return crux_core::render::render();
                }
                entries.push(ScaffoldEntry::New(input));
            }
            ScaffoldEntry::Existing { id } => {
                if let Err(e) = validation::validate_exercise_link_target(&id, model) {
                    model.last_error_target =
                        Some(FormErrorTarget::Exercise { index, field: None });
                    model.raise_error(e.to_string());
                    return crux_core::render::render();
                }
                entries.push(ScaffoldEntry::Existing { id });
            }
        }
    }

    // No piece exists yet, so the chart derives against the key the form
    // is carrying and the default metre; `SetMetre` re-derives later.
    let chart = match chart.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
        Some(raw) => {
            let key = piece_input.key.clone().unwrap_or_else(|| "C".to_string());
            let modality = piece_input.modality.unwrap_or(Modality::Major);
            match crate::domain::chart::parse_chart(raw, &key, modality, &Metre::default()) {
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
            ScaffoldEntry::New(input) => {
                let exercise = Item {
                    id: ulid::Ulid::generate().to_string(),
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
                    photo_id: input.photo_id,
                    metre: None,
                };
                linked_ids.push(exercise.id.clone());
                created.push(exercise);
            }
            ScaffoldEntry::Existing { id } => {
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
        modality: piece_input.modality,
        tempo: piece_input.tempo,
        notes: piece_input.notes,
        tags: piece_input.tags,
        linked_exercise_ids: linked_ids,
        created_at: now,
        updated_at: now,
        priority: false,
        chord_chart: chart,
        variants: vec![],
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
        crate::persistence::save_items(to_save),
        crux_core::render::render(),
    ])
}
