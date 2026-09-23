use super::*;
use crate::app::Intrada;
use crate::model::{FormErrorField, FormErrorTarget, Model};
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
        photo_id: None,
        metre: None,
    }
}

fn model_with_piece_and_exercise() -> Model {
    Model {
        items: vec![make_piece("piece-1"), make_exercise("ex-1")],
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

fn beats_of(model: &Model, id: &str) -> Vec<u8> {
    model
        .items
        .iter()
        .find(|i| i.id == id)
        .and_then(|i| i.chord_chart.as_ref())
        .expect("chart stored")
        .sections[0]
        .bars[0]
        .chords
        .iter()
        .map(|c| c.beats)
        .collect()
}

/// The chart derives its beat split from the item's metre, at parse time
/// and again when the metre changes (spec question 1).
#[test]
fn set_metre_rederives_a_charted_pieces_beats_and_persists() {
    let mut model = model_with_piece_and_exercise();
    let _ = send_cmd(
        &mut model,
        ItemEvent::SetChordChart {
            piece_id: "piece-1".to_string(),
            raw_chart: "| Cm7 F7 |".to_string(),
        },
    );
    assert_eq!(beats_of(&model, "piece-1"), vec![2, 2], "4/4 by default");
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
    assert_eq!(beats_of(&model, "piece-1"), vec![2, 1]);
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
fn a_chart_parses_against_the_pieces_metre() {
    let mut model = model_with_piece_and_exercise();
    let _ = send_cmd(
        &mut model,
        ItemEvent::SetMetre {
            id: "piece-1".to_string(),
            metre: Some(Metre {
                beats: 6,
                unit: 8,
                groups: Some(vec![3, 3]),
            }),
        },
    );
    let _ = send_cmd(
        &mut model,
        ItemEvent::SetChordChart {
            piece_id: "piece-1".to_string(),
            raw_chart: "| Cm7 F7 |".to_string(),
        },
    );
    assert_eq!(beats_of(&model, "piece-1"), vec![3, 3]);
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

// ── RenameVariant ──

#[test]
fn rename_variant_renames_in_place_keeping_id_and_history() {
    let mut model = model_with_piece_and_exercise();
    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["C".to_string(), "F".to_string()],
        },
    );
    let c_id = exercise_variants(&model)
        .iter()
        .find(|v| v.label == "C")
        .unwrap()
        .id
        .clone();
    let before = model
        .items
        .iter()
        .find(|i| i.id == "ex-1")
        .unwrap()
        .updated_at;

    let mut cmd = send_cmd(
        &mut model,
        ItemEvent::RenameVariant {
            item_id: "ex-1".to_string(),
            variant_id: c_id.clone(),
            new_label: "Do".to_string(),
        },
    );

    let variants = exercise_variants(&model);
    let renamed = variants.iter().find(|v| v.id == c_id).unwrap();
    assert_eq!(renamed.label, "Do", "label updated");
    assert_eq!(variants.len(), 2, "no new row created");
    assert!(
        variants.iter().any(|v| v.label == "F"),
        "other variation untouched"
    );
    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert!(ex.updated_at >= before, "touches the item's updated_at");
    assert!(model.last_error.is_none());
    assert!(emits_save(&mut cmd, "ex-1"), "local-first persists");
}

#[test]
fn rename_variant_rejects_a_duplicate_against_another_live_variation() {
    let mut model = model_with_piece_and_exercise();
    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["C".to_string(), "F".to_string()],
        },
    );
    let c_id = exercise_variants(&model)
        .iter()
        .find(|v| v.label == "C")
        .unwrap()
        .id
        .clone();

    send(
        &mut model,
        ItemEvent::RenameVariant {
            item_id: "ex-1".to_string(),
            variant_id: c_id,
            new_label: "f".to_string(),
        },
    );

    assert!(
        model.last_error.is_some(),
        "case-insensitive duplicate rejected"
    );
    let variants = exercise_variants(&model);
    assert!(variants.iter().any(|v| v.label == "C"), "unchanged");
    assert!(variants.iter().any(|v| v.label == "F"), "unchanged");
}

