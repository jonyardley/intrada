use super::*;
use crate::app::Intrada;
use crate::domain::key::Key;
use crate::domain::types::TempoInput;
use crate::model::{FormErrorField, FormErrorTarget, Model};
use crux_core::App;
use crux_core::Command;

fn make_piece(id: &str) -> Item {
    let now = chrono::Utc::now();
    Item {
        id: id.to_string(),
        title: "Moonlight Sonata".to_string(),
        kind: ItemKind::Piece,
        composer: Some("Beethoven".to_string()),
        key: None,
        tempo: None,
        notes: None,
        tags: vec![],
        exercise_links: vec![],
        created_at: now,
        updated_at: now,
        priority: false,
        chord_chart: None,
        variation_ids: vec![],
        keys: vec![],
        sections: vec![],
        photo_id: None,
        metre: None,
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
        tempo: None,
        notes: None,
        tags: vec![],
        exercise_links: vec![],
        created_at: now,
        updated_at: now,
        priority: false,
        chord_chart: None,
        variation_ids: vec![],
        keys: vec![],
        sections: vec![],
        photo_id: None,
        metre: None,
    }
}

fn model_with_piece_and_exercise() -> Model {
    Model {
        items: vec![make_piece("piece-1"), make_exercise("ex-1")].into(),
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

fn emits_save(
    cmd: &mut crux_core::Command<crate::app::Effect, crate::app::Event>,
    id: &str,
) -> bool {
    cmd.effects().any(|e| {
        matches!(e, crate::app::Effect::Persistence(req)
            if matches!(&req.operation, crate::persistence::PersistenceOperation::SaveItem(item) if item.id == id))
    })
}

// ── SetMetre ──

#[test]
fn set_metre_stores_the_metre_on_a_charted_piece_and_persists() {
    let mut model = model_with_piece_and_exercise();
    let _ = send_cmd(
        &mut model,
        ItemEvent::SetChordChart {
            piece_id: "piece-1".to_string(),
            raw_chart: "| Cm7 F7 |".to_string(),
        },
    );
    let before = model
        .items
        .iter()
        .find(|i| i.id == "piece-1")
        .unwrap()
        .updated_at;

    let waltz = Metre {
        beats: 3,
        unit: 4,
        groups: None,
    };
    let mut cmd = send_cmd(
        &mut model,
        ItemEvent::SetMetre {
            id: "piece-1".to_string(),
            metre: Some(waltz.clone()),
        },
    );

    let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
    assert_eq!(piece.metre, Some(waltz));
    assert!(
        piece.updated_at > before,
        "the metre rides the piece's updated_at"
    );
    assert!(model.last_error.is_none());
    assert!(emits_save(&mut cmd, "piece-1"));
}

#[test]
fn set_metre_rejects_a_grouping_that_does_not_add_up() {
    let mut model = model_with_piece_and_exercise();
    let _ = send_cmd(
        &mut model,
        ItemEvent::SetMetre {
            id: "piece-1".to_string(),
            metre: Some(Metre {
                beats: 7,
                unit: 8,
                groups: Some(vec![3, 3]),
            }),
        },
    );
    assert!(model.last_error.is_some());
    let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
    assert_eq!(piece.metre, None);
}

#[test]
fn set_metre_event_and_item_round_trip_on_ffi_bincode_wire() {
    let metre = Metre {
        beats: 7,
        unit: 8,
        groups: Some(vec![3, 2, 2]),
    };
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::SetMetre {
        id: "p1".to_string(),
        metre: Some(metre.clone()),
    }));
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::SetMetre {
        id: "p1".to_string(),
        metre: None,
    }));
    let mut model = model_with_piece_and_exercise();
    let _ = send_cmd(
        &mut model,
        ItemEvent::SetMetre {
            id: "piece-1".to_string(),
            metre: Some(metre),
        },
    );
    let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
    crate::domain::types::assert_round_trips(piece.clone());
}

// ── SetChordChart ──

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
        p.key = Key::parse("G minor");
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
    assert_eq!(chart.key, Key::parse("G minor"));
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
    let titles: std::collections::HashSet<&str> = new.iter().map(|e| e.title.as_str()).collect();
    assert!(titles.contains("Shells") && titles.contains("Guide-tone lines"));
    assert!(
        new.iter().all(|e| e.key == Some(Key::C_MAJOR)),
        "exercises carry the chart's key"
    );

    let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
    for e in &new {
        assert!(
            piece.linked_exercise_ids().contains(&e.id),
            "each new exercise is linked to the piece"
        );
    }
    assert!(model.last_error.is_none());

    let batch = emits_save_items(&mut cmd).expect("a SaveItems batch is persisted");
    assert_eq!(batch.len(), 3, "two exercises + the piece, one transaction");
    assert!(batch.contains(&"piece-1".to_string()));
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
        piece.exercise_links =
            crate::domain::link::whole_piece_links(&["handmade-shells"], chrono::Utc::now());
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

    let vm = crate::view::rendered(&model);
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

// ── Variations and keys (#2246) ──

fn library_row(id: &str, label: &str) -> crate::domain::variation::Variation {
    crate::domain::variation::Variation {
        id: id.to_string(),
        label: label.to_string(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
    }
}

fn model_with_variation_library() -> Model {
    let mut model = model_with_piece_and_exercise();
    model.variations.extend([
        library_row("v-hs", "Hands separately"),
        library_row("v-dot", "Dotted rhythms"),
    ]);
    model
}

fn set_item_variations(model: &mut Model, id: &str, ids: &[&str], labels: &[&str]) {
    send(
        model,
        ItemEvent::UpdateItemVariations {
            id: id.to_string(),
            variation_ids: ids.iter().map(|s| s.to_string()).collect(),
            new_labels: labels.iter().map(|s| s.to_string()).collect(),
        },
    );
}

fn item_variations(model: &Model, id: &str) -> Vec<String> {
    model
        .items
        .iter()
        .find(|i| i.id == id)
        .expect("the item")
        .variation_ids
        .clone()
}

fn saved_variation_rows(
    cmd: &mut Command<Effect, Event>,
) -> Vec<crate::domain::variation::Variation> {
    cmd.effects()
        .filter_map(|e| match e {
            Effect::Persistence(req) => match req.operation {
                crate::persistence::PersistenceOperation::SaveVariations(rows) => Some(rows),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn an_item_takes_library_variations_in_the_order_chosen() {
    let mut model = model_with_variation_library();

    set_item_variations(&mut model, "piece-1", &["v-dot", "v-hs"], &[]);

    assert!(model.last_error.is_none());
    assert_eq!(item_variations(&model, "piece-1"), vec!["v-dot", "v-hs"]);
}

#[test]
fn a_new_label_mints_one_library_row_and_saves_it() {
    let mut model = model_with_variation_library();

    let mut cmd = Intrada.update(
        Event::Item(ItemEvent::UpdateItemVariations {
            id: "ex-1".to_string(),
            variation_ids: vec!["v-hs".to_string()],
            new_labels: vec![" Slow ".to_string(), "slow".to_string()],
        }),
        &mut model,
    );

    let minted = saved_variation_rows(&mut cmd);
    assert_eq!(minted.len(), 1, "one row, typed twice");
    assert_eq!(minted[0].label, "Slow", "trimmed");
    assert_eq!(model.variations.len(), 3);
    assert_eq!(
        item_variations(&model, "ex-1"),
        vec!["v-hs".to_string(), minted[0].id.clone()]
    );
}

/// A variation made on one item is offered on every item: typing its label
/// on another reuses the row rather than making a second one.
#[test]
fn a_live_label_typed_on_another_item_reuses_the_row() {
    let mut model = model_with_variation_library();

    let mut cmd = Intrada.update(
        Event::Item(ItemEvent::UpdateItemVariations {
            id: "ex-1".to_string(),
            variation_ids: vec![],
            new_labels: vec!["hands SEPARATELY".to_string()],
        }),
        &mut model,
    );

    assert!(saved_variation_rows(&mut cmd).is_empty());
    assert_eq!(item_variations(&model, "ex-1"), vec!["v-hs"]);
    assert_eq!(model.variations.len(), 2);
}

#[test]
fn removing_a_variation_from_an_item_keeps_it_in_the_library() {
    let mut model = model_with_variation_library();
    set_item_variations(&mut model, "ex-1", &["v-hs", "v-dot"], &[]);

    set_item_variations(&mut model, "ex-1", &["v-dot"], &[]);

    assert_eq!(item_variations(&model, "ex-1"), vec!["v-dot"]);
    assert!(crate::domain::variation::is_live(&model.variations, "v-hs"));
}

#[test]
fn an_unchanged_set_writes_nothing() {
    let mut model = model_with_variation_library();
    set_item_variations(&mut model, "ex-1", &["v-hs"], &[]);

    let mut cmd = Intrada.update(
        Event::Item(ItemEvent::UpdateItemVariations {
            id: "ex-1".to_string(),
            variation_ids: vec!["v-hs".to_string()],
            new_labels: vec!["Hands separately".to_string()],
        }),
        &mut model,
    );

    assert!(!cmd.effects().any(|e| matches!(e, Effect::Persistence(_))));
}

#[test]
fn an_item_set_is_refused_whole_on_a_bad_row() {
    let long = "x".repeat(crate::validation::MAX_VARIATION_LABEL + 1);
    let cases: Vec<(Vec<&str>, Vec<&str>)> = vec![
        (vec!["v-gone"], vec![]),
        (vec!["v-hs", "v-hs"], vec![]),
        (vec![], vec![long.as_str()]),
    ];
    for (ids, labels) in cases {
        let mut model = model_with_variation_library();

        set_item_variations(&mut model, "ex-1", &ids, &labels);

        assert!(model.last_error.is_some(), "{ids:?} {labels:?}");
        assert!(item_variations(&model, "ex-1").is_empty());
        assert_eq!(model.variations.len(), 2, "nothing minted");
    }
}

#[test]
fn renaming_a_variation_renames_it_everywhere_and_refuses_a_clash() {
    let mut model = model_with_variation_library();

    send_variation(
        &mut model,
        VariationEvent::Rename {
            id: "v-hs".to_string(),
            label: " Left hand alone ".to_string(),
        },
    );
    assert!(model.last_error.is_none());
    assert_eq!(model.variations[0].label, "Left hand alone");

    send_variation(
        &mut model,
        VariationEvent::Rename {
            id: "v-hs".to_string(),
            label: "dotted RHYTHMS".to_string(),
        },
    );
    assert!(model.last_error.is_some());
    assert_eq!(model.variations[0].label, "Left hand alone");
}

#[test]
fn deleting_a_variation_tombstones_it() {
    let mut model = model_with_variation_library();
    set_item_variations(&mut model, "ex-1", &["v-hs"], &[]);

    let mut cmd = Intrada.update(
        Event::Variation(VariationEvent::Delete {
            id: "v-hs".to_string(),
        }),
        &mut model,
    );

    let saved = saved_variation_rows(&mut cmd);
    assert_eq!(saved.len(), 1);
    assert!(
        saved[0].deleted_at.is_some(),
        "a tombstone, never a hard delete"
    );
    assert_eq!(model.variations.len(), 2, "the row stays for the plays");
    assert!(!crate::domain::variation::is_live(
        &model.variations,
        "v-hs"
    ));
    set_item_variations(&mut model, "ex-2", &["v-hs"], &[]);
    assert!(model.last_error.is_some(), "gone from every picker");
}

fn send_variation(model: &mut Model, event: VariationEvent) {
    let _ = Intrada.update(Event::Variation(event), model);
}

fn key(raw: &str) -> Key {
    Key::parse(raw).expect("a key")
}

#[test]
fn an_item_keeps_its_keys_in_order_and_refuses_one_twice() {
    let mut model = model_with_piece_and_exercise();
    let keys = vec![key("C major"), key("G major"), key("Eb major")];

    send(
        &mut model,
        ItemEvent::UpdateKeys {
            id: "ex-1".to_string(),
            keys: keys.clone(),
        },
    );
    assert!(model.last_error.is_none());
    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(ex.keys, keys);

    send(
        &mut model,
        ItemEvent::UpdateKeys {
            id: "ex-1".to_string(),
            keys: vec![key("Eb major"), key("D# major")],
        },
    );
    assert!(model.last_error.is_some(), "both spellings are one key");
    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(ex.keys, keys);
}

#[test]
fn variation_and_key_events_round_trip_on_the_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(crate::app::Event::Item(
        ItemEvent::UpdateItemVariations {
            id: "ex-1".to_string(),
            variation_ids: vec!["v1".to_string()],
            new_labels: vec!["Slow".to_string()],
        },
    ));
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::UpdateKeys {
        id: "ex-1".to_string(),
        keys: vec![key("F# minor"), key("Bb")],
    }));
    crate::domain::types::assert_round_trips(crate::app::Event::Variation(
        VariationEvent::Rename {
            id: "v1".to_string(),
            label: "Slow".to_string(),
        },
    ));
    crate::domain::types::assert_round_trips(crate::app::Event::Variation(
        VariationEvent::Delete {
            id: "v1".to_string(),
        },
    ));
    let mut ex = make_exercise("ex-1");
    ex.key = Some(key("Db major"));
    ex.keys = vec![key("C"), key("C# minor")];
    ex.variation_ids = vec!["v1".to_string(), "v2".to_string()];
    crate::domain::types::assert_round_trips(ex);
    crate::domain::types::assert_round_trips(library_row("v1", "Hands separately"));
}

// ── Add: variations chosen on create (#2246) ──

#[test]
fn add_creates_the_item_with_its_variations() {
    let mut model = model_with_variation_library();

    send(
        &mut model,
        ItemEvent::Add(CreateItem {
            title: "Nocturne".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Chopin".to_string()),
            key: Some(key("Eb major")),
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: vec!["Hands separately".to_string(), "Slow".to_string()],
        }),
    );

    assert!(model.last_error.is_none());
    let added = model.items.iter().find(|i| i.title == "Nocturne").unwrap();
    assert_eq!(added.key, Some(key("Eb major")));
    assert_eq!(added.variation_ids.len(), 2);
    assert_eq!(added.variation_ids[0], "v-hs");
    assert_eq!(model.variations.len(), 3, "Slow minted");
}

// ── Bridge round-trip for the write events (#846) ──

#[test]
fn chord_chart_events_round_trip_on_the_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::SetChordChart {
        piece_id: "P".to_string(),
        raw_chart: "| Cm7 | F7 |".to_string(),
    }));
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::CommitScaffold {
        piece_id: "P".to_string(),
        kinds: vec![ScaffoldKind::Shells, ScaffoldKind::ScalesToChordTones],
    }));
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
    assert_eq!(piece.linked_exercise_ids(), vec!["ex-1".to_string()]);
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
    assert!(piece.linked_exercise_ids().is_empty());
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
    assert!(piece.linked_exercise_ids().is_empty());
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
    assert!(ex.linked_exercise_ids().is_empty());
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
    assert_eq!(piece.linked_exercise_ids().len(), 1);
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
    assert!(piece.linked_exercise_ids().is_empty());
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
    assert!(piece.linked_exercise_ids().is_empty());
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
        piece.linked_exercise_ids(),
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
        piece.linked_exercise_ids(),
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
    assert_eq!(piece.linked_exercise_ids(), vec!["ex-1".to_string()]);
    assert!(model.last_error.is_none());
}