#[test]
fn rename_variant_is_a_noop_without_a_save_when_unchanged() {
    let mut model = model_with_piece_and_exercise();
    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["C".to_string()],
        },
    );
    let c_id = exercise_variants(&model)[0].id.clone();
    let before = model
        .items
        .iter()
        .find(|i| i.id == "ex-1")
        .unwrap()
        .updated_at;

    let mut cmd = send_cmd(
        &mut model,
        ItemEvent::RenameVariant {
            item_id: "ex-1".to_string(),
            variant_id: c_id,
            new_label: "C".to_string(),
        },
    );

    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(ex.updated_at, before, "unchanged label leaves the stamp");
    assert!(!emits_save(&mut cmd, "ex-1"), "nothing to persist");
    assert!(model.last_error.is_none());
}

#[test]
fn rename_variant_rejects_a_blank_label() {
    let mut model = model_with_piece_and_exercise();
    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["C".to_string()],
        },
    );
    let c_id = exercise_variants(&model)[0].id.clone();

    send(
        &mut model,
        ItemEvent::RenameVariant {
            item_id: "ex-1".to_string(),
            variant_id: c_id,
            new_label: "   ".to_string(),
        },
    );

    assert!(model.last_error.is_some());
    assert_eq!(exercise_variants(&model)[0].label, "C", "unchanged");
}

#[test]
fn rename_variant_on_missing_variant_surfaces_not_found() {
    let mut model = model_with_piece_and_exercise();
    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["C".to_string()],
        },
    );

    send(
        &mut model,
        ItemEvent::RenameVariant {
            item_id: "ex-1".to_string(),
            variant_id: "nope".to_string(),
            new_label: "Do".to_string(),
        },
    );

    assert!(model.last_error.is_some());
    assert_eq!(exercise_variants(&model)[0].label, "C", "unchanged");
}

#[test]
fn rename_variant_on_a_tombstoned_variation_surfaces_not_found() {
    let mut model = model_with_piece_and_exercise();
    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["C".to_string(), "F".to_string()],
        },
    );
    let f_id = exercise_variants(&model)
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
        ItemEvent::RenameVariant {
            item_id: "ex-1".to_string(),
            variant_id: f_id,
            new_label: "Fa".to_string(),
        },
    );

    assert!(
        model.last_error.is_some(),
        "a tombstoned variation can't be renamed"
    );
}

#[test]
fn rename_variant_event_round_trips_on_the_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::RenameVariant {
        item_id: "ex-1".to_string(),
        variant_id: "v-1".to_string(),
        new_label: "Do".to_string(),
    }));
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

// ── Update: a key cannot return to a laddered exercise (#1783) ──

#[test]
fn update_rejects_setting_a_key_on_an_exercise_with_a_live_variation() {
    let mut model = model_with_piece_and_exercise();
    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["C".to_string()],
        },
    );

    send(
        &mut model,
        ItemEvent::Update {
            id: "ex-1".to_string(),
            input: crate::domain::types::UpdateItem {
                key: Some(Some("G".to_string())),
                ..Default::default()
            },
        },
    );

    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(ex.key, None, "the set was refused");
    assert!(model.last_error.is_some());
}

/// A pre-this-change exercise can carry both a key and a live variation
/// (#1783 self-review). Its edit form loads that key and resends it
/// unchanged on every save (`ItemFormModel.updateInput()`), so an
/// unchanged key must not be treated as a new one, or every other field
/// on the same edit becomes unreachable until the key is cleared by
/// hand.
#[test]
fn update_resending_an_unchanged_key_on_an_exercise_with_a_live_variation_still_updates_other_fields(
) {
    let mut model = model_with_piece_and_exercise();
    model.items.iter_mut().find(|i| i.id == "ex-1").unwrap().key = Some("C".to_string());
    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["F".to_string()],
        },
    );
    // The migration above already cleared it; set it back by hand so
    // this test exercises resending a key still on the exercise, not
    // one only just cleared.
    model.items.iter_mut().find(|i| i.id == "ex-1").unwrap().key = Some("stale".to_string());

    send(
        &mut model,
        ItemEvent::Update {
            id: "ex-1".to_string(),
            input: crate::domain::types::UpdateItem {
                title: Some("Renamed".to_string()),
                key: Some(Some("stale".to_string())),
                ..Default::default()
            },
        },
    );

    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(
        ex.title, "Renamed",
        "the unrelated field update was refused"
    );
    assert_eq!(
        ex.key,
        Some("stale".to_string()),
        "the unchanged key was kept"
    );
    assert!(model.last_error.is_none());
}