#[test]
fn reorder_linked_exercises_dedupes_repeated_ids() {
    let mut model = model_with_piece_and_exercise();
    model.items.push(make_exercise("ex-2"));

    for ex in ["ex-1", "ex-2"] {
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
            ordered_ids: vec!["ex-2".to_string(), "ex-1".to_string(), "ex-2".to_string()],
        },
    );

    let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
    assert_eq!(
        piece.linked_exercise_ids(),
        vec!["ex-2".to_string(), "ex-1".to_string()]
    );
    assert!(model.last_error.is_none());
}

// ── AddLinkedExercise ──

fn new_exercise_input(title: &str) -> CreateItem {
    CreateItem {
        title: title.to_string(),
        kind: ItemKind::Exercise,
        composer: None,
        key: crate::domain::key::Key::parse("G"),
        tempo: None,
        notes: None,
        tags: vec![],
        photo_id: None,
        variation_labels: Vec::new(),
    }
}

#[test]
fn add_linked_exercise_creates_it_already_linked_and_persists_one_batch() {
    let mut model = model_with_piece_and_exercise();

    let mut cmd = send_cmd(
        &mut model,
        ItemEvent::AddLinkedExercise {
            piece_id: "piece-1".to_string(),
            input: new_exercise_input("Shell voicings"),
        },
    );

    let created = model
        .items
        .iter()
        .find(|i| i.title == "Shell voicings")
        .expect("the exercise is created");
    assert_eq!(created.kind, ItemKind::Exercise);
    assert_eq!(created.key, Key::parse("G"));

    let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
    assert!(
        piece.linked_exercise_ids().contains(&created.id),
        "the point of the event: created already linked, with no second variation"
    );
    assert!(model.last_error.is_none());

    let batch = emits_save_items(&mut cmd).expect("a SaveItems batch is persisted");
    assert_eq!(batch.len(), 2, "the exercise + the piece, one transaction");
    assert!(batch.contains(&"piece-1".to_string()));
}

#[test]
fn add_linked_exercise_forces_the_kind_to_exercise() {
    let mut model = model_with_piece_and_exercise();
    let mut input = new_exercise_input("Not a piece");
    input.kind = ItemKind::Piece;

    send(
        &mut model,
        ItemEvent::AddLinkedExercise {
            piece_id: "piece-1".to_string(),
            input,
        },
    );

    let created = model
        .items
        .iter()
        .find(|i| i.title == "Not a piece")
        .expect("the item is created");
    assert_eq!(
        created.kind,
        ItemKind::Exercise,
        "a piece linked as a related exercise would break the link invariant"
    );
    assert!(model.last_error.is_none());
}

#[test]
fn add_linked_exercise_rejects_a_non_piece_host() {
    let mut model = model_with_piece_and_exercise();

    let mut cmd = send_cmd(
        &mut model,
        ItemEvent::AddLinkedExercise {
            piece_id: "ex-1".to_string(),
            input: new_exercise_input("Shell voicings"),
        },
    );

    assert!(model.last_error.is_some());
    assert!(
        !model.items.iter().any(|i| i.title == "Shell voicings"),
        "a rejected host creates nothing"
    );
    assert!(emits_save_items(&mut cmd).is_none());
}

#[test]
fn add_linked_exercise_missing_piece_surfaces_not_found() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::AddLinkedExercise {
            piece_id: "nope".to_string(),
            input: new_exercise_input("Shell voicings"),
        },
    );

    assert!(model.last_error.is_some());
    assert!(!model.items.iter().any(|i| i.title == "Shell voicings"));
}

#[test]
fn add_linked_exercise_rejects_a_blank_title() {
    let mut model = model_with_piece_and_exercise();
    let before = model.items.len();

    send(
        &mut model,
        ItemEvent::AddLinkedExercise {
            piece_id: "piece-1".to_string(),
            input: new_exercise_input("   "),
        },
    );

    assert!(model.last_error.is_some());
    assert_eq!(model.items.len(), before, "nothing is created");
    let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
    assert!(
        piece.linked_exercise_ids().is_empty(),
        "and nothing is linked"
    );
}

#[test]
fn add_linked_exercise_rejects_inline_variations_rather_than_dropping_them() {
    let mut model = model_with_piece_and_exercise();
    let before = model.items.len();
    let mut input = new_exercise_input("Shell voicings");
    input.variation_labels = vec!["C".to_string()];

    send(
        &mut model,
        ItemEvent::AddLinkedExercise {
            piece_id: "piece-1".to_string(),
            input,
        },
    );

    assert!(model.last_error.is_some());
    assert_eq!(model.items.len(), before, "nothing is created");
}

// ── The photo a piece is created with ──

/// The page the form was read off is the page you practise from: adding
/// the piece must keep it, or the user photographs it a second time
/// (#1436). Delete the assignment in `Add` and this fails.
#[test]
fn a_piece_created_from_a_scan_keeps_the_page_it_was_read_from() {
    let app = crate::app::Intrada;
    let mut model = Model::default();

    let _ = app.update(
        crate::app::Event::Item(ItemEvent::Add(crate::domain::types::CreateItem {
            title: "Cry Me A River".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Arthur Hamilton".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: Some(PHOTO.to_string()),
            variation_labels: Vec::new(),
        })),
        &mut model,
    );

    assert_eq!(model.items[0].photo_id.as_deref(), Some(PHOTO));
}

/// The id becomes a path component in the shell, so a create carrying a
/// bad one is refused rather than stored.
#[test]
fn a_create_naming_an_unreadable_photo_is_refused() {
    let app = crate::app::Intrada;
    let mut model = Model::default();

    let _ = app.update(
        crate::app::Event::Item(ItemEvent::Add(crate::domain::types::CreateItem {
            title: "Cry Me A River".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Arthur Hamilton".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: Some("../../etc/passwd".to_string()),
            variation_labels: Vec::new(),
        })),
        &mut model,
    );

    assert!(model.items.is_empty());
    assert!(model.last_error.is_some());
}

// ── SetPhoto / ClearPhoto ──

const PHOTO: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const OTHER_PHOTO: &str = "01ARZ3NDEKTSV4RRFFQ69G5FBW";

fn photo_of(model: &Model, id: &str) -> Option<String> {
    model
        .items
        .iter()
        .find(|i| i.id == id)
        .and_then(|i| i.photo_id.clone())
}

fn set_photo_event(id: &str, photo_id: &str) -> ItemEvent {
    ItemEvent::SetPhoto {
        id: id.to_string(),
        photo_id: photo_id.to_string(),
    }
}

#[test]
fn set_photo_stores_the_id_and_persists_without_http() {
    let mut model = model_with_piece_and_exercise();
    let before = model
        .items
        .iter()
        .find(|i| i.id == "piece-1")
        .unwrap()
        .updated_at;

    let mut cmd = send_cmd(&mut model, set_photo_event("piece-1", PHOTO));

    assert_eq!(photo_of(&model, "piece-1").as_deref(), Some(PHOTO));
    assert!(model.last_error.is_none());
    assert!(
        model
            .items
            .iter()
            .find(|i| i.id == "piece-1")
            .unwrap()
            .updated_at
            > before,
        "the photo is part of the item's state, so it moves updated_at for LWW"
    );
    assert!(emits_save(&mut cmd, "piece-1"));
}