#[test]
fn update_still_allows_clearing_a_key_on_an_exercise_with_a_live_variation() {
    let mut model = model_with_piece_and_exercise();
    model.items.iter_mut().find(|i| i.id == "ex-1").unwrap().key = Some("C".to_string());
    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["F".to_string()],
        },
    );
    // The migration above already cleared it; set it back by hand so this
    // test still exercises a clear on a laddered exercise that has one.
    model.items.iter_mut().find(|i| i.id == "ex-1").unwrap().key = Some("stale".to_string());

    send(
        &mut model,
        ItemEvent::Update {
            id: "ex-1".to_string(),
            input: crate::domain::types::UpdateItem {
                key: Some(None),
                ..Default::default()
            },
        },
    );

    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(ex.key, None);
    assert!(model.last_error.is_none());
}

#[test]
fn update_allows_setting_a_key_on_an_exercise_with_no_live_variation() {
    let mut model = model_with_piece_and_exercise();

    send(
        &mut model,
        ItemEvent::Update {
            id: "ex-1".to_string(),
            input: crate::domain::types::UpdateItem {
                key: Some(Some("G".to_string())),
                ..Default::default()
            },
        },
    );

    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(ex.key.as_deref(), Some("G"));
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
            "reordering keeps each variation's id (and so its history)"
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
    assert_eq!(
        dead.len(),
        1,
        "the removed variation is kept as a tombstone"
    );
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
        "the re-added variation resurrects its old id (score history intact)"
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
    assert_eq!(variants.len(), 2, "cleared variations remain as tombstones");
    assert!(variants.iter().all(|v| v.deleted_at.is_some()));
    assert!(model.last_error.is_none());
    assert!(emits_save(&mut cmd, "ex-1"), "the clear persists");
}

#[test]
fn set_variants_untouched_variation_keeps_its_updated_at() {
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
        "an untouched variation keeps its LWW timestamp"
    );
    let f = variants.iter().find(|v| v.label == "F").unwrap();
    assert!(
        f.updated_at > c_updated_at,
        "the tombstoned variation is stamped"
    );
}

#[test]
fn set_variants_adopts_incoming_casing_keeping_the_id() {
    let mut model = model_with_piece_and_exercise();
    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["bb".to_string()],
        },
    );
    let id_before = exercise_variants(&model)[0].id.clone();

    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["Bb".to_string()],
        },
    );

    let variants = exercise_variants(&model);
    assert_eq!(variants.len(), 1);
    assert_eq!(
        variants[0].label, "Bb",
        "a relabel in casing only is adopted"
    );
    assert_eq!(variants[0].id, id_before, "same variation, history intact");
}

#[test]
fn set_variants_identical_labels_is_a_noop_without_a_save() {
    // LWW hygiene one level up: a no-op must not bump the parent row's
    // timestamp or write, or it could spuriously win a future sync merge.
    let mut model = model_with_piece_and_exercise();
    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["C".to_string(), "F".to_string()],
        },
    );
    let before = model
        .items
        .iter()
        .find(|i| i.id == "ex-1")
        .unwrap()
        .updated_at;

    let mut cmd = send_cmd(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["C".to_string(), "F".to_string()],
        },
    );

    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(
        ex.updated_at, before,
        "an unchanged ladder leaves the item stamp"
    );
    assert!(!emits_save(&mut cmd, "ex-1"), "nothing to persist");
    assert!(model.last_error.is_none());
}

#[test]
fn set_variants_migrates_an_existing_key_into_the_first_variation() {
    let mut model = model_with_piece_and_exercise();
    model.items.iter_mut().find(|i| i.id == "ex-1").unwrap().key = Some("C".to_string());

    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["F".to_string(), "Bb".to_string()],
        },
    );

    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(ex.key, None, "the key moved into the ladder");
    assert_eq!(
        ex.variants
            .iter()
            .map(|v| v.label.as_str())
            .collect::<Vec<_>>(),
        vec!["C", "F", "Bb"],
        "the old key leads the ladder it is folded into"
    );
}

#[test]
fn set_variants_clears_a_key_already_named_among_the_labels_without_duplicating_it() {
    let mut model = model_with_piece_and_exercise();
    model.items.iter_mut().find(|i| i.id == "ex-1").unwrap().key = Some("c".to_string());

    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["C".to_string(), "F".to_string()],
        },
    );

    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(
        ex.key, None,
        "the ladder already covers the key, case-insensitively, so nothing folds in, \
         but the exercise is still gaining its first live variation and the key stops \
         being a fact about it"
    );
    assert_eq!(ex.variants.len(), 2, "no duplicate C variation was added");
}

#[test]
fn set_variants_leaves_a_key_alone_when_the_exercise_already_had_variations() {
    let mut model = model_with_piece_and_exercise();
    let ex = model.items.iter_mut().find(|i| i.id == "ex-1").unwrap();
    ex.key = Some("C".to_string());
    ex.variants = vec![Variant {
        id: "v-existing".to_string(),
        label: "G".to_string(),
        position: 0,
        updated_at: chrono::Utc::now(),
        deleted_at: None,
    }];

    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec!["G".to_string(), "D".to_string()],
        },
    );

    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(
        ex.key,
        Some("C".to_string()),
        "the transition already happened before this call; migration is not repeated"
    );
    assert_eq!(ex.variants.len(), 2);
}

#[test]
fn set_variants_clearing_the_ladder_does_not_migrate_a_key() {
    let mut model = model_with_piece_and_exercise();
    model.items.iter_mut().find(|i| i.id == "ex-1").unwrap().key = Some("C".to_string());

    send(
        &mut model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: vec![],
        },
    );

    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(
        ex.key,
        Some("C".to_string()),
        "clearing an empty ladder to empty is a no-op, the key is untouched"
    );
    assert!(ex.variants.is_empty());
}

#[test]
fn set_variants_event_round_trips_on_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::SetVariants {
        id: "ex-1".to_string(),
        labels: vec!["C".to_string(), "F♯".to_string()],
    }));
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

#[test]
fn variant_payloads_round_trip_on_the_ffi_bincode_wire() {
    // Bare Variant, with the tombstone set (deleted_at is a later bincode
    // field, so guard both levels).
    let now = chrono::Utc::now();
    crate::domain::types::assert_round_trips(Variant {
        id: "v1".to_string(),
        label: "F major".to_string(),
        position: 2,
        updated_at: now,
        deleted_at: Some(now),
    });

    // The whole exercise carries its ladder in ViewModel/persistence
    // payloads, so round-trip an Item with variants populated.
    let mut ex = make_exercise("ex-1");
    ex.variants = vec![
        Variant {
            id: "v1".to_string(),
            label: "F".to_string(),
            position: 0,
            updated_at: now,
            deleted_at: None,
        },
        Variant {
            id: "v2".to_string(),
            label: "Bb".to_string(),
            position: 1,
            updated_at: now,
            deleted_at: None,
        },
    ];
    crate::domain::types::assert_round_trips(ex);
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
        piece.linked_exercise_ids,
        vec!["ex-2".to_string(), "ex-1".to_string()]
    );
    assert!(model.last_error.is_none());
}

// ── Add: variations added inline (#1783) ──

#[test]
fn add_creates_the_exercise_with_its_inline_variations() {
    let mut model = model_with_piece_and_exercise();
    let mut input = new_exercise_input("Shell voicings");
    input.key = None;
    input.variant_labels = vec!["C".to_string(), "F".to_string()];

    send(&mut model, ItemEvent::Add(input));

    let created = model
        .items
        .iter()
        .find(|i| i.title == "Shell voicings")
        .expect("the exercise is created");
    assert_eq!(
        created
            .variants
            .iter()
            .map(|v| v.label.as_str())
            .collect::<Vec<_>>(),
        vec!["C", "F"]
    );
    assert!(model.last_error.is_none());
}

#[test]
fn add_migrates_the_typed_key_into_the_first_inline_variation() {
    let mut model = model_with_piece_and_exercise();
    let mut input = new_exercise_input("Shell voicings");
    input.key = Some("G".to_string());
    input.variant_labels = vec!["C".to_string(), "F".to_string()];

    send(&mut model, ItemEvent::Add(input));

    let created = model
        .items
        .iter()
        .find(|i| i.title == "Shell voicings")
        .expect("the exercise is created");
    assert_eq!(created.key, None, "the typed key moved into the ladder");
    assert_eq!(
        created
            .variants
            .iter()
            .map(|v| v.label.as_str())
            .collect::<Vec<_>>(),
        vec!["G", "C", "F"],
        "the typed key leads the ladder it is folded into"
    );
}