#[test]
fn set_photo_over_an_existing_one_replaces_the_id() {
    let mut model = model_with_piece_and_exercise();
    send(&mut model, set_photo_event("piece-1", PHOTO));

    let mut cmd = send_cmd(&mut model, set_photo_event("piece-1", OTHER_PHOTO));

    assert_eq!(photo_of(&model, "piece-1").as_deref(), Some(OTHER_PHOTO));
    assert!(emits_save(&mut cmd, "piece-1"));
}

#[test]
fn setting_the_same_photo_twice_writes_nothing_the_second_time() {
    let mut model = model_with_piece_and_exercise();
    send(&mut model, set_photo_event("piece-1", PHOTO));
    let settled = model
        .items
        .iter()
        .find(|i| i.id == "piece-1")
        .unwrap()
        .updated_at;

    let mut cmd = send_cmd(&mut model, set_photo_event("piece-1", PHOTO));

    assert!(
        !emits_save(&mut cmd, "piece-1"),
        "nothing changed, so there is nothing to write"
    );
    assert_eq!(
        model
            .items
            .iter()
            .find(|i| i.id == "piece-1")
            .unwrap()
            .updated_at,
        settled,
        "an unchanged item must not move updated_at and win a later LWW merge"
    );
}

#[test]
fn clear_photo_clears_the_id() {
    let mut model = model_with_piece_and_exercise();
    send(&mut model, set_photo_event("piece-1", PHOTO));

    let mut cmd = send_cmd(
        &mut model,
        ItemEvent::ClearPhoto {
            id: "piece-1".to_string(),
        },
    );

    assert_eq!(photo_of(&model, "piece-1"), None);
    assert!(model.last_error.is_none());
    assert!(emits_save(&mut cmd, "piece-1"));
}

#[test]
fn clear_photo_on_an_item_without_one_is_a_no_op() {
    let mut model = model_with_piece_and_exercise();

    let mut cmd = send_cmd(
        &mut model,
        ItemEvent::ClearPhoto {
            id: "piece-1".to_string(),
        },
    );

    assert!(model.last_error.is_none());
    assert!(!emits_save(&mut cmd, "piece-1"));
}

#[test]
fn photo_events_reject_an_unknown_item() {
    let mut model = model_with_piece_and_exercise();

    send(&mut model, set_photo_event("nope", PHOTO));
    assert!(model.last_error.is_some());

    model.last_error = None;
    send(
        &mut model,
        ItemEvent::ClearPhoto {
            id: "nope".to_string(),
        },
    );
    assert!(model.last_error.is_some());
}

#[test]
fn set_photo_rejects_an_id_that_is_not_a_ulid() {
    let mut model = model_with_piece_and_exercise();

    // A photo id becomes a path component in the shell, so anything that
    // is not a ulid is a traversal out of the app container.
    for bad in ["../../../etc/passwd", "", "not a ulid", "01J0/0000"] {
        model.last_error = None;
        let mut cmd = send_cmd(&mut model, set_photo_event("piece-1", bad));
        assert!(
            model.last_error.is_some(),
            "{bad:?} should be refused before it reaches the filesystem"
        );
        assert_eq!(photo_of(&model, "piece-1"), None, "{bad:?} stored nothing");
        assert!(
            !emits_save(&mut cmd, "piece-1"),
            "{bad:?} persisted nothing"
        );
    }
}

#[test]
fn set_photo_accepts_an_exercise_too() {
    let mut model = model_with_piece_and_exercise();

    send(&mut model, set_photo_event("ex-1", PHOTO));

    assert_eq!(
        photo_of(&model, "ex-1").as_deref(),
        Some(PHOTO),
        "an exercise photographed from a technique book is the same need"
    );
}

#[test]
fn deleting_an_item_leaves_its_photo_file_alone() {
    let mut model = model_with_piece_and_exercise();
    send(&mut model, set_photo_event("piece-1", PHOTO));

    let mut cmd = send_cmd(
        &mut model,
        ItemEvent::Delete {
            id: "piece-1".to_string(),
        },
    );

    // Key decision 2: the row tombstones, the file stays for the reaping
    // pass (#1442). Deleting here would destroy the photo whenever the
    // delete write that justified it went on to fail.
    let ops: Vec<_> = cmd
        .effects()
        .filter_map(|e| match e {
            crate::app::Effect::Persistence(req) => Some(req.operation.clone()),
            _ => None,
        })
        .collect();
    assert!(
        ops.iter().any(|o| matches!(
            o,
            crate::persistence::PersistenceOperation::DeleteItem { .. }
        )),
        "the row is still tombstoned"
    );
    assert_eq!(ops.len(), 1, "and nothing touches the file: {ops:?}");
}

#[test]
fn photo_events_round_trip_on_the_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(crate::app::Event::Item(set_photo_event(
        "piece-1", PHOTO,
    )));
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::ClearPhoto {
        id: "piece-1".to_string(),
    }));
}

// ── AddPieceInFull ──

fn one_pass_piece_input(title: &str) -> CreateItem {
    CreateItem {
        title: title.to_string(),
        kind: ItemKind::Piece,
        composer: Some("Kosma".to_string()),
        key: crate::domain::key::Key::parse("G minor"),
        tempo: None,
        notes: None,
        tags: vec![],
        photo_id: None,
        variation_labels: Vec::new(),
    }
}

#[test]
fn add_piece_in_full_saves_the_piece_with_its_chart_and_exercises_in_one_batch() {
    let mut model = model_with_piece_and_exercise();

    let mut cmd = send_cmd(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: Some("| Cm7 | F7 | BbMaj7 |".to_string()),
            exercises: vec![
                ScaffoldEntry::Existing {
                    id: "ex-1".to_string(),
                },
                ScaffoldEntry::New(new_exercise_input("Shell voicings")),
            ],
        },
    );

    let piece = model
        .items
        .iter()
        .find(|i| i.title == "Autumn Leaves")
        .expect("the piece is created");
    let written = model
        .items
        .iter()
        .find(|i| i.title == "Shell voicings")
        .expect("the new exercise is created alongside");

    assert!(
        piece.chord_chart.is_some(),
        "the chart lands on the piece, with no second event"
    );
    assert_eq!(
        piece.linked_exercise_ids(),
        vec!["ex-1".to_string(), written.id.clone()],
        "chosen then written, in the order given: neither minting order nor sorted"
    );

    let batch = emits_save_items(&mut cmd).expect("a SaveItems batch is persisted");
    assert_eq!(
        batch.len(),
        2,
        "the new exercise and the piece in one transaction, never one write each"
    );
    assert!(model.last_error.is_none());
}

#[test]
fn add_piece_in_full_writes_nothing_when_the_chart_will_not_parse() {
    let mut model = model_with_piece_and_exercise();
    let before = model.items.len();

    let mut cmd = send_cmd(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: Some("| Cm7 | (F7) |".to_string()),
            exercises: vec![ScaffoldEntry::New(new_exercise_input("Shell voicings"))],
        },
    );

    assert_eq!(
        model.items.len(),
        before,
        "a bar the parser rejects leaves no half-made piece and no orphan exercise"
    );
    assert!(emits_save_items(&mut cmd).is_none(), "and nothing is saved");
    assert!(model.last_error.is_some(), "the parse error is surfaced");
}

#[test]
fn add_piece_in_full_writes_nothing_when_any_exercise_is_invalid() {
    let mut model = model_with_piece_and_exercise();
    let before = model.items.len();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: None,
            exercises: vec![
                ScaffoldEntry::New(new_exercise_input("Shell voicings")),
                ScaffoldEntry::New(new_exercise_input("   ")),
            ],
        },
    );

    assert_eq!(
        model.items.len(),
        before,
        "validation runs over every part before anything is written"
    );
    assert!(model.last_error.is_some());
}

#[test]
fn add_piece_in_full_rejects_inline_variations_on_a_staged_exercise_rather_than_dropping_them() {
    let mut model = model_with_piece_and_exercise();
    let before = model.items.len();
    let mut staged = new_exercise_input("Shell voicings");
    staged.variation_labels = vec!["C".to_string()];

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: None,
            exercises: vec![ScaffoldEntry::New(staged)],
        },
    );

    assert_eq!(model.items.len(), before, "nothing is written");
    assert!(model.last_error.is_some());
}

#[test]
fn add_piece_in_full_rejects_an_exercise_id_that_is_not_there() {
    let mut model = model_with_piece_and_exercise();
    let before = model.items.len();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: None,
            exercises: vec![ScaffoldEntry::Existing {
                id: "gone".to_string(),
            }],
        },
    );

    assert_eq!(model.items.len(), before);
    assert!(model.last_error.is_some());
}

#[test]
fn add_piece_in_full_rejects_linking_a_piece_as_an_exercise() {
    let mut model = model_with_piece_and_exercise();
    let before = model.items.len();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: None,
            exercises: vec![ScaffoldEntry::Existing {
                id: "piece-1".to_string(),
            }],
        },
    );

    assert_eq!(model.items.len(), before);
    assert!(model.last_error.is_some());
}

#[test]
fn add_piece_in_full_takes_a_piece_with_neither_chart_nor_exercises() {
    let mut model = model_with_piece_and_exercise();

    let mut cmd = send_cmd(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: None,
            exercises: vec![],
        },
    );

    let piece = model
        .items
        .iter()
        .find(|i| i.title == "Autumn Leaves")
        .expect("the plain create still works through this path");
    assert!(piece.chord_chart.is_none());
    assert!(piece.linked_exercise_ids().is_empty());
    assert_eq!(emits_save_items(&mut cmd).map(|b| b.len()), Some(1));
    assert!(model.last_error.is_none());
}

#[test]
fn add_piece_in_full_treats_an_empty_chart_as_no_chart() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: Some("   ".to_string()),
            exercises: vec![],
        },
    );

    let piece = model
        .items
        .iter()
        .find(|i| i.title == "Autumn Leaves")
        .expect("an emptied chart sheet still creates the piece");
    assert!(
        piece.chord_chart.is_none(),
        "whitespace is not a chart, and must not be a parse error either"
    );
    assert!(model.last_error.is_none());
}

#[test]
fn add_piece_in_full_points_at_the_piece_field_that_failed() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: CreateItem {
                composer: Some("x".repeat(201)),
                ..one_pass_piece_input("Autumn Leaves")
            },
            chart: None,
            exercises: vec![],
        },
    );

    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Piece {
            field: FormErrorField::Composer
        }),
        "the banner says the composer is too long; the target says which field holds it"
    );
}

#[test]
fn add_piece_in_full_points_at_the_written_row_that_failed() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: None,
            exercises: vec![
                ScaffoldEntry::New(new_exercise_input("Shell voicings")),
                ScaffoldEntry::New(new_exercise_input("   ")),
            ],
        },
    );

    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Exercise {
            index: 1,
            field: Some(FormErrorField::Title)
        }),
        "the blank one is the second row, so a target that always names the first is wrong"
    );
}