#[test]
fn add_clears_a_typed_key_already_named_among_the_inline_variations() {
    let mut model = model_with_piece_and_exercise();
    let mut input = new_exercise_input("Shell voicings");
    input.key = Some("g".to_string());
    input.variant_labels = vec!["G".to_string(), "C".to_string()];

    send(&mut model, ItemEvent::Add(input));

    let created = model
        .items
        .iter()
        .find(|i| i.title == "Shell voicings")
        .expect("the exercise is created");
    assert_eq!(created.key, None, "the ladder already covers the key");
    assert_eq!(
        created
            .variants
            .iter()
            .map(|v| v.label.as_str())
            .collect::<Vec<_>>(),
        vec!["G", "C"],
        "no duplicate G variation was added"
    );
}

#[test]
fn add_with_no_inline_variations_keeps_the_key_as_a_key() {
    let mut model = model_with_piece_and_exercise();
    let mut input = new_exercise_input("Shell voicings");
    input.key = Some("G".to_string());

    send(&mut model, ItemEvent::Add(input));

    let created = model
        .items
        .iter()
        .find(|i| i.title == "Shell voicings")
        .expect("the exercise is created");
    assert_eq!(created.key.as_deref(), Some("G"));
    assert!(created.variants.is_empty());
}

#[test]
fn add_rejects_inline_variations_on_a_piece() {
    let mut model = model_with_piece_and_exercise();
    let before = model.items.len();
    let input = CreateItem {
        title: "Clair de Lune".to_string(),
        kind: ItemKind::Piece,
        composer: Some("Debussy".to_string()),
        key: None,
        modality: None,
        tempo: None,
        notes: None,
        tags: vec![],
        photo_id: None,
        variant_labels: vec!["Slow".to_string()],
    };

    send(&mut model, ItemEvent::Add(input));

    assert!(model.last_error.is_some());
    assert_eq!(
        model.items.len(),
        before,
        "a rejected create writes nothing"
    );
}

#[test]
fn add_rejects_inline_variations_over_the_cap_once_the_key_is_folded_in() {
    let mut model = model_with_piece_and_exercise();
    let before = model.items.len();
    let mut input = new_exercise_input("Shell voicings");
    input.key = Some("G".to_string());
    input.variant_labels = (0..crate::validation::MAX_VARIANTS)
        .map(|i| format!("Variation {i}"))
        .collect();

    send(&mut model, ItemEvent::Add(input));

    assert!(
        model.last_error.is_some(),
        "the migrated key pushes the ladder one over the cap"
    );
    assert_eq!(model.items.len(), before);
}

#[test]
fn add_event_with_variant_labels_round_trips_on_ffi_bincode_wire() {
    let mut input = new_exercise_input("Shell voicings");
    input.variant_labels = vec!["C".to_string(), "F\u{266F}".to_string()];
    crate::domain::types::assert_round_trips(crate::app::Event::Item(ItemEvent::Add(input)));
}

// ── AddLinkedExercise ──

fn new_exercise_input(title: &str) -> CreateItem {
    CreateItem {
        title: title.to_string(),
        kind: ItemKind::Exercise,
        composer: None,
        key: Some("G".to_string()),
        modality: None,
        tempo: None,
        notes: None,
        tags: vec![],
        photo_id: None,
        variant_labels: Vec::new(),
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
    assert_eq!(created.key.as_deref(), Some("G"));

    let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
    assert!(
        piece.linked_exercise_ids.contains(&created.id),
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
        piece.linked_exercise_ids.is_empty(),
        "and nothing is linked"
    );
}

#[test]
fn add_linked_exercise_rejects_inline_variations_rather_than_dropping_them() {
    let mut model = model_with_piece_and_exercise();
    let before = model.items.len();
    let mut input = new_exercise_input("Shell voicings");
    input.variant_labels = vec!["C".to_string()];

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
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: Some(PHOTO.to_string()),
            variant_labels: Vec::new(),
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
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: Some("../../etc/passwd".to_string()),
            variant_labels: Vec::new(),
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
        key: Some("G".to_string()),
        modality: Some(Modality::Minor),
        tempo: None,
        notes: None,
        tags: vec![],
        photo_id: None,
        variant_labels: Vec::new(),
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
        piece.linked_exercise_ids,
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
    staged.variant_labels = vec!["C".to_string()];

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
    assert!(piece.linked_exercise_ids.is_empty());
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
        ItemEvent::SetVariants {
            id: "piece-1".to_string(),
            labels: vec!["C".to_string()],
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

// ── UpdateVariants ──

fn set_ladder(model: &mut Model, labels: &[&str]) {
    send(
        model,
        ItemEvent::SetVariants {
            id: "ex-1".to_string(),
            labels: labels.iter().map(|l| l.to_string()).collect(),
        },
    );
}

fn variant_id(model: &Model, label: &str) -> String {
    exercise_variants(model)
        .iter()
        .find(|v| v.label == label && v.deleted_at.is_none())
        .unwrap_or_else(|| panic!("live variation {label}"))
        .id
        .clone()
}

fn live_ladder(model: &Model) -> Vec<(String, String)> {
    let mut live: Vec<_> = exercise_variants(model)
        .iter()
        .filter(|v| v.deleted_at.is_none())
        .collect();
    live.sort_by_key(|v| v.position);
    live.iter()
        .map(|v| (v.id.clone(), v.label.clone()))
        .collect()
}

fn edit(id: Option<String>, label: &str) -> VariantEdit {
    VariantEdit {
        id,
        label: label.to_string(),
    }
}

fn update_variants(
    model: &mut Model,
    id: &str,
    variants: Vec<VariantEdit>,
) -> Command<Effect, Event> {
    send_cmd(
        model,
        ItemEvent::UpdateVariants {
            id: id.to_string(),
            variants,
        },
    )
}

#[test]
fn update_variants_renames_by_id_keeping_the_row() {
    let mut model = model_with_piece_and_exercise();
    set_ladder(&mut model, &["C", "F"]);
    let c = variant_id(&model, "C");
    let f = variant_id(&model, "F");

    let mut cmd = update_variants(
        &mut model,
        "ex-1",
        vec![edit(Some(c.clone()), "Do"), edit(Some(f.clone()), "F")],
    );

    assert_eq!(
        live_ladder(&model),
        vec![(c, "Do".to_string()), (f, "F".to_string())]
    );
    assert_eq!(
        exercise_variants(&model).len(),
        2,
        "a rename is not remove plus add"
    );
    assert!(model.last_error.is_none());
    assert!(emits_save(&mut cmd, "ex-1"));
}

#[test]
fn update_variants_swaps_two_labels_in_one_write() {
    let mut model = model_with_piece_and_exercise();
    set_ladder(&mut model, &["C", "F"]);
    let c = variant_id(&model, "C");
    let f = variant_id(&model, "F");

    let _ = update_variants(
        &mut model,
        "ex-1",
        vec![edit(Some(c.clone()), "F"), edit(Some(f.clone()), "C")],
    );

    assert!(model.last_error.is_none(), "{:?}", model.last_error);
    assert_eq!(
        live_ladder(&model),
        vec![(c, "F".to_string()), (f, "C".to_string())]
    );
}

#[test]
fn update_variants_adds_removes_and_reorders_together() {
    let mut model = model_with_piece_and_exercise();
    set_ladder(&mut model, &["C", "F", "G"]);
    let c = variant_id(&model, "C");
    let f = variant_id(&model, "F");
    let g = variant_id(&model, "G");

    let _ = update_variants(
        &mut model,
        "ex-1",
        vec![
            edit(Some(g.clone()), "G"),
            edit(None, "A"),
            edit(Some(c.clone()), "C"),
        ],
    );

    let live = live_ladder(&model);
    let labels: Vec<&str> = live.iter().map(|(_, l)| l.as_str()).collect();
    assert_eq!(labels, ["G", "A", "C"]);
    assert_eq!(live[0].0, g);
    assert_eq!(live[2].0, c);
    let removed = exercise_variants(&model)
        .iter()
        .find(|v| v.id == f)
        .unwrap();
    assert!(removed.deleted_at.is_some(), "F is tombstoned, not dropped");
}

#[test]
fn update_variants_reads_a_bare_label_as_the_existing_row() {
    let mut model = model_with_piece_and_exercise();
    set_ladder(&mut model, &["C"]);
    let c = variant_id(&model, "C");

    let _ = update_variants(&mut model, "ex-1", vec![edit(None, "c")]);

    assert_eq!(live_ladder(&model), vec![(c, "c".to_string())]);
}

#[test]
fn update_variants_resurrects_a_removed_label() {
    let mut model = model_with_piece_and_exercise();
    set_ladder(&mut model, &["C", "F"]);
    let c = variant_id(&model, "C");
    let f = variant_id(&model, "F");
    set_ladder(&mut model, &["C"]);

    let _ = update_variants(
        &mut model,
        "ex-1",
        vec![edit(None, "F"), edit(Some(c.clone()), "C")],
    );

    assert_eq!(
        live_ladder(&model),
        vec![(f, "F".to_string()), (c, "C".to_string())]
    );
}

#[test]
fn update_variants_rejects_a_duplicate_and_marks_the_section() {
    let mut model = model_with_piece_and_exercise();
    set_ladder(&mut model, &["C", "F"]);
    let c = variant_id(&model, "C");
    let f = variant_id(&model, "F");

    let mut cmd = update_variants(
        &mut model,
        "ex-1",
        vec![edit(Some(c.clone()), "f"), edit(Some(f.clone()), "F")],
    );

    assert!(model.last_error.is_some());
    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Piece {
            field: FormErrorField::Variations
        })
    );
    assert_eq!(
        live_ladder(&model),
        vec![(c, "C".to_string()), (f, "F".to_string())]
    );
    assert!(!emits_save(&mut cmd, "ex-1"));
}

#[test]
fn update_variants_folds_the_key_into_the_first_rung() {
    let mut model = model_with_piece_and_exercise();
    model.items.iter_mut().find(|i| i.id == "ex-1").unwrap().key = Some("D".to_string());

    let _ = update_variants(&mut model, "ex-1", vec![edit(None, "C")]);

    let labels: Vec<String> = live_ladder(&model).into_iter().map(|(_, l)| l).collect();
    assert_eq!(labels, ["D", "C"]);
    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert!(
        ex.key.is_none(),
        "an exercise in several keys has no single key"
    );
}

#[test]
fn update_variants_is_a_noop_without_a_save_when_unchanged() {
    let mut model = model_with_piece_and_exercise();
    set_ladder(&mut model, &["C", "F"]);
    let c = variant_id(&model, "C");
    let f = variant_id(&model, "F");
    let before = model
        .items
        .iter()
        .find(|i| i.id == "ex-1")
        .unwrap()
        .updated_at;

    let mut cmd = update_variants(
        &mut model,
        "ex-1",
        vec![edit(Some(c), "C"), edit(Some(f), "F")],
    );

    let ex = model.items.iter().find(|i| i.id == "ex-1").unwrap();
    assert_eq!(ex.updated_at, before);
    assert!(!emits_save(&mut cmd, "ex-1"));
    assert!(model.last_error.is_none());
}

#[test]
fn update_variants_rejects_a_piece_host() {
    let mut model = model_with_piece_and_exercise();

    let _ = update_variants(&mut model, "piece-1", vec![edit(None, "C")]);

    let piece = model.items.iter().find(|i| i.id == "piece-1").unwrap();
    assert!(piece.variants.is_empty());
    assert!(model.last_error.is_some());
}

// ── Error targets on the item form (#1831) ──

#[test]
fn set_variants_and_rename_variant_mark_the_variations_section() {
    let mut model = model_with_piece_and_exercise();
    set_ladder(&mut model, &["C", "F"]);
    let c = variant_id(&model, "C");

    set_ladder(&mut model, &["C", "c"]);
    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Piece {
            field: FormErrorField::Variations
        }),
        "SetVariants"
    );

    send(
        &mut model,
        ItemEvent::RenameVariant {
            item_id: "ex-1".to_string(),
            variant_id: c,
            new_label: "f".to_string(),
        },
    );
    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Piece {
            field: FormErrorField::Variations
        }),
        "RenameVariant"
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
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variant_labels: vec!["C".to_string(), "c".to_string()],
        }),
    );
    assert_eq!(
        model.last_error_target,
        Some(FormErrorTarget::Piece {
            field: FormErrorField::Variations
        }),
        "a duplicate rung"
    );

    send(
        &mut model,
        ItemEvent::Add(CreateItem {
            title: "Scale".to_string(),
            kind: ItemKind::Exercise,
            composer: Some("x".repeat(201)),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variant_labels: vec![],
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
                modality: None,
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