#[test]
fn add_piece_in_full_points_at_the_chosen_row_that_has_gone() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: None,
            exercises: vec![
                ScaffoldEntry::Existing {
                    id: "ex-1".to_string(),
                },
                ScaffoldEntry::Existing {
                    id: "gone".to_string(),
                },
            ],
        },
    );

    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Exercise {
            index: 1,
            field: None
        }),
        "a chosen row has no field of its own to mark, only the row"
    );
}

#[test]
fn add_piece_in_full_points_at_the_bar_the_chart_stumbled_on() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: Some("| Cm7 | (F7) |".to_string()),
            exercises: vec![],
        },
    );

    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::ChartBar {
            bar_number: 2,
            token: "(F7)".to_string()
        }),
        "the second bar and the token in it, so the shell highlights in place"
    );
}

#[test]
fn add_piece_in_full_points_at_the_whole_chart_when_it_holds_no_bars() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: Some("swing feel".to_string()),
            exercises: vec![],
        },
    );

    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Chart),
        "prose with no bars in it fails at no bar, so there is no number to hand the shell"
    );
}

#[test]
fn a_quiet_event_keeps_the_message_and_drops_the_mark() {
    let mut model = model_with_piece_and_exercise();
    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: Some("| Cm7 | (F7) |".to_string()),
            exercises: vec![],
        },
    );
    let message = model.last_error.clone();
    assert!(message.is_some());

    let app = Intrada;
    let _cmd = app.update(crate::app::Event::SetUtcOffset { minutes: 60 }, &mut model);

    assert_eq!(model.last_error, message, "the banner keeps its sentence");
    assert!(
        model.last_error_target.is_none(),
        "an event that never touched the error takes the mark with it: no target means \
         the banner alone, never the last mark held over (#1595)"
    );
}

#[test]
fn a_failure_from_anywhere_else_stops_pointing_at_the_form() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: Some("| Cm7 | (F7) |".to_string()),
            exercises: vec![],
        },
    );
    assert!(
        model.last_error_target.is_some(),
        "the chart failure points"
    );

    send(
        &mut model,
        ItemEvent::UpdateKeys {
            id: "no-such-item".to_string(),
            keys: vec![],
        },
    );

    assert!(
        model.last_error.is_some(),
        "the next failure still says what went wrong"
    );
    assert!(
        model.last_error_target.is_none(),
        "but a failure with no field of its own leaves no mark, and never the last one's"
    );
}

#[test]
fn add_piece_in_full_stops_pointing_once_the_form_is_fixed() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: Some("| Cm7 | (F7) |".to_string()),
            exercises: vec![],
        },
    );
    assert!(model.last_error_target.is_some());

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: Some("| Cm7 | F7 |".to_string()),
            exercises: vec![],
        },
    );

    assert!(
        model.last_error_target.is_none(),
        "a create that goes through leaves nothing marked"
    );
}

#[test]
fn form_error_target_round_trips_on_the_ffi_bincode_wire() {
    for target in [
        FormErrorTarget::Piece {
            field: FormErrorField::Title,
        },
        FormErrorTarget::Chart,
        FormErrorTarget::ChartBar {
            bar_number: 2,
            token: "(F7)".to_string(),
        },
        FormErrorTarget::Exercise {
            index: 1,
            field: Some(FormErrorField::Tempo),
        },
        FormErrorTarget::Exercise {
            index: 0,
            field: None,
        },
    ] {
        crate::domain::types::assert_round_trips(target);
    }
}

#[test]
fn add_piece_in_full_round_trips_on_the_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::AddPieceInFull {
        piece: one_pass_piece_input("Autumn Leaves"),
        chart: Some("| Cm7 | F7 |".to_string()),
        exercises: vec![
            ScaffoldEntry::New(new_exercise_input("Shell voicings")),
            ScaffoldEntry::Existing {
                id: "ex-1".to_string(),
            },
        ],
    }));
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::AddPieceInFull {
        piece: one_pass_piece_input("Bare"),
        chart: None,
        exercises: vec![],
    }));
}

// ── Error targets on the item form (#1831) ──

#[test]
fn an_item_set_marks_the_variations_section() {
    let mut model = model_with_piece_and_exercise();

    set_item_variations(&mut model, "ex-1", &[], &[&"x".repeat(101)]);
    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Piece {
            field: FormErrorField::Variations
        }),
    );
}

#[test]
fn add_marks_the_field_it_refused() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::Add(CreateItem {
            title: "Scale".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: vec!["x".repeat(101)],
        }),
    );
    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Piece {
            field: FormErrorField::Variations
        }),
        "an overlong label"
    );

    send(
        &mut model,
        ItemEvent::Add(CreateItem {
            title: "Scale".to_string(),
            kind: ItemKind::Exercise,
            composer: Some("x".repeat(201)),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: vec![],
        }),
    );
    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Piece {
            field: FormErrorField::Composer
        }),
        "a plain field, the way the one-pass create already marks it"
    );
}

#[test]
fn update_marks_the_field_it_refused() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::Update {
            id: "ex-1".to_string(),
            input: UpdateItem {
                title: Some("Scale".to_string()),
                kind: Some(ItemKind::Exercise),
                composer: Some(Some("x".repeat(201))),
                key: None,
                tempo: None,
                notes: None,
                tags: Some(vec![]),
                priority: None,
            },
        },
    );

    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Piece {
            field: FormErrorField::Composer
        })
    );
}

// ── What a musician types into the add and edit forms ──

#[derive(Debug, PartialEq)]
enum Saved {
    As {
        title: String,
        composer: Option<String>,
        tags: Vec<String>,
    },
    Refused(String),
}

fn saved_as(title: &str, composer: Option<&str>, tags: &[&str]) -> Saved {
    Saved::As {
        title: title.to_string(),
        composer: composer.map(str::to_string),
        tags: tags.iter().map(|t| t.to_string()).collect(),
    }
}

fn refused(message: &str) -> Saved {
    Saved::Refused(message.to_string())
}

fn saved_item(model: &Model, id: Option<&str>) -> Saved {
    if let Some(message) = &model.last_error {
        return Saved::Refused(message.clone());
    }
    let item = model
        .items
        .iter()
        .find(|i| id.is_none_or(|id| i.id == id))
        .expect("the item was stored");
    Saved::As {
        title: item.title.clone(),
        composer: item.composer.clone(),
        tags: item.tags.clone(),
    }
}

#[test]
fn adding_a_piece_keeps_what_a_musician_types_and_refuses_what_is_too_long() {
    let over = |c: &str, max: usize| c.repeat(max + 1);
    let cases: Vec<(&str, CreateItem, Saved)> = vec![
        (
            "a háček in the composer",
            CreateItem {
                composer: Some("Dvořák".to_string()),
                ..one_pass_piece_input("Humoresque in G♭")
            },
            saved_as("Humoresque in G♭", Some("Dvořák"), &[]),
        ),
        (
            "padding round an accented composer",
            CreateItem {
                composer: Some("  Saint-Saëns  ".to_string()),
                ..one_pass_piece_input(" Le Cygne ")
            },
            saved_as("Le Cygne", Some("Saint-Saëns"), &[]),
        ),
        (
            "a Japanese title and composer",
            CreateItem {
                composer: Some("ドビュッシー".to_string()),
                ..one_pass_piece_input("月の光")
            },
            saved_as("月の光", Some("ドビュッシー"), &[]),
        ),
        (
            "emoji and accented tags, one per spelling whatever the case",
            CreateItem {
                composer: Some("Fauré".to_string()),
                tags: vec![
                    "🎹 warm-ups".to_string(),
                    "Études".to_string(),
                    "études".to_string(),
                ],
                ..one_pass_piece_input("Pavane")
            },
            saved_as("Pavane", Some("Fauré"), &["🎹 warm-ups", "Études"]),
        ),
        (
            "a blank composer is no composer",
            CreateItem {
                composer: Some("   ".to_string()),
                ..one_pass_piece_input("Gymnopédie No. 1")
            },
            saved_as("Gymnopédie No. 1", None, &[]),
        ),
        (
            "an accented title at the limit",
            one_pass_piece_input(&"é".repeat(validation::MAX_TITLE)),
            saved_as(&"é".repeat(validation::MAX_TITLE), Some("Kosma"), &[]),
        ),
        (
            "an accented composer at the limit",
            CreateItem {
                composer: Some("ø".repeat(validation::MAX_COMPOSER)),
                ..one_pass_piece_input("Holberg Suite")
            },
            saved_as(
                "Holberg Suite",
                Some(&"ø".repeat(validation::MAX_COMPOSER)),
                &[],
            ),
        ),
        (
            "an emoji tag at the limit",
            CreateItem {
                tags: vec!["🎻".repeat(validation::MAX_TAG)],
                ..one_pass_piece_input("Méditation")
            },
            saved_as(
                "Méditation",
                Some("Kosma"),
                &[&"🎻".repeat(validation::MAX_TAG)],
            ),
        ),
        (
            "a blank title",
            one_pass_piece_input("   "),
            refused("Title must be between 1 and 500 characters"),
        ),
        (
            "an accented title one over the limit",
            one_pass_piece_input(&over("é", validation::MAX_TITLE)),
            refused("Title must be between 1 and 500 characters"),
        ),
        (
            "an accented composer one over the limit",
            CreateItem {
                composer: Some(over("ø", validation::MAX_COMPOSER)),
                ..one_pass_piece_input("Holberg Suite")
            },
            refused("Composer must be between 1 and 200 characters"),
        ),
        (
            "an emoji tag one over the limit",
            CreateItem {
                tags: vec![over("🎻", validation::MAX_TAG)],
                ..one_pass_piece_input("Méditation")
            },
            refused("Each tag must be between 1 and 100 characters"),
        ),
        (
            "emoji notes one over the limit",
            CreateItem {
                notes: Some(over("🎶", validation::MAX_NOTES)),
                ..one_pass_piece_input("Rêverie")
            },
            refused("Notes must not exceed 5000 characters"),
        ),
    ];

    for (why, input, expected) in cases {
        let mut model = Model::default();
        send(&mut model, ItemEvent::Add(input));
        assert_eq!(saved_item(&model, None), expected, "{why}");
        if matches!(expected, Saved::Refused(_)) {
            assert!(model.items.is_empty(), "{why}");
        }
    }
}

#[test]
fn editing_a_piece_keeps_what_a_musician_types_and_refuses_what_is_too_long() {
    let over = |c: &str, max: usize| c.repeat(max + 1);
    let cases: Vec<(&str, UpdateItem, Saved)> = vec![
        (
            "an accented title and composer",
            UpdateItem {
                title: Some("Humoresque in G♭".to_string()),
                composer: Some(Some("Antonín Dvořák".to_string())),
                ..Default::default()
            },
            saved_as("Humoresque in G♭", Some("Antonín Dvořák"), &[]),
        ),
        (
            "padding round an accented composer",
            UpdateItem {
                composer: Some(Some("  Saint-Saëns  ".to_string())),
                ..Default::default()
            },
            saved_as("Moonlight Sonata", Some("Saint-Saëns"), &[]),
        ),
        (
            "a Japanese title",
            UpdateItem {
                title: Some("月の光".to_string()),
                ..Default::default()
            },
            saved_as("月の光", Some("Beethoven"), &[]),
        ),
        (
            "a blank composer clears it",
            UpdateItem {
                composer: Some(Some("   ".to_string())),
                ..Default::default()
            },
            saved_as("Moonlight Sonata", None, &[]),
        ),
        (
            "emoji and accented tags, one per spelling whatever the case",
            UpdateItem {
                tags: Some(vec![
                    "🎹 warm-ups".to_string(),
                    "Études".to_string(),
                    "études".to_string(),
                ]),
                ..Default::default()
            },
            saved_as(
                "Moonlight Sonata",
                Some("Beethoven"),
                &["🎹 warm-ups", "Études"],
            ),
        ),
        (
            "a blank title",
            UpdateItem {
                title: Some("   ".to_string()),
                ..Default::default()
            },
            refused("Title must be between 1 and 500 characters"),
        ),
        (
            "an accented composer one over the limit",
            UpdateItem {
                composer: Some(Some(over("ø", validation::MAX_COMPOSER))),
                ..Default::default()
            },
            refused("Composer must be between 1 and 200 characters"),
        ),
        (
            "emoji notes one over the limit",
            UpdateItem {
                notes: Some(Some(over("🎶", validation::MAX_NOTES))),
                ..Default::default()
            },
            refused("Notes must not exceed 5000 characters"),
        ),
    ];

    for (why, input, expected) in cases {
        let mut model = Model {
            items: vec![make_piece("piece-1")].into(),
            ..Default::default()
        };
        let before = model
            .items
            .iter()
            .next()
            .cloned()
            .expect("the fixture piece");
        send(
            &mut model,
            ItemEvent::Update {
                id: "piece-1".to_string(),
                input,
            },
        );
        assert_eq!(saved_item(&model, Some("piece-1")), expected, "{why}");
        if matches!(expected, Saved::Refused(_)) {
            let after = model
                .items
                .iter()
                .find(|i| i.id == "piece-1")
                .expect("the piece is still there");
            assert_eq!(after, &before, "{why}: nothing changed");
        }
    }
}

// ── Typed BPM (#2224) ──

fn typed_bpm(bpm: &str) -> Option<TempoInput> {
    Some(TempoInput {
        marking: None,
        bpm: Some(bpm.to_string()),
    })
}

#[test]
fn add_reads_the_typed_bpm_onto_the_item() {
    let mut model = Model::default();

    send(
        &mut model,
        ItemEvent::Add(CreateItem {
            tempo: typed_bpm(" 96 "),
            ..one_pass_piece_input("Clair de Lune")
        }),
    );

    assert_eq!(
        model.items[0].tempo,
        Some(Tempo {
            marking: None,
            bpm: Some(96)
        })
    );
}

#[test]
fn add_refuses_a_bpm_it_cannot_read_and_stores_nothing() {
    let mut model = Model::default();

    send(
        &mut model,
        ItemEvent::Add(CreateItem {
            tempo: typed_bpm("12a"),
            ..one_pass_piece_input("Clair de Lune")
        }),
    );

    assert!(model.items.is_empty());
    assert_eq!(
        model.last_error.as_deref(),
        Some("BPM must be a whole number between 1 and 400")
    );
    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Piece {
            field: FormErrorField::Tempo
        })
    );
}

#[test]
fn update_refuses_a_bpm_it_cannot_read_and_keeps_the_old_tempo() {
    let mut model = model_with_piece_and_exercise();
    let update = |bpm: &str| ItemEvent::Update {
        id: "piece-1".to_string(),
        input: UpdateItem {
            tempo: typed_bpm(bpm),
            ..Default::default()
        },
    };
    send(&mut model, update("60"));

    send(&mut model, update("12a"));

    assert_eq!(model.items[0].tempo.as_ref().and_then(|t| t.bpm), Some(60));
    assert_eq!(
        model.last_error.as_deref(),
        Some("BPM must be a whole number between 1 and 400")
    );
    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Piece {
            field: FormErrorField::Tempo
        })
    );
}

#[test]
fn add_linked_exercise_refuses_a_bpm_it_cannot_read_and_links_nothing() {
    let mut model = model_with_piece_and_exercise();
    let links_before = model.items[0].linked_exercise_ids().clone();

    send(
        &mut model,
        ItemEvent::AddLinkedExercise {
            piece_id: "piece-1".to_string(),
            input: CreateItem {
                tempo: typed_bpm("12a"),
                ..new_exercise_input("Guide tones")
            },
        },
    );

    assert_eq!(model.items.len(), 2, "nothing is written");
    assert_eq!(model.items[0].linked_exercise_ids(), links_before);
    assert_eq!(
        model.last_error.as_deref(),
        Some("BPM must be a whole number between 1 and 400")
    );
}

#[test]
fn add_linked_exercise_reads_the_typed_bpm_onto_the_exercise() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::AddLinkedExercise {
            piece_id: "piece-1".to_string(),
            input: CreateItem {
                tempo: typed_bpm("80"),
                ..new_exercise_input("Guide tones")
            },
        },
    );

    let exercise = model.items.iter().find(|i| i.title == "Guide tones");
    assert_eq!(
        exercise.and_then(|i| i.tempo.as_ref()).and_then(|t| t.bpm),
        Some(80)
    );
}

#[test]
fn update_sets_and_clears_the_tempo_from_typed_text() {
    let mut model = model_with_piece_and_exercise();
    let update = |tempo| ItemEvent::Update {
        id: "piece-1".to_string(),
        input: UpdateItem {
            tempo: Some(tempo),
            ..Default::default()
        },
    };

    send(&mut model, update(typed_bpm("72").unwrap_or_default()));
    assert_eq!(model.items[0].tempo.as_ref().and_then(|t| t.bpm), Some(72));

    send(&mut model, update(TempoInput::default()));
    assert_eq!(model.items[0].tempo, None);
}

#[test]
fn add_piece_in_full_points_at_the_row_whose_bpm_it_cannot_read() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: one_pass_piece_input("Autumn Leaves"),
            chart: None,
            exercises: vec![
                ScaffoldEntry::New(CreateItem {
                    tempo: typed_bpm("80"),
                    ..new_exercise_input("Shell voicings")
                }),
                ScaffoldEntry::New(CreateItem {
                    tempo: typed_bpm("96.5"),
                    ..new_exercise_input("Guide tones")
                }),
            ],
        },
    );

    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Exercise {
            index: 1,
            field: Some(FormErrorField::Tempo)
        })
    );
    assert_eq!(model.items.len(), 2, "nothing is written");
}

#[test]
fn add_piece_in_full_reads_the_typed_bpm_onto_piece_and_rows() {
    let mut model = Model::default();

    send(
        &mut model,
        ItemEvent::AddPieceInFull {
            piece: CreateItem {
                tempo: typed_bpm("120"),
                ..one_pass_piece_input("Autumn Leaves")
            },
            chart: None,
            exercises: vec![ScaffoldEntry::New(CreateItem {
                tempo: typed_bpm("80"),
                ..new_exercise_input("Shell voicings")
            })],
        },
    );

    let bpm_of = |title: &str| {
        model
            .items
            .iter()
            .find(|i| i.title == title)
            .and_then(|i| i.tempo.as_ref())
            .and_then(|t| t.bpm)
    };
    assert_eq!(bpm_of("Autumn Leaves"), Some(120));
    assert_eq!(bpm_of("Shell voicings"), Some(80));
}

#[test]
fn typed_tempo_round_trips_on_the_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(ItemEvent::AddPieceInFull {
        piece: CreateItem {
            tempo: Some(TempoInput {
                marking: Some("Allegro".to_string()),
                bpm: Some("12a".to_string()),
            }),
            ..one_pass_piece_input("Autumn Leaves")
        },
        chart: None,
        exercises: vec![ScaffoldEntry::New(CreateItem {
            tempo: typed_bpm(""),
            ..new_exercise_input("Shell voicings")
        })],
    });
    crate::domain::types::assert_round_trips(ItemEvent::Update {
        id: "piece-1".to_string(),
        input: UpdateItem {
            tempo: Some(TempoInput::default()),
            ..Default::default()
        },
    });
}
// ── UpdateSections ──

fn row(id: Option<&str>, name: &str, bars: BarsInput) -> SectionEdit {
    SectionEdit {
        id: id.map(str::to_string),
        name: name.to_string(),
        bars,
        kind: SectionKind::Form,
        target_bpm: String::new(),
    }
}

fn typed(raw: &str) -> BarsInput {
    BarsInput::Typed(raw.to_string())
}

fn update_sections(
    model: &mut Model,
    id: &str,
    sections: Vec<SectionEdit>,
) -> Command<Effect, Event> {
    send_cmd(
        model,
        ItemEvent::UpdateSections {
            id: id.to_string(),
            sections,
        },
    )
}

fn piece_sections(model: &Model) -> Vec<ItemSection> {
    model
        .items
        .iter()
        .find(|i| i.id == "piece-1")
        .map(|i| i.sections.clone())
        .unwrap_or_default()
}

fn live_sections(model: &Model) -> Vec<ItemSection> {
    let mut live: Vec<ItemSection> = piece_sections(model)
        .into_iter()
        .filter(|s| s.deleted_at.is_none())
        .collect();
    live.sort_by_key(|s| s.position);
    live
}

fn section_id(model: &Model, name: &str) -> String {
    live_sections(model)
        .into_iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("live section {name}"))
        .id
}

fn piece_view(model: &Model) -> crate::model::LibraryItemView {
    Intrada
        .rendered(model)
        .items
        .into_iter()
        .find(|i| i.id == "piece-1")
        .expect("piece in view")
}

fn bars(first: u16, last: u16) -> Option<BarRange> {
    Some(BarRange { first, last })
}

/// Bars a musician types or dictates, read through the event that consumes
/// them (#1256): stored bars, or `None` when the field was left blank.
#[test]
fn typed_bars_a_musician_would_write_are_read() {
    for (raw, expected) in [
        ("1-16", bars(1, 16)),
        ("1 - 16", bars(1, 16)),
        ("1\u{2013}16", bars(1, 16)),
        ("1\u{2014}16", bars(1, 16)),
        ("1\u{2212}16", bars(1, 16)),
        ("1 to 16", bars(1, 16)),
        ("bars 5 to 12", bars(5, 12)),
        ("Bars 5-12", bars(5, 12)),
        ("bar 12", bars(12, 12)),
        ("12", bars(12, 12)),
        ("bb. 5-12", bars(5, 12)),
        ("mm. 5-12", bars(5, 12)),
        ("", None),
        ("   ", None),
    ] {
        let mut model = model_with_piece_and_exercise();
        let mut cmd = update_sections(&mut model, "piece-1", vec![row(None, "A", typed(raw))]);

        assert!(model.last_error.is_none(), "{raw:?} refused");
        assert!(emits_save(&mut cmd, "piece-1"), "{raw:?} not saved");
        let live = live_sections(&model);
        assert_eq!(live.len(), 1, "{raw:?}");
        assert_eq!(live[0].bars, expected, "{raw:?}");
    }
}

#[test]
fn typed_bars_that_are_not_one_range_are_refused_and_nothing_saves() {
    for raw in [
        "16-1",
        "0-4",
        "1-",
        "-4",
        "1-16, 20-24",
        "12-14 left hand",
        "five to twelve",
        "10000",
        "1.5-3",
    ] {
        let mut model = model_with_piece_and_exercise();
        let mut cmd = update_sections(&mut model, "piece-1", vec![row(None, "A", typed(raw))]);

        assert!(model.last_error.is_some(), "{raw:?} accepted");
        assert_eq!(
            model.last_error_target,
            Some(FormErrorTarget::Piece {
                field: FormErrorField::Sections
            }),
            "{raw:?}"
        );
        assert!(!emits_save(&mut cmd, "piece-1"), "{raw:?} saved");
        assert!(piece_sections(&model).is_empty(), "{raw:?}");
    }
}

#[test]
fn picked_bars_follow_the_same_rules_as_typed_ones() {
    let mut model = model_with_piece_and_exercise();
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![row(None, "A", BarsInput::Picked { first: 9, last: 4 })],
    );
    assert!(model.last_error.is_some(), "a reversed pick is refused");

    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![row(None, "A", BarsInput::Picked { first: 4, last: 9 })],
    );
    assert!(model.last_error.is_none());
    assert_eq!(live_sections(&model)[0].bars, bars(4, 9));
}

#[test]
fn a_piece_takes_a_form_and_a_nameless_trouble_spot() {
    let mut model = model_with_piece_and_exercise();
    let mut cmd = update_sections(
        &mut model,
        "piece-1",
        vec![
            row(None, "A1", typed("1-16")),
            row(None, "B", BarsInput::Blank),
            row(None, "A2", BarsInput::Blank),
            SectionEdit {
                kind: SectionKind::TroubleSpot,
                target_bpm: " 72 ".to_string(),
                ..row(None, "", typed("bars 12 to 14"))
            },
        ],
    );

    assert!(model.last_error.is_none());
    assert!(emits_save(&mut cmd, "piece-1"));
    let live = live_sections(&model);
    let names: Vec<&str> = live.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["A1", "B", "A2", ""]);
    assert_eq!(live[3].kind, SectionKind::TroubleSpot);
    assert_eq!(live[3].bars, bars(12, 14));
    assert_eq!(live[3].target_bpm, Some(72));
    assert_eq!(live[0].target_bpm, None, "blank is the piece's tempo");
}

#[test]
fn an_exercise_takes_sections_too() {
    let mut model = model_with_piece_and_exercise();
    let _ = update_sections(&mut model, "ex-1", vec![row(None, "Up", BarsInput::Blank)]);

    assert!(model.last_error.is_none());
    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(ex.sections.len(), 1);
}

#[test]
fn rename_rebar_kind_target_reorder_add_and_remove_land_in_one_write() {
    let mut model = model_with_piece_and_exercise();
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![
            row(None, "A", typed("1-8")),
            row(None, "B", typed("9-16")),
            row(None, "C", typed("17-24")),
        ],
    );
    let a = section_id(&model, "A");
    let b = section_id(&model, "B");
    let c = section_id(&model, "C");

    let mut cmd = update_sections(
        &mut model,
        "piece-1",
        vec![
            SectionEdit {
                kind: SectionKind::TroubleSpot,
                target_bpm: "60".to_string(),
                ..row(Some(&b), "B'", typed("9-12"))
            },
            row(Some(&a), "A", typed("1-8")),
            row(None, "Coda", BarsInput::Blank),
        ],
    );

    assert!(model.last_error.is_none());
    assert!(emits_save(&mut cmd, "piece-1"));
    let live = live_sections(&model);
    let shape: Vec<(&str, &str)> = live
        .iter()
        .map(|s| (s.id.as_str(), s.name.as_str()))
        .collect();
    assert_eq!(shape[0], (b.as_str(), "B'"));
    assert_eq!(shape[1], (a.as_str(), "A"));
    assert_eq!(shape[2].1, "Coda");
    assert_eq!(live[0].bars, bars(9, 12));
    assert_eq!(live[0].kind, SectionKind::TroubleSpot);
    assert_eq!(live[0].target_bpm, Some(60));

    let removed = piece_sections(&model)
        .into_iter()
        .find(|s| s.id == c)
        .expect("a removed section stays as a tombstone");
    assert!(removed.deleted_at.is_some());
    assert_eq!(
        Some(removed.updated_at),
        removed.deleted_at,
        "the tombstone carries its own LWW stamp"
    );
}

#[test]
fn a_row_naming_a_tombstone_revives_it() {
    let mut model = model_with_piece_and_exercise();
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![row(None, "A", BarsInput::Blank)],
    );
    let a = section_id(&model, "A");
    let _ = update_sections(&mut model, "piece-1", vec![]);

    let mut cmd = update_sections(
        &mut model,
        "piece-1",
        vec![row(Some(&a), "A", BarsInput::Blank)],
    );

    assert!(emits_save(&mut cmd, "piece-1"));
    let live = live_sections(&model);
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].id, a);
}

#[test]
fn the_view_orders_by_position_whatever_order_the_store_loaded() {
    let mut model = model_with_piece_and_exercise();
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![
            row(None, "A", BarsInput::Blank),
            row(None, "B", BarsInput::Blank),
        ],
    );
    let piece = model.items.iter_mut().find(|i| i.id == "piece-1").unwrap();
    piece.sections.reverse();

    let labels: Vec<String> = piece_view(&model)
        .sections
        .into_iter()
        .map(|s| s.label)
        .collect();
    assert_eq!(labels, vec!["A", "B"]);
}

#[test]
fn a_swap_of_two_names_keeps_each_id() {
    let mut model = model_with_piece_and_exercise();
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![
            row(None, "A", BarsInput::Blank),
            row(None, "B", BarsInput::Blank),
        ],
    );
    let a = section_id(&model, "A");
    let b = section_id(&model, "B");

    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![
            row(Some(&a), "B", BarsInput::Blank),
            row(Some(&b), "A", BarsInput::Blank),
        ],
    );

    let live = live_sections(&model);
    assert_eq!(live[0].id, a);
    assert_eq!(live[0].name, "B");
    assert_eq!(live[1].id, b);
    assert_eq!(
        piece_sections(&model).len(),
        2,
        "a swap is not remove plus add"
    );
}

#[test]
fn duplicate_names_are_two_sections() {
    let mut model = model_with_piece_and_exercise();
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![
            row(None, "A", typed("1-8")),
            row(None, "B", typed("9-16")),
            row(None, "A", typed("17-24")),
        ],
    );

    assert!(model.last_error.is_none());
    assert_eq!(live_sections(&model).len(), 3);
}

#[test]
fn a_refused_row_saves_nothing_even_after_valid_ones() {
    let mut model = model_with_piece_and_exercise();
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![row(None, "A", BarsInput::Blank)],
    );
    let before = piece_sections(&model);

    let mut cmd = update_sections(
        &mut model,
        "piece-1",
        vec![
            row(None, "B", BarsInput::Blank),
            row(None, "C", typed("16-1")),
        ],
    );

    assert!(model.last_error.is_some());
    assert!(!emits_save(&mut cmd, "piece-1"));
    assert_eq!(piece_sections(&model), before);
}

#[test]
fn a_nameless_row_with_no_bars_is_refused() {
    let mut model = model_with_piece_and_exercise();
    let mut cmd = update_sections(
        &mut model,
        "piece-1",
        vec![row(None, "  ", BarsInput::Blank)],
    );

    assert!(model.last_error.is_some());
    assert!(!emits_save(&mut cmd, "piece-1"));
}

#[test]
fn a_long_name_and_an_unreadable_target_are_refused() {
    let mut model = model_with_piece_and_exercise();
    let long = "x".repeat(crate::validation::MAX_SECTION_NAME + 1);
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![row(None, &long, BarsInput::Blank)],
    );
    assert!(model.last_error.is_some(), "name over the limit");

    for target in ["0", "401", "fast", "72.5"] {
        let mut model = model_with_piece_and_exercise();
        let _ = update_sections(
            &mut model,
            "piece-1",
            vec![SectionEdit {
                target_bpm: target.to_string(),
                ..row(None, "A", BarsInput::Blank)
            }],
        );
        assert_eq!(
            model.last_error_target,
            Some(FormErrorTarget::Piece {
                field: FormErrorField::Sections
            }),
            "{target:?}"
        );
    }
}

#[test]
fn an_identical_list_writes_nothing_and_keeps_updated_at() {
    let mut model = model_with_piece_and_exercise();
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![
            row(None, "A", typed("1-8")),
            row(None, "B", BarsInput::Blank),
        ],
    );
    let a = section_id(&model, "A");
    let b = section_id(&model, "B");
    let before = model
        .items
        .iter()
        .find(|i| i.id == "piece-1")
        .unwrap()
        .clone();

    let mut cmd = update_sections(
        &mut model,
        "piece-1",
        vec![
            row(Some(&a), " A ", typed("bars 1 to 8")),
            row(Some(&b), "B", BarsInput::Blank),
        ],
    );

    assert!(model.last_error.is_none());
    assert!(!emits_save(&mut cmd, "piece-1"));
    let after = model.items.iter().find(|i| i.id == "piece-1").unwrap();
    assert_eq!(after, &before);
}

#[test]
fn an_empty_list_tombstones_every_section() {
    let mut model = model_with_piece_and_exercise();
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![row(None, "A", BarsInput::Blank)],
    );

    let mut cmd = update_sections(&mut model, "piece-1", vec![]);

    assert!(emits_save(&mut cmd, "piece-1"));
    assert!(live_sections(&model).is_empty());
    assert_eq!(piece_sections(&model).len(), 1);
}

#[test]
fn an_unknown_item_is_not_found() {
    let mut model = model_with_piece_and_exercise();
    let mut cmd = update_sections(
        &mut model,
        "missing",
        vec![row(None, "A", BarsInput::Blank)],
    );

    assert!(model.last_error.is_some());
    assert!(!emits_save(&mut cmd, "missing"));
}

#[test]
fn the_view_hides_tombstones_orders_by_position_and_labels_by_bars() {
    let mut model = model_with_piece_and_exercise();
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![
            row(None, "Gone", BarsInput::Blank),
            row(None, "A1", typed("1-16")),
            row(None, "", typed("bar 12")),
            row(None, "", typed("12-14")),
            row(None, "B", BarsInput::Blank),
        ],
    );
    let gone = section_id(&model, "Gone");
    let a1 = section_id(&model, "A1");
    let b = section_id(&model, "B");
    let spots: Vec<String> = live_sections(&model)
        .into_iter()
        .filter(|s| s.name.is_empty())
        .map(|s| s.id)
        .collect();
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![
            row(Some(&b), "B", BarsInput::Blank),
            row(Some(&a1), "A1", typed("1-16")),
            row(Some(&spots[0]), "", typed("bar 12")),
            row(Some(&spots[1]), "", typed("12-14")),
        ],
    );
    assert!(piece_sections(&model).iter().any(|s| s.id == gone));

    let views = piece_view(&model).sections;
    let labels: Vec<(&str, Option<&str>)> = views
        .iter()
        .map(|v| (v.label.as_str(), v.bars_caption.as_deref()))
        .collect();
    assert_eq!(
        labels,
        vec![
            ("B", None),
            ("A1", Some("Bars 1 to 16")),
            ("Bar 12", None),
            ("Bars 12 to 14", None),
        ]
    );
    assert_eq!(views[1].first_bar, Some(1));
    assert_eq!(views[1].last_bar, Some(16));
    assert_eq!(views[3].name, "");
}

#[test]
fn update_sections_and_an_item_with_sections_round_trip_on_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::UpdateSections {
        id: "piece-1".to_string(),
        sections: vec![
            row(Some("s-1"), "A1", typed("1\u{2013}16")),
            SectionEdit {
                kind: SectionKind::TroubleSpot,
                target_bpm: "72".to_string(),
                ..row(
                    None,
                    "",
                    BarsInput::Picked {
                        first: 12,
                        last: 14,
                    },
                )
            },
            row(None, "B", BarsInput::Blank),
        ],
    }));
    let mut model = model_with_piece_and_exercise();
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![row(None, "A1", typed("1-16")), row(None, "", typed("12"))],
    );
    let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
    crate::domain::types::assert_round_trips(piece.clone());
    crate::domain::types::assert_round_trips(piece_view(&model));
}

// ── Section links (#2248) ──

use crate::domain::link::{ExerciseLink, LinkEdit, LinkTarget};

fn section(id: &str, name: &str, position: usize) -> ItemSection {
    ItemSection {
        id: id.to_string(),
        name: name.to_string(),
        bars: None,
        kind: SectionKind::Form,
        target_bpm: None,
        position,
        updated_at: chrono::Utc::now(),
        deleted_at: None,
    }
}

/// Nocturne (A1, A2, a removed B) and an étude (coda), with two drills.
fn linking_model() -> Model {
    let mut nocturne = make_piece("piece-1");
    let mut removed = section("s-b", "B", 2);
    removed.deleted_at = Some(chrono::Utc::now());
    nocturne.sections = vec![section("s-a1", "A1", 0), section("s-a2", "A2", 1), removed];
    let mut etude = make_piece("piece-2");
    etude.title = "Étude".to_string();
    etude.sections = vec![section("s-coda", "Coda", 0)];
    let mut thirds = make_exercise("ex-2");
    thirds.title = "Thirds".to_string();
    Model {
        items: vec![nocturne, etude, make_exercise("ex-1"), thirds].into(),
        ..Default::default()
    }
}

fn item<'a>(model: &'a Model, id: &str) -> &'a Item {
    model.items.iter().find(|i| i.id == id).expect("item")
}

/// The live link rows of a piece as (exercise, section), by position.
fn live_pairs(model: &Model, piece_id: &str) -> Vec<(String, Option<String>)> {
    item(model, piece_id)
        .live_links()
        .into_iter()
        .map(|l| (l.exercise_id.clone(), l.section_id.clone()))
        .collect()
}

fn pair(exercise: &str, section: Option<&str>) -> (String, Option<String>) {
    (exercise.to_string(), section.map(str::to_string))
}

fn existing(exercise: &str, section: Option<&str>) -> LinkEdit {
    LinkEdit {
        exercise: ScaffoldEntry::Existing {
            id: exercise.to_string(),
        },
        section_id: section.map(str::to_string),
    }
}

fn target(piece: &str, section: Option<&str>) -> LinkTarget {
    LinkTarget {
        piece_id: piece.to_string(),
        section_id: section.map(str::to_string),
    }
}

fn set_piece_links(
    model: &mut Model,
    piece_id: &str,
    links: Vec<LinkEdit>,
) -> Command<Effect, Event> {
    send_cmd(
        model,
        ItemEvent::SetPieceLinks {
            piece_id: piece_id.to_string(),
            links,
        },
    )
}

fn set_exercise_links(
    model: &mut Model,
    exercise_id: &str,
    targets: Vec<LinkTarget>,
) -> Command<Effect, Event> {
    send_cmd(
        model,
        ItemEvent::SetExerciseLinks {
            exercise_id: exercise_id.to_string(),
            targets,
        },
    )
}

fn persists_anything(cmd: &mut Command<Effect, Event>) -> bool {
    cmd.effects()
        .any(|e| matches!(e, crate::app::Effect::Persistence(_)))
}

fn view_of(model: &Model, id: &str) -> crate::model::LibraryItemView {
    Intrada
        .rendered(model)
        .items
        .into_iter()
        .find(|i| i.id == id)
        .expect("item in view")
}

fn draft(title: &str) -> CreateItem {
    CreateItem {
        title: title.to_string(),
        kind: ItemKind::Piece,
        composer: None,
        key: None,
        tempo: None,
        notes: None,
        tags: vec![],
        photo_id: None,
        variation_labels: vec![],
    }
}

#[test]
fn set_piece_links_holds_whole_piece_and_section_links_in_card_order() {
    let mut model = linking_model();

    let mut cmd = set_piece_links(
        &mut model,
        "piece-1",
        vec![
            existing("ex-1", None),
            existing("ex-2", Some("s-a2")),
            existing("ex-1", Some("s-a2")),
        ],
    );

    assert!(model.last_error.is_none(), "{:?}", model.last_error);
    assert!(emits_save(&mut cmd, "piece-1"));
    assert_eq!(
        live_pairs(&model, "piece-1"),
        vec![
            pair("ex-1", None),
            pair("ex-2", Some("s-a2")),
            pair("ex-1", Some("s-a2")),
        ]
    );
    let card = view_of(&model, "piece-1").linked_exercises;
    let rows: Vec<(String, bool, Vec<String>)> = card
        .iter()
        .map(|e| {
            (
                e.id.clone(),
                e.whole_piece,
                e.sections.iter().map(|s| s.label.clone()).collect(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            ("ex-1".to_string(), true, vec!["A2".to_string()]),
            ("ex-2".to_string(), false, vec!["A2".to_string()]),
        ]
    );
}

#[test]
fn set_piece_links_adds_and_removes_in_one_event() {
    let mut model = linking_model();
    let _ = set_piece_links(&mut model, "piece-1", vec![existing("ex-1", None)]);

    let _ = set_piece_links(
        &mut model,
        "piece-1",
        vec![existing("ex-2", None), existing("ex-1", Some("s-a1"))],
    );

    assert_eq!(
        live_pairs(&model, "piece-1"),
        vec![pair("ex-2", None), pair("ex-1", Some("s-a1"))]
    );
    let removed: Vec<&ExerciseLink> = item(&model, "piece-1")
        .exercise_links
        .iter()
        .filter(|l| l.exercise_id == "ex-1" && l.section_id.is_none())
        .collect();
    assert_eq!(removed.len(), 1);
    assert!(
        removed[0].deleted_at.is_some(),
        "an unlink tombstones the row"
    );
}

#[test]
fn set_piece_links_creates_a_written_exercise_linked_to_a_section() {
    let mut model = linking_model();

    let mut cmd = set_piece_links(
        &mut model,
        "piece-1",
        vec![LinkEdit {
            exercise: ScaffoldEntry::New(draft("  Broken octaves ")),
            section_id: Some("s-a2".to_string()),
        }],
    );

    assert!(model.last_error.is_none(), "{:?}", model.last_error);
    let created = model
        .items
        .iter()
        .find(|i| i.title == "Broken octaves")
        .expect("created")
        .clone();
    assert_eq!(created.kind, ItemKind::Exercise);
    assert_eq!(
        live_pairs(&model, "piece-1"),
        vec![pair(&created.id, Some("s-a2"))]
    );
    assert_eq!(
        emits_save_items(&mut cmd),
        Some(vec![created.id.clone(), "piece-1".to_string()])
    );
}

#[test]
fn set_piece_links_refuses_the_whole_set_on_one_bad_row() {
    for (case, bad) in [
        ("unknown section", existing("ex-2", Some("s-nope"))),
        ("another piece's section", existing("ex-2", Some("s-coda"))),
        ("a removed section", existing("ex-2", Some("s-b"))),
        ("a piece as the exercise", existing("piece-2", None)),
        ("an unknown exercise", existing("ex-nope", None)),
        (
            "a draft with no title",
            LinkEdit {
                exercise: ScaffoldEntry::New(draft("   ")),
                section_id: None,
            },
        ),
    ] {
        let mut model = linking_model();
        let _ = set_piece_links(&mut model, "piece-1", vec![existing("ex-1", None)]);
        let before = item(&model, "piece-1").clone();
        let count = model.items.len();

        let mut cmd = set_piece_links(
            &mut model,
            "piece-1",
            vec![existing("ex-1", Some("s-a1")), bad],
        );

        assert!(model.last_error.is_some(), "{case}: refused");
        assert!(!persists_anything(&mut cmd), "{case}: nothing saved");
        assert_eq!(item(&model, "piece-1"), &before, "{case}: links unchanged");
        assert_eq!(model.items.len(), count, "{case}: nothing created");
    }
}

#[test]
fn set_piece_links_refuses_an_exercise_as_the_host() {
    let mut model = linking_model();
    let mut cmd = set_piece_links(&mut model, "ex-1", vec![existing("ex-2", None)]);
    assert!(model.last_error.is_some());
    assert!(!persists_anything(&mut cmd));
    assert!(item(&model, "ex-1").exercise_links.is_empty());
}

#[test]
fn set_piece_links_with_the_stored_set_writes_nothing() {
    let mut model = linking_model();
    let _ = set_piece_links(
        &mut model,
        "piece-1",
        vec![existing("ex-1", None), existing("ex-2", Some("s-a2"))],
    );
    let stamped = item(&model, "piece-1").updated_at;

    let mut cmd = set_piece_links(
        &mut model,
        "piece-1",
        vec![existing("ex-1", None), existing("ex-2", Some("s-a2"))],
    );

    assert!(!persists_anything(&mut cmd));
    assert_eq!(item(&model, "piece-1").updated_at, stamped);
}

#[test]
fn relinking_after_an_unlink_revives_the_same_row() {
    let mut model = linking_model();
    let _ = set_piece_links(&mut model, "piece-1", vec![existing("ex-1", Some("s-a2"))]);
    let first = item(&model, "piece-1").exercise_links[0].id.clone();

    let _ = set_piece_links(&mut model, "piece-1", vec![]);
    let _ = set_piece_links(&mut model, "piece-1", vec![existing("ex-1", Some("s-a2"))]);

    let rows = &item(&model, "piece-1").exercise_links;
    assert_eq!(rows.len(), 1, "no twin row minted");
    assert_eq!(rows[0].id, first);
    assert!(rows[0].deleted_at.is_none());
}

#[test]
fn set_exercise_links_reaches_sections_of_two_pieces_in_one_batch() {
    let mut model = linking_model();

    let mut cmd = set_exercise_links(
        &mut model,
        "ex-1",
        vec![
            target("piece-1", Some("s-a2")),
            target("piece-2", Some("s-coda")),
        ],
    );

    assert!(model.last_error.is_none(), "{:?}", model.last_error);
    let mut saved = emits_save_items(&mut cmd).expect("one batch");
    saved.sort();
    assert_eq!(saved, vec!["piece-1".to_string(), "piece-2".to_string()]);
    assert_eq!(
        live_pairs(&model, "piece-1"),
        vec![pair("ex-1", Some("s-a2"))]
    );
    assert_eq!(
        live_pairs(&model, "piece-2"),
        vec![pair("ex-1", Some("s-coda"))]
    );

    let mut used_in: Vec<(String, bool, bool, Vec<String>)> = view_of(&model, "ex-1")
        .used_in
        .iter()
        .map(|u| {
            (
                u.piece.as_ref().expect("a piece").id.clone(),
                u.linked,
                u.whole_piece,
                u.sections.iter().map(|s| s.label.clone()).collect(),
            )
        })
        .collect();
    used_in.sort();
    assert_eq!(
        used_in,
        vec![
            ("piece-1".to_string(), true, false, vec!["A2".to_string()]),
            ("piece-2".to_string(), true, false, vec!["Coda".to_string()]),
        ]
    );
}

#[test]
fn set_exercise_links_saves_only_the_pieces_it_changes() {
    let mut model = linking_model();
    let _ = set_exercise_links(
        &mut model,
        "ex-1",
        vec![target("piece-1", None), target("piece-2", None)],
    );
    let _ = set_piece_links(
        &mut model,
        "piece-1",
        vec![existing("ex-1", None), existing("ex-2", None)],
    );

    let mut cmd = set_exercise_links(&mut model, "ex-1", vec![target("piece-2", None)]);

    assert_eq!(
        emits_save_items(&mut cmd),
        Some(vec!["piece-1".to_string()])
    );
    assert_eq!(live_pairs(&model, "piece-1"), vec![pair("ex-2", None)]);
    assert_eq!(live_pairs(&model, "piece-2"), vec![pair("ex-1", None)]);
}

#[test]
fn set_exercise_links_refuses_the_whole_set_on_one_bad_target() {
    for (case, bad) in [
        ("another piece's section", target("piece-1", Some("s-coda"))),
        ("a removed section", target("piece-1", Some("s-b"))),
        ("an exercise as the piece", target("ex-2", None)),
        ("an unknown piece", target("piece-nope", None)),
    ] {
        let mut model = linking_model();
        let before: Vec<Item> = model.items.iter().cloned().collect();

        let mut cmd = set_exercise_links(&mut model, "ex-1", vec![target("piece-2", None), bad]);

        assert!(model.last_error.is_some(), "{case}: refused");
        assert!(!persists_anything(&mut cmd), "{case}: nothing saved");
        let after: Vec<Item> = model.items.iter().cloned().collect();
        assert_eq!(after, before, "{case}: nothing changed");
    }
    let mut model = linking_model();
    let mut cmd = set_exercise_links(&mut model, "piece-2", vec![target("piece-1", None)]);
    assert!(
        model.last_error.is_some(),
        "a piece is not linked as an exercise"
    );
    assert!(!persists_anything(&mut cmd));
}

#[test]
fn removing_a_section_removes_only_its_links() {
    let mut model = linking_model();
    let _ = set_piece_links(
        &mut model,
        "piece-1",
        vec![
            existing("ex-1", None),
            existing("ex-1", Some("s-a2")),
            existing("ex-2", Some("s-a1")),
        ],
    );
    let exercise_before = item(&model, "ex-1").clone();

    let mut cmd = update_sections(
        &mut model,
        "piece-1",
        vec![row(Some("s-a1"), "A1", BarsInput::Blank)],
    );

    assert!(emits_save(&mut cmd, "piece-1"));
    assert!(!emits_save(&mut cmd, "ex-1"), "the exercise is not written");
    assert_eq!(item(&model, "ex-1"), &exercise_before);
    let piece = item(&model, "piece-1");
    let a2 = piece
        .exercise_links
        .iter()
        .find(|l| l.section_id.as_deref() == Some("s-a2"))
        .expect("the row is kept");
    assert!(
        a2.deleted_at.is_some(),
        "the removed section's link is tombstoned"
    );
    assert_eq!(
        live_pairs(&model, "piece-1"),
        vec![pair("ex-1", None), pair("ex-2", Some("s-a1"))]
    );
}

#[test]
fn a_link_to_a_removed_section_is_hidden_from_both_screens() {
    let mut model = linking_model();
    let now = chrono::Utc::now();
    if let Some(p) = model.items.iter_mut().find(|i| i.id == "piece-1") {
        p.exercise_links = vec![
            ExerciseLink::new("ex-1".to_string(), Some("s-b".to_string()), 0, now),
            ExerciseLink::new("ex-2".to_string(), Some("s-a1".to_string()), 1, now),
        ];
    }

    let card: Vec<String> = view_of(&model, "piece-1")
        .linked_exercises
        .iter()
        .map(|e| e.id.clone())
        .collect();
    assert_eq!(card, vec!["ex-2".to_string()]);
    assert!(view_of(&model, "ex-1").used_in.is_empty());
}

#[test]
fn link_events_and_an_item_with_links_round_trip_on_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::SetPieceLinks {
        piece_id: "piece-1".to_string(),
        links: vec![
            existing("ex-1", None),
            LinkEdit {
                exercise: ScaffoldEntry::New(draft("Thirds")),
                section_id: Some("s-a2".to_string()),
            },
        ],
    }));
    crate::domain::types::assert_round_trips(crate::app::Event::Item(
        ItemEvent::SetExerciseLinks {
            exercise_id: "ex-1".to_string(),
            targets: vec![target("piece-1", None), target("piece-2", Some("s-coda"))],
        },
    ));
    let mut model = linking_model();
    let _ = set_piece_links(
        &mut model,
        "piece-1",
        vec![existing("ex-1", None), existing("ex-2", Some("s-a2"))],
    );
    let _ = set_piece_links(&mut model, "piece-1", vec![existing("ex-2", Some("s-a2"))]);
    let piece = item(&model, "piece-1");
    assert!(piece.exercise_links.iter().any(|l| l.deleted_at.is_some()));
    crate::domain::types::assert_round_trips(piece.clone());
    crate::domain::types::assert_round_trips(view_of(&model, "piece-1"));
    crate::domain::types::assert_round_trips(view_of(&model, "ex-2"));
}

fn old_stamp() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now() - chrono::Duration::days(30)
}

/// Links on piece-1 as (exercise, section, position), every row stamped long ago.
fn seed_links(model: &mut Model, rows: &[(&str, Option<&str>, usize)]) {
    let at = old_stamp();
    let piece = model
        .items
        .iter_mut()
        .find(|i| i.id == "piece-1")
        .expect("piece");
    piece.exercise_links = rows
        .iter()
        .map(|(ex, section, position)| {
            ExerciseLink::new(ex.to_string(), section.map(str::to_string), *position, at)
        })
        .collect();
}

fn link_row<'a>(model: &'a Model, exercise: &str, section: Option<&str>) -> &'a ExerciseLink {
    item(model, "piece-1")
        .exercise_links
        .iter()
        .find(|l| l.exercise_id == exercise && l.section_id.as_deref() == section)
        .expect("link row")
}

#[test]
fn set_piece_links_with_the_stored_set_at_gapped_positions_writes_nothing() {
    let mut model = linking_model();
    seed_links(&mut model, &[("ex-1", None, 0), ("ex-2", Some("s-a2"), 2)]);
    let stamped = item(&model, "piece-1").updated_at;

    let mut cmd = set_piece_links(
        &mut model,
        "piece-1",
        vec![existing("ex-1", None), existing("ex-2", Some("s-a2"))],
    );

    assert!(!persists_anything(&mut cmd));
    assert_eq!(item(&model, "piece-1").updated_at, stamped);
}

#[test]
fn link_exercise_after_an_unlink_revives_the_same_row() {
    let mut model = linking_model();
    seed_links(&mut model, &[("ex-1", None, 0)]);
    let first = link_row(&model, "ex-1", None).id.clone();

    for event in [
        ItemEvent::UnlinkExercise {
            piece_id: "piece-1".to_string(),
            exercise_id: "ex-1".to_string(),
        },
        ItemEvent::LinkExercise {
            piece_id: "piece-1".to_string(),
            exercise_id: "ex-1".to_string(),
        },
    ] {
        send(&mut model, event);
    }

    let rows = &item(&model, "piece-1").exercise_links;
    assert_eq!(rows.len(), 1, "no twin row minted");
    assert_eq!(rows[0].id, first);
    assert!(rows[0].deleted_at.is_none());
    assert!(rows[0].updated_at > old_stamp());
}

#[test]
fn every_link_write_stamps_the_rows_it_changes() {
    let stale = |model: &Model, ex: &str, section: Option<&str>| {
        link_row(model, ex, section).updated_at <= old_stamp()
    };

    let mut model = linking_model();
    seed_links(&mut model, &[("ex-1", None, 0), ("ex-2", None, 1)]);
    let _ = set_piece_links(&mut model, "piece-1", vec![existing("ex-2", None)]);
    assert!(!stale(&model, "ex-1", None), "a tombstone is stamped");
    assert!(!stale(&model, "ex-2", None), "a moved row is stamped");

    let mut model = linking_model();
    seed_links(&mut model, &[("ex-1", None, 0)]);
    let mut tombstoned = model
        .items
        .iter()
        .find(|i| i.id == "piece-1")
        .unwrap()
        .clone();
    tombstoned.exercise_links[0].deleted_at = Some(old_stamp());
    if let Some(p) = model.items.iter_mut().find(|i| i.id == "piece-1") {
        *p = tombstoned;
    }
    let _ = set_piece_links(&mut model, "piece-1", vec![existing("ex-1", None)]);
    assert!(!stale(&model, "ex-1", None), "a revived row is stamped");

    let mut model = linking_model();
    seed_links(&mut model, &[("ex-1", None, 0), ("ex-2", None, 1)]);
    send(
        &mut model,
        ItemEvent::ReorderLinkedExercises {
            piece_id: "piece-1".to_string(),
            ordered_ids: vec!["ex-2".to_string(), "ex-1".to_string()],
        },
    );
    assert!(!stale(&model, "ex-1", None), "a reordered row is stamped");

    let mut model = linking_model();
    seed_links(&mut model, &[("ex-1", Some("s-a2"), 0)]);
    let _ = update_sections(
        &mut model,
        "piece-1",
        vec![row(Some("s-a1"), "A1", BarsInput::Blank)],
    );
    assert!(
        !stale(&model, "ex-1", Some("s-a2")),
        "a link removed with its section is stamped"
    );
}

#[test]
fn a_repeated_row_links_once() {
    let mut model = linking_model();
    let _ = set_piece_links(
        &mut model,
        "piece-1",
        vec![
            existing("ex-1", Some("s-a2")),
            existing("ex-1", Some("s-a2")),
        ],
    );
    assert_eq!(item(&model, "piece-1").exercise_links.len(), 1);

    let mut model = linking_model();
    let _ = set_exercise_links(
        &mut model,
        "ex-1",
        vec![
            target("piece-2", Some("s-coda")),
            target("piece-2", Some("s-coda")),
        ],
    );
    assert_eq!(item(&model, "piece-2").exercise_links.len(), 1);
}
