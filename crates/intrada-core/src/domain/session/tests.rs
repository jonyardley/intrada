use super::building::*;
use super::plays::*;
use super::summary::SAVE_FAILED;
use super::*;
use crate::app::AppEffect;
use crate::app::Intrada;
use crate::domain::item::Item;
use crux_core::App;

fn model_with_library() -> Model {
    let now = Utc::now();
    Model {
        items: vec![
            Item {
                id: "piece-1".to_string(),
                title: "Moonlight Sonata".to_string(),
                kind: ItemKind::Piece,
                composer: Some("Beethoven".to_string()),
                key: None,
                modality: None,
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
                linked_exercise_ids: vec![],
                priority: false,
                chord_chart: None,
                variants: vec![],
                photo_id: None,
                metre: None,
            },
            Item {
                id: "piece-2".to_string(),
                title: "Clair de Lune".to_string(),
                kind: ItemKind::Piece,
                composer: Some("Debussy".to_string()),
                key: None,
                modality: None,
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
                linked_exercise_ids: vec![],
                priority: false,
                chord_chart: None,
                variants: vec![],
                photo_id: None,
                metre: None,
            },
            Item {
                id: "exercise-1".to_string(),
                title: "C Major Scale".to_string(),
                kind: ItemKind::Exercise,
                composer: None,
                key: None,
                modality: None,
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
                linked_exercise_ids: vec![],
                priority: false,
                chord_chart: None,
                variants: vec![],
                photo_id: None,
                metre: None,
            },
        ]
        .into(),
        ..Default::default()
    }
}

fn update(model: &mut Model, event: Event) {
    let app = Intrada;
    let _cmd = app.update(event, model);
}

// --- Block grouping (related exercises travel with a piece) ---

fn linked_model() -> Model {
    let now = Utc::now();
    let mk = |id: &str, title: &str, kind: ItemKind, linked: &[&str]| Item {
        id: id.to_string(),
        title: title.to_string(),
        kind,
        composer: None,
        key: None,
        modality: None,
        tempo: None,
        notes: None,
        tags: vec![],
        linked_exercise_ids: linked.iter().map(|s| s.to_string()).collect(),
        created_at: now,
        updated_at: now,
        priority: false,
        chord_chart: None,
        variants: vec![],
        photo_id: None,
        metre: None,
    };
    Model {
        items: vec![
            mk("piece-P", "Sonata", ItemKind::Piece, &["ex-A", "ex-B"]),
            mk("piece-Q", "Nocturne", ItemKind::Piece, &[]),
            mk("piece-R", "Etude", ItemKind::Piece, &["ex-C"]),
            mk("ex-A", "Scales", ItemKind::Exercise, &[]),
            mk("ex-B", "Arpeggios", ItemKind::Exercise, &[]),
            mk("ex-C", "Sight-reading", ItemKind::Exercise, &[]),
            mk("ex-D", "Trills", ItemKind::Exercise, &[]),
        ]
        .into(),
        ..Default::default()
    }
}

fn building_entries(model: &Model) -> &[SetlistEntry] {
    match &model.session_status {
        SessionStatus::Building(b) => &b.entries,
        _ => panic!("expected Building state"),
    }
}

fn session_entries(model: &Model) -> &[SetlistEntry] {
    match &model.session_status {
        SessionStatus::Building(b) => &b.entries,
        SessionStatus::Active(a) => &a.entries,
        SessionStatus::Summary(s) => &s.entries,
        SessionStatus::Idle => panic!("expected a session"),
    }
}

fn play_of(entry: &SetlistEntry) -> &VariationPlay {
    entry.open_play().expect("the entry has an open play")
}

fn first_play_id(model: &Model, entry_id: &str) -> String {
    session_entries(model)
        .iter()
        .find(|e| e.id == entry_id)
        .expect("the entry is in the session")
        .plays
        .first()
        .expect("the entry has a play")
        .id
        .clone()
}

fn ids(model: &Model) -> Vec<String> {
    building_entries(model)
        .iter()
        .map(|e| e.item_id.clone())
        .collect()
}

fn add(model: &mut Model, item_id: &str) {
    update(
        model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: item_id.to_string(),
        }),
    );
}

fn group_of(model: &Model, item_id: &str) -> Option<String> {
    building_entries(model)
        .iter()
        .find(|e| e.item_id == item_id)
        .and_then(|e| e.group_id.clone())
}

#[test]
fn start_building_with_seeds_exercise_from_idle() {
    let mut m = linked_model();
    update(
        &mut m,
        Event::Session(SessionEvent::StartBuildingWith {
            item_id: "ex-C".to_string(),
        }),
    );
    let e = building_entries(&m);
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].item_id, "ex-C");
    assert_eq!(e[0].group_id, None);
    assert_eq!(m.last_error, None);
}

#[test]
fn start_building_with_piece_forms_block() {
    let mut m = linked_model();
    update(
        &mut m,
        Event::Session(SessionEvent::StartBuildingWith {
            item_id: "piece-P".to_string(),
        }),
    );
    assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
    let e = building_entries(&m);
    assert!(e[0].group_id.is_some(), "block has a group_id");
    assert!(e.iter().all(|x| x.group_id == e[0].group_id));
}

#[test]
fn start_building_with_rejects_when_not_idle() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "ex-A");
    update(
        &mut m,
        Event::Session(SessionEvent::StartBuildingWith {
            item_id: "ex-C".to_string(),
        }),
    );
    assert_eq!(
        m.last_error.as_deref(),
        Some("A practice is already in progress")
    );
    assert_eq!(ids(&m), ["ex-A"], "existing setlist untouched");
}

#[test]
fn start_building_with_unknown_item_stays_idle() {
    let mut m = linked_model();
    update(
        &mut m,
        Event::Session(SessionEvent::StartBuildingWith {
            item_id: "nope".to_string(),
        }),
    );
    assert!(m.last_error.is_some());
    assert!(
        matches!(m.session_status, SessionStatus::Idle),
        "a failed seed must not leave an empty building session"
    );
}

// ── The Up next card's seeding event (#1082) ─────────────────────

fn suggestion_model() -> Model {
    let mut m = linked_model();
    // Pin the anchor so these tests assert the seeding, not the ranking
    // (which `suggestion.rs` covers).
    for item in m.items.iter_mut() {
        if item.id == "piece-P" {
            item.priority = true;
        }
    }
    m
}

fn start_from_suggestion(model: &mut Model) {
    update(
        model,
        Event::Session(SessionEvent::StartBuildingFromSuggestion { now: Utc::now() }),
    );
}

#[test]
fn start_building_from_suggestion_seeds_the_derived_block() {
    let mut m = suggestion_model();
    start_from_suggestion(&mut m);

    assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
    let e = building_entries(&m);
    assert!(e[0].group_id.is_some(), "the seeded block has a group_id");
    assert!(
        e.iter().all(|x| x.group_id == e[0].group_id),
        "one block, not three standalone entries"
    );
    assert_eq!(m.last_error, None);
}

#[test]
fn start_building_from_suggestion_caps_the_block_at_three_items() {
    let mut m = suggestion_model();
    for item in m.items.iter_mut() {
        if item.id == "piece-P" {
            item.linked_exercise_ids = vec![
                "ex-A".to_string(),
                "ex-B".to_string(),
                "ex-C".to_string(),
                "ex-D".to_string(),
            ];
        }
    }
    start_from_suggestion(&mut m);

    assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
}

#[test]
fn start_building_from_suggestion_rejects_when_not_idle() {
    let mut m = suggestion_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "ex-C");
    start_from_suggestion(&mut m);

    assert_eq!(
        m.last_error.as_deref(),
        Some("A practice is already in progress")
    );
    assert_eq!(ids(&m), ["ex-C"], "existing setlist untouched");
}

#[test]
fn start_building_from_suggestion_is_a_no_op_with_nothing_to_suggest() {
    // No piece has a related exercise linked, so there is no block to resume.
    let mut m = linked_model();
    m.items.retain(|i| i.id == "piece-Q" || i.id == "ex-D");
    start_from_suggestion(&mut m);

    assert!(
        matches!(m.session_status, SessionStatus::Idle),
        "an empty derivation must not leave an empty building session"
    );
    assert_eq!(m.last_error, None, "nothing to suggest is not an error");
}

#[test]
fn start_building_from_suggestion_attributes_the_current_step() {
    let mut m = suggestion_model();
    let step_id = "v-C".to_string();
    for item in m.items.iter_mut() {
        if item.id == "ex-A" {
            item.variants = vec![crate::domain::variant::Variant {
                id: step_id.clone(),
                label: "C".to_string(),
                position: 0,
                updated_at: Utc::now(),
                deleted_at: None,
            }];
        }
    }
    start_from_suggestion(&mut m);

    let e = building_entries(&m);
    let seeded = e.iter().find(|x| x.item_id == "ex-A").expect("ex-A seeded");
    assert_eq!(
        seeded.planned_variation_id.as_deref(),
        Some(step_id.as_str())
    );
    let unladdered = e.iter().find(|x| x.item_id == "ex-B").expect("ex-B seeded");
    assert_eq!(unladdered.planned_variation_id, None);
}

#[test]
fn start_building_from_suggestion_makes_no_network_call() {
    let mut m = suggestion_model();
    let app = Intrada;
    let mut cmd = app.update(
        Event::Session(SessionEvent::StartBuildingFromSuggestion { now: Utc::now() }),
        &mut m,
    );
    assert!(
        cmd.effects().all(|e| matches!(e, Effect::Render(_))),
        "the suggestion is derived from data already in the model: no HTTP, \
         no storage read (invariant 1)"
    );
}

#[test]
fn start_building_from_suggestion_round_trips_on_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(Event::Session(
        SessionEvent::StartBuildingFromSuggestion { now: Utc::now() },
    ));
}

#[test]
fn prepare_reflection_round_trips_on_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(Event::Session(SessionEvent::PrepareReflection {
        now: Utc::now(),
        reading: TempoReading::silent(),
    }));
}

// ── "Practise your priorities" (#981) ────────────────────────────

fn star(model: &mut Model, starred: &[&str]) {
    for item in model.items.iter_mut() {
        if starred.contains(&item.id.as_str()) {
            item.priority = true;
        }
    }
}

/// Same mark and return count on every fixture, so the expected interval
/// is identical and only the gap decides the order.
fn practised_days_ago(model: &mut Model, item_id: &str, days: i64) {
    model.practice_summaries.insert(
        item_id.to_string(),
        crate::model::ItemPracticeSummary {
            session_count: 4,
            latest_score: Some(6),
            last_practiced_at: Some((Utc::now() - chrono::Duration::days(days)).to_rfc3339()),
            ..crate::model::ItemPracticeSummary::fixture()
        },
    );
}

fn start_with_priorities(model: &mut Model) {
    update(
        model,
        Event::Session(SessionEvent::StartBuildingWithPriorities { now: Utc::now() }),
    );
}

#[test]
fn start_building_with_priorities_seeds_every_starred_item() {
    let mut m = linked_model();
    star(&mut m, &["piece-Q", "ex-D"]);
    start_with_priorities(&mut m);

    assert_eq!(
        ids(&m),
        ["piece-Q", "ex-D"],
        "both starred items reach the setlist; the ordering itself is pinned in priorities.rs"
    );
    assert_eq!(m.last_error, None);
}

#[test]
fn start_building_with_priorities_seeds_least_ready_first() {
    let mut m = linked_model();
    star(&mut m, &["piece-Q", "ex-D"]);
    practised_days_ago(&mut m, "piece-Q", 2);
    practised_days_ago(&mut m, "ex-D", 90);
    start_with_priorities(&mut m);

    assert_eq!(
        ids(&m),
        ["ex-D", "piece-Q"],
        "the longer-neglected item leads, against both library order and the alphabet"
    );
}

#[test]
fn start_building_with_priorities_brings_a_starred_pieces_block() {
    let mut m = linked_model();
    star(&mut m, &["piece-P"]);
    start_with_priorities(&mut m);

    assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
    let e = building_entries(&m);
    assert!(e[0].group_id.is_some(), "the seeded piece keeps its block");
    assert!(e.iter().all(|x| x.group_id == e[0].group_id));
}

#[test]
fn start_building_with_priorities_seeds_a_starred_exercise_once() {
    let mut m = linked_model();
    star(&mut m, &["piece-P", "ex-A"]);
    start_with_priorities(&mut m);

    assert_eq!(
        ids(&m).iter().filter(|id| *id == "ex-A").count(),
        1,
        "starred in its own right and pulled in by its piece is still one entry"
    );
}

#[test]
fn start_building_with_priorities_rejects_when_not_idle() {
    let mut m = linked_model();
    star(&mut m, &["piece-Q"]);
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "ex-C");
    start_with_priorities(&mut m);

    assert_eq!(
        m.last_error.as_deref(),
        Some("A practice is already in progress")
    );
    assert_eq!(ids(&m), ["ex-C"], "existing setlist untouched");
}

#[test]
fn start_building_with_priorities_is_a_no_op_with_nothing_starred() {
    let mut m = linked_model();
    start_with_priorities(&mut m);

    assert!(
        matches!(m.session_status, SessionStatus::Idle),
        "an empty priority set must not leave an empty building session"
    );
    assert_eq!(m.last_error, None, "nothing starred is not an error");
}

#[test]
fn start_building_with_priorities_makes_no_network_call() {
    let mut m = linked_model();
    star(&mut m, &["piece-P"]);
    let app = Intrada;
    let mut cmd = app.update(
        Event::Session(SessionEvent::StartBuildingWithPriorities { now: Utc::now() }),
        &mut m,
    );
    assert!(
        cmd.effects().all(|e| matches!(e, Effect::Render(_))),
        "the priority set is derived from data already in the model: no HTTP, \
         no storage read (invariant 1)"
    );
}

#[test]
fn start_building_with_priorities_round_trips_on_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(Event::Session(
        SessionEvent::StartBuildingWithPriorities { now: Utc::now() },
    ));
}

#[test]
fn add_to_setlist_is_idempotent_by_item_id() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "ex-C");
    add(&mut m, "ex-C");
    assert_eq!(ids(&m), ["ex-C"], "second add of the same item is a no-op");
    assert_eq!(m.last_error, None);
}

#[test]
fn add_piece_twice_does_not_duplicate_block() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P");
    add(&mut m, "piece-P");
    assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
    assert_eq!(m.last_error, None);
}

#[test]
fn re_adding_present_piece_is_a_full_no_op_even_with_new_relateds() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-Q");
    if let Some(piece) = m.items.iter_mut().find(|i| i.id == "piece-Q") {
        piece.linked_exercise_ids = vec!["ex-C".to_string()];
    }
    add(&mut m, "piece-Q");
    assert_eq!(
        ids(&m),
        ["piece-Q"],
        "membership no-op wins; newly linked relateds come in by removing and re-adding"
    );
}

#[test]
fn add_piece_pulls_related_into_a_block_related_first() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P");
    let e = building_entries(&m);
    assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
    let g = e[0].group_id.clone();
    assert!(g.is_some(), "block has a group_id");
    assert!(
        e.iter().all(|x| x.group_id == g),
        "all three share the group"
    );
    assert_eq!((e[0].position, e[1].position, e[2].position), (0, 1, 2));
}

#[test]
fn add_piece_without_related_is_standalone() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-Q");
    let e = building_entries(&m);
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].group_id, None);
}

#[test]
fn add_exercise_directly_is_standalone() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "ex-C");
    let e = building_entries(&m);
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].group_id, None);
}

#[test]
fn add_piece_skips_already_present_related() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "ex-A");
    add(&mut m, "piece-P");
    let e = building_entries(&m);
    assert_eq!(
        e.iter().filter(|x| x.item_id == "ex-A").count(),
        1,
        "ex-A not duplicated"
    );
    assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
    assert_eq!(
        e[0].group_id, None,
        "the pre-existing ex-A stays standalone"
    );
    assert!(e[1].group_id.is_some());
    assert_eq!(e[1].group_id, e[2].group_id, "ex-B + piece form the block");
}

#[test]
fn add_exercise_to_block_inserts_before_anchor_piece() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P");
    let g = group_of(&m, "piece-P").unwrap();
    update(
        &mut m,
        Event::Session(SessionEvent::AddExerciseToBlock {
            group_id: g.clone(),
            item_id: "ex-D".to_string(),
        }),
    );
    assert_eq!(m.last_error, None);
    assert_eq!(ids(&m), ["ex-A", "ex-B", "ex-D", "piece-P"]);
    assert_eq!(group_of(&m, "ex-D"), Some(g.clone()));
    assert!(groups_contiguous(building_entries(&m)));
}

#[test]
fn add_exercise_to_block_rejects_unknown_group() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P");
    update(
        &mut m,
        Event::Session(SessionEvent::AddExerciseToBlock {
            group_id: "no-such-group".to_string(),
            item_id: "ex-D".to_string(),
        }),
    );
    assert!(m.last_error.is_some());
    assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"], "setlist untouched");
}

#[test]
fn add_exercise_to_block_rejects_non_exercise_item() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P");
    let g = group_of(&m, "piece-P").unwrap();
    update(
        &mut m,
        Event::Session(SessionEvent::AddExerciseToBlock {
            group_id: g,
            item_id: "piece-Q".to_string(),
        }),
    );
    assert!(m.last_error.is_some());
    assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"], "setlist untouched");
}

#[test]
fn add_exercise_to_block_is_idempotent_by_item_id() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P");
    let g = group_of(&m, "piece-P").unwrap();
    for _ in 0..2 {
        update(
            &mut m,
            Event::Session(SessionEvent::AddExerciseToBlock {
                group_id: g.clone(),
                item_id: "ex-D".to_string(),
            }),
        );
    }
    assert_eq!(m.last_error, None);
    assert_eq!(ids(&m), ["ex-A", "ex-B", "ex-D", "piece-P"]);
}

#[test]
fn remove_block_removes_piece_and_related() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "ex-C");
    add(&mut m, "piece-P");
    let g = group_of(&m, "piece-P").unwrap();
    update(
        &mut m,
        Event::Session(SessionEvent::RemoveBlock { group_id: g }),
    );
    assert!(m.last_error.is_none());
    assert_eq!(ids(&m), ["ex-C"]);
}

#[test]
fn keep_only_piece_drops_related_and_destandalones() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P");
    let g = group_of(&m, "piece-P").unwrap();
    update(
        &mut m,
        Event::Session(SessionEvent::KeepOnlyPiece { group_id: g }),
    );
    let e = building_entries(&m);
    assert_eq!(ids(&m), ["piece-P"]);
    assert_eq!(e[0].group_id, None, "lone piece is no longer a block");
}

#[test]
fn ungroup_block_keeps_items_in_place() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P");
    let g = group_of(&m, "piece-P").unwrap();
    update(
        &mut m,
        Event::Session(SessionEvent::UngroupBlock { group_id: g }),
    );
    let e = building_entries(&m);
    assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
    assert!(e.iter().all(|x| x.group_id.is_none()));
}

#[test]
fn ungroup_all_clears_every_group() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P");
    add(&mut m, "ex-C");
    update(&mut m, Event::Session(SessionEvent::UngroupAllBlocks));
    assert!(building_entries(&m).iter().all(|x| x.group_id.is_none()));
}

#[test]
fn building_view_projects_blocks_and_standalones() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P");
    add(&mut m, "ex-C");
    let vm = Intrada.view(&m);
    let b = vm.building_setlist.expect("building view");
    assert_eq!(b.item_count, 4);
    assert_eq!(b.blocks.len(), 2);
    let block = &b.blocks[0];
    assert!(block.group_id.is_some());
    assert_eq!(block.piece_title.as_deref(), Some("Sonata"));
    assert_eq!(block.related_count, 2);
    assert_eq!(block.entries.len(), 3);
    let solo = &b.blocks[1];
    assert_eq!(solo.group_id, None);
    assert_eq!(solo.piece_title, None);
    assert_eq!(solo.related_count, 0);
}

#[test]
fn two_adjacent_blocks_project_separately() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P"); // block G1: ex-A, ex-B, piece-P
    add(&mut m, "piece-R"); // block G2: ex-C, piece-R
    assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P", "ex-C", "piece-R"]);
    let g1 = group_of(&m, "piece-P");
    let g2 = group_of(&m, "piece-R");
    assert!(g1.is_some() && g2.is_some() && g1 != g2, "distinct blocks");
    let b = Intrada.view(&m).building_setlist.unwrap();
    assert_eq!(b.blocks.len(), 2);
    assert_eq!(b.item_count, 5);
    assert_eq!(b.blocks[0].piece_title.as_deref(), Some("Sonata"));
    assert_eq!(b.blocks[0].related_count, 2);
    assert_eq!(b.blocks[1].piece_title.as_deref(), Some("Etude"));
    assert_eq!(b.blocks[1].related_count, 1);
}

// ── The builder's drag moves (#1957) ──

/// Units in order: [ex-A ex-B piece-P] [ex-D] [ex-C piece-R] [piece-Q].
fn four_unit_model() -> Model {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P");
    add(&mut m, "ex-D");
    add(&mut m, "piece-R");
    add(&mut m, "piece-Q");
    assert_eq!(
        ids(&m),
        ["ex-A", "ex-B", "piece-P", "ex-D", "ex-C", "piece-R", "piece-Q"]
    );
    m
}

fn entry_id_of(model: &Model, item_id: &str) -> String {
    building_entries(model)
        .iter()
        .find(|e| e.item_id == item_id)
        .map_or_else(|| format!("no-entry-{item_id}"), |e| e.id.clone())
}

fn move_unit(model: &mut Model, item_id: &str, new_position: usize) {
    let entry_id = entry_id_of(model, item_id);
    update(
        model,
        Event::Session(SessionEvent::MoveUnit {
            entry_id,
            new_position,
        }),
    );
}

fn move_related(model: &mut Model, item_id: &str, new_position: usize) {
    let entry_id = entry_id_of(model, item_id);
    update(
        model,
        Event::Session(SessionEvent::MoveRelated {
            entry_id,
            new_position,
        }),
    );
}

#[test]
fn move_unit_moves_the_unit_holding_the_entry() {
    let cases: &[(&str, usize, [&str; 7])] = &[
        (
            "ex-D",
            0,
            [
                "ex-D", "ex-A", "ex-B", "piece-P", "ex-C", "piece-R", "piece-Q",
            ],
        ),
        (
            "ex-D",
            2,
            [
                "ex-A", "ex-B", "piece-P", "ex-C", "piece-R", "ex-D", "piece-Q",
            ],
        ),
        (
            "piece-Q",
            0,
            [
                "piece-Q", "ex-A", "ex-B", "piece-P", "ex-D", "ex-C", "piece-R",
            ],
        ),
        (
            "ex-B",
            3,
            [
                "ex-D", "ex-C", "piece-R", "piece-Q", "ex-A", "ex-B", "piece-P",
            ],
        ),
        (
            "piece-P",
            1,
            [
                "ex-D", "ex-A", "ex-B", "piece-P", "ex-C", "piece-R", "piece-Q",
            ],
        ),
        (
            "piece-R",
            99,
            [
                "ex-A", "ex-B", "piece-P", "ex-D", "piece-Q", "ex-C", "piece-R",
            ],
        ),
        (
            "ex-D",
            1,
            [
                "ex-A", "ex-B", "piece-P", "ex-D", "ex-C", "piece-R", "piece-Q",
            ],
        ),
    ];
    for (item_id, new_position, expected) in cases {
        let mut m = four_unit_model();
        let groups_before: Vec<_> = building_entries(&m)
            .iter()
            .map(|e| (e.item_id.clone(), e.group_id.clone()))
            .collect();
        move_unit(&mut m, "ex-nowhere", 0);
        assert!(m.last_error.is_some(), "a refused move first");
        move_unit(&mut m, item_id, *new_position);
        assert!(
            m.last_error.is_none(),
            "{item_id} to {new_position}: {:?}",
            m.last_error
        );
        assert_eq!(ids(&m), expected, "{item_id} to {new_position}");
        assert!(groups_contiguous(building_entries(&m)));
        let mut groups_after: Vec<_> = building_entries(&m)
            .iter()
            .map(|e| (e.item_id.clone(), e.group_id.clone()))
            .collect();
        let mut groups_before = groups_before.clone();
        groups_before.sort();
        groups_after.sort();
        assert_eq!(groups_after, groups_before, "no entry changes block");
        let positions: Vec<_> = building_entries(&m).iter().map(|e| e.position).collect();
        assert_eq!(positions, (0..7).collect::<Vec<_>>(), "reindexed");
    }
}

#[test]
fn move_unit_with_an_unknown_entry_is_refused() {
    let mut m = four_unit_model();
    let before = ids(&m);
    move_unit(&mut m, "ex-nowhere", 0);
    assert!(m.last_error.is_some());
    assert_eq!(ids(&m), before);
}

#[test]
fn move_related_moves_an_exercise_within_its_block() {
    let cases: &[(&str, usize, [&str; 3])] = &[
        ("ex-B", 0, ["ex-B", "ex-A", "piece-P"]),
        ("ex-A", 1, ["ex-B", "ex-A", "piece-P"]),
        ("ex-A", 0, ["ex-A", "ex-B", "piece-P"]),
        ("ex-B", 1, ["ex-A", "ex-B", "piece-P"]),
    ];
    for (item_id, new_position, expected_block) in cases {
        let mut m = four_unit_model();
        move_related(&mut m, "piece-P", 0);
        assert!(m.last_error.is_some(), "a refused move first");
        move_related(&mut m, item_id, *new_position);
        assert!(
            m.last_error.is_none(),
            "{item_id} to {new_position}: {:?}",
            m.last_error
        );
        let got = ids(&m);
        assert_eq!(&got[..3], expected_block, "{item_id} to {new_position}");
        assert_eq!(
            &got[3..],
            ["ex-D", "ex-C", "piece-R", "piece-Q"],
            "the other units stay put"
        );
        assert!(groups_contiguous(building_entries(&m)));
        let group = group_of(&m, "piece-P");
        assert!(group.is_some());
        assert!(building_entries(&m)[..3]
            .iter()
            .all(|e| e.group_id == group));
    }
}

#[test]
fn move_related_refuses_what_is_not_a_related_exercise_move() {
    let cases: &[(&str, usize, &str)] = &[
        ("ex-A", 2, "past the related run, onto the anchor piece"),
        ("ex-B", 9, "far past the run"),
        ("piece-P", 0, "the anchor piece itself"),
        ("ex-D", 0, "a standalone exercise"),
        ("ex-nowhere", 0, "an unknown entry"),
    ];
    for (item_id, new_position, why) in cases {
        let mut m = four_unit_model();
        let before = ids(&m);
        move_related(&mut m, item_id, *new_position);
        assert!(m.last_error.is_some(), "{why}");
        assert_eq!(ids(&m), before, "{why}: order unchanged");
    }
}

#[test]
fn moves_outside_building_are_refused() {
    let mut m = linked_model();
    update(
        &mut m,
        Event::Session(SessionEvent::MoveUnit {
            entry_id: "e".to_string(),
            new_position: 0,
        }),
    );
    assert!(m.last_error.is_some());
    m.last_error = None;
    update(
        &mut m,
        Event::Session(SessionEvent::MoveRelated {
            entry_id: "e".to_string(),
            new_position: 0,
        }),
    );
    assert!(m.last_error.is_some());
}

#[test]
fn drag_moves_round_trip_on_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(Event::Session(SessionEvent::MoveUnit {
        entry_id: "e1".to_string(),
        new_position: 3,
    }));
    crate::domain::types::assert_round_trips(Event::Session(SessionEvent::MoveRelated {
        entry_id: "e2".to_string(),
        new_position: 1,
    }));
}

#[test]
fn removing_the_piece_dissolves_the_block_to_standalone() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P"); // ex-A, ex-B, piece-P (group)
    let piece = building_entries(&m)
        .iter()
        .find(|e| e.item_id == "piece-P")
        .unwrap()
        .id
        .clone();
    update(
        &mut m,
        Event::Session(SessionEvent::RemoveFromSetlist { entry_id: piece }),
    );
    assert!(m.last_error.is_none());
    assert_eq!(ids(&m), ["ex-A", "ex-B"]);
    assert!(
        building_entries(&m).iter().all(|x| x.group_id.is_none()),
        "related become standalone when their piece is removed"
    );
    let b = Intrada.view(&m).building_setlist.unwrap();
    assert_eq!(
        b.blocks.len(),
        2,
        "two standalone units, not one pieceless block"
    );
}

#[test]
fn removing_a_related_keeps_the_block() {
    let mut m = linked_model();
    update(&mut m, Event::Session(SessionEvent::StartBuilding));
    add(&mut m, "piece-P"); // ex-A, ex-B, piece-P
    let ex_a = building_entries(&m)[0].id.clone();
    update(
        &mut m,
        Event::Session(SessionEvent::RemoveFromSetlist { entry_id: ex_a }),
    );
    assert!(m.last_error.is_none());
    assert_eq!(ids(&m), ["ex-B", "piece-P"]);
    let e = building_entries(&m);
    assert!(
        e[0].group_id.is_some() && e[0].group_id == e[1].group_id,
        "the piece + remaining related stay one block"
    );
}

// --- Building Phase Tests ---

#[test]
fn test_start_building() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));

    assert!(model.last_error.is_none());
    assert!(matches!(model.session_status, SessionStatus::Building(_)));
}

#[test]
fn test_start_building_when_already_building() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(&mut model, Event::Session(SessionEvent::StartBuilding));

    assert!(model.last_error.is_some());
}

#[test]
fn test_add_to_setlist() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Building(ref b) = model.session_status {
        assert_eq!(b.entries.len(), 1);
        assert_eq!(b.entries[0].item_title, "Moonlight Sonata");
        assert_eq!(b.entries[0].item_type, ItemKind::Piece);
        assert_eq!(b.entries[0].position, 0);
    } else {
        panic!("Expected Building state");
    }
}

#[test]
fn test_add_to_setlist_item_not_found() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "nonexistent".to_string(),
        }),
    );

    assert!(model.last_error.is_some());
}

#[test]
fn test_remove_from_setlist() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-2".to_string(),
        }),
    );

    let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
        b.entries[0].id.clone()
    } else {
        panic!("Expected Building state");
    };

    update(
        &mut model,
        Event::Session(SessionEvent::RemoveFromSetlist { entry_id }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Building(ref b) = model.session_status {
        assert_eq!(b.entries.len(), 1);
        assert_eq!(b.entries[0].item_title, "Clair de Lune");
        assert_eq!(b.entries[0].position, 0); // Re-indexed
    } else {
        panic!("Expected Building state");
    }
}

#[test]
fn test_start_session_empty_setlist() {
    let mut model = model_with_library();
    let now = Utc::now();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );

    assert!(model.last_error.is_some());
    assert!(matches!(model.session_status, SessionStatus::Building(_)));
}

#[test]
fn test_start_session_with_items() {
    let mut model = model_with_library();
    let now = Utc::now();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Active(ref active) = model.session_status {
        assert_eq!(active.current_index, 0);
        assert_eq!(active.entries.len(), 1);
        assert_eq!(active.session_started_at, now);
    } else {
        panic!("Expected Active state");
    }
}

// --- StartSession variation seeding tests (#1758) ---

#[test]
fn test_start_session_seeds_the_first_live_variation_when_no_plan_is_set() {
    let (mut model, entry_id) = model_with_exercise_building();
    let now = Utc::now();

    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );

    let entry = session_entries(&model)
        .iter()
        .find(|e| e.id == entry_id)
        .expect("the entry is in the session");
    assert_eq!(play_of(entry).variation_id, Some("v-c".to_string()));
}

#[test]
fn test_start_session_keeps_the_planned_variation_over_the_first_live_one() {
    let (mut model, entry_id) = model_with_exercise_building();
    update(
        &mut model,
        Event::Session(SessionEvent::SetEntryVariant {
            entry_id: entry_id.clone(),
            variant_id: Some("v-f".to_string()),
        }),
    );
    let now = Utc::now();

    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );

    let entry = session_entries(&model)
        .iter()
        .find(|e| e.id == entry_id)
        .expect("the entry is in the session");
    assert_eq!(play_of(entry).variation_id, Some("v-f".to_string()));
}

#[test]
fn test_start_session_leaves_a_piece_unattributed() {
    let mut model = model_with_library();
    let now = Utc::now();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );

    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );

    let entry = &session_entries(&model)[0];
    assert_eq!(play_of(entry).variation_id, None);
}

#[test]
fn test_start_session_leaves_an_all_tombstoned_ladder_unattributed() {
    let (mut model, entry_id) = model_with_exercise_building();
    model
        .items
        .iter_mut()
        .find(|i| i.id == "exercise-1")
        .expect("the library fixture has exercise-1")
        .variants
        .iter_mut()
        .for_each(|v| v.deleted_at = Some(Utc::now()));
    let now = Utc::now();

    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );

    let entry = session_entries(&model)
        .iter()
        .find(|e| e.id == entry_id)
        .expect("the entry is in the session");
    assert_eq!(play_of(entry).variation_id, None);
}

#[test]
fn test_start_session_seeds_by_position_not_by_vec_order() {
    let mut model = model_with_library();
    let now = Utc::now();
    let ex = model
        .items
        .iter_mut()
        .find(|i| i.id == "exercise-1")
        .expect("the library fixture has exercise-1");
    // v-d sits first in the vec but v-c has the lower position: the seed
    // must read position, not vec order.
    ex.variants = vec![
        crate::domain::variant::Variant {
            id: "v-d".to_string(),
            label: "D".to_string(),
            position: 1,
            updated_at: now,
            deleted_at: None,
        },
        crate::domain::variant::Variant {
            id: "v-c".to_string(),
            label: "C".to_string(),
            position: 0,
            updated_at: now,
            deleted_at: None,
        },
    ];
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "exercise-1".to_string(),
        }),
    );

    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );

    let entry = &session_entries(&model)[0];
    assert_eq!(play_of(entry).variation_id, Some("v-c".to_string()));
}

#[test]
fn test_switching_to_the_seeded_default_writes_nothing() {
    let (mut model, entry_id) = model_with_exercise_building();
    let now = Utc::now();
    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );

    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id: entry_id.clone(),
            variation_id: Some("v-c".to_string()),
            now: now + chrono::Duration::seconds(10),
            reading: TempoReading::silent(),
        }),
    );

    let entry = session_entries(&model)
        .iter()
        .find(|e| e.id == entry_id)
        .expect("the entry is in the session");
    assert_eq!(entry.plays.len(), 1, "the seeded default was already open");
}

#[test]
fn test_next_item_seeds_the_first_live_variation_for_the_new_entry() {
    let mut model = model_with_library();
    give_exercise_a_ladder(&mut model);
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "exercise-1".to_string(),
        }),
    );
    let now = Utc::now();
    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: now + chrono::Duration::seconds(30),
            next_item_started_at: now + chrono::Duration::seconds(30),
            reading: TempoReading::silent(),
        }),
    );

    let entries = session_entries(&model);
    assert_eq!(play_of(&entries[1]).variation_id, Some("v-c".to_string()));
}

#[test]
fn test_cancel_building() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(&mut model, Event::Session(SessionEvent::CancelBuilding));

    assert!(model.last_error.is_none());
    assert!(matches!(model.session_status, SessionStatus::Idle));
}

#[test]
fn test_cancel_building_when_idle_is_noop_success() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::CancelBuilding));

    assert!(model.last_error.is_none());
    assert!(matches!(model.session_status, SessionStatus::Idle));
}

#[test]
fn test_cancel_building_is_idempotent_when_called_twice() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(&mut model, Event::Session(SessionEvent::CancelBuilding));
    update(&mut model, Event::Session(SessionEvent::CancelBuilding));

    assert!(model.last_error.is_none());
    assert!(matches!(model.session_status, SessionStatus::Idle));
}

#[test]
fn test_cancel_building_from_active_is_a_wrong_state_error() {
    // Cancelling the builder must not silently nuke an Active session (#944).
    let (mut model, _) = model_with_active_session(2);

    update(&mut model, Event::Session(SessionEvent::CancelBuilding));

    assert_eq!(model.last_error.as_deref(), Some("Not in building state"));
    assert!(matches!(model.session_status, SessionStatus::Active(_)));
}

// --- Active Phase Tests ---

fn model_with_active_session(item_count: usize) -> (Model, DateTime<Utc>) {
    let mut model = model_with_library();
    let now = Utc::now();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));

    let items = ["piece-1", "piece-2", "exercise-1"];
    for item_id in items.iter().take(item_count.min(3)) {
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: (*item_id).to_string(),
            }),
        );
    }

    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );
    (model, now)
}

#[test]
fn test_next_item() {
    let (mut model, start) = model_with_active_session(3);
    let now = start + chrono::Duration::seconds(30);

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now,
            next_item_started_at: now,
            reading: TempoReading::silent(),
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Active(ref active) = model.session_status {
        assert_eq!(active.current_index, 1);
        assert_eq!(active.entries[0].duration_secs, 30);
        assert_eq!(active.entries[0].status, EntryStatus::Completed);
    } else {
        panic!("Expected Active state");
    }
}

/// A reflection sheet's dwell must not read as practice on the item that
/// follows (#1758).
#[test]
fn next_item_started_at_not_now_opens_the_next_item_clock() {
    let (mut model, start) = model_with_active_session(3);
    let closed_at = start + chrono::Duration::seconds(30);
    let opened_at = closed_at + chrono::Duration::seconds(45);

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: closed_at,
            next_item_started_at: opened_at,
            reading: TempoReading::silent(),
        }),
    );

    {
        let SessionStatus::Active(ref active) = model.session_status else {
            panic!("Expected Active state");
        };
        assert_eq!(
            active.entries[0].duration_secs, 30,
            "the finished item's duration uses the closing instant, not the dwell"
        );
    }

    // Ten real seconds on item two, no reflection sheet in between this
    // time: if item two's clock had started at `closed_at` (the bug),
    // this would read 55, the 45 second dwell included.
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: opened_at + chrono::Duration::seconds(10),
            next_item_started_at: opened_at + chrono::Duration::seconds(10),
            reading: TempoReading::silent(),
        }),
    );
    let SessionStatus::Active(ref active) = model.session_status else {
        panic!("Expected Active state");
    };
    assert_eq!(
        active.entries[1].duration_secs, 10,
        "item two's own duration excludes the dwell on item one's sheet"
    );
    assert_eq!(
        active.entries[1].plays[0].seconds, 10,
        "item two's first play is stamped from the advance, not from item one's close"
    );
}

#[test]
fn test_next_item_on_last_transitions_to_summary() {
    let (mut model, start) = model_with_active_session(1);
    let now = start + chrono::Duration::seconds(60);

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now,
            next_item_started_at: now,
            reading: TempoReading::silent(),
        }),
    );

    assert!(model.last_error.is_none());
    assert!(matches!(model.session_status, SessionStatus::Summary(_)));
}

#[test]
fn test_next_item_on_the_last_item_records_both_durations() {
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(30);
    let t2 = t1 + chrono::Duration::seconds(45);

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading: TempoReading::silent(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t2,
            next_item_started_at: t2,
            reading: TempoReading::silent(),
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Summary(ref summary) = model.session_status {
        assert_eq!(summary.entries[0].duration_secs, 30);
        assert_eq!(summary.entries[1].duration_secs, 45);
        assert_eq!(summary.completion_status, CompletionStatus::Completed);
    } else {
        panic!("Expected Summary state");
    }
}

#[test]
fn test_end_session_early() {
    let (mut model, start) = model_with_active_session(3);
    let t1 = start + chrono::Duration::seconds(30);
    let t2 = t1 + chrono::Duration::seconds(20);

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading: TempoReading::silent(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::EndSessionEarly {
            now: t2,
            reading: TempoReading::silent(),
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Summary(ref summary) = model.session_status {
        assert_eq!(summary.entries[0].duration_secs, 30);
        assert_eq!(summary.entries[0].status, EntryStatus::Completed);
        assert_eq!(summary.entries[1].duration_secs, 20);
        assert_eq!(summary.entries[1].status, EntryStatus::Completed);
        assert_eq!(summary.entries[2].duration_secs, 0);
        assert_eq!(summary.entries[2].status, EntryStatus::NotAttempted);
        assert_eq!(summary.completion_status, CompletionStatus::EndedEarly);
    } else {
        panic!("Expected Summary state");
    }
}

#[test]
fn test_skip_item() {
    let (mut model, start) = model_with_active_session(3);
    let now = start + chrono::Duration::seconds(10);

    update(&mut model, Event::Session(SessionEvent::SkipItem { now }));

    assert!(model.last_error.is_none());
    if let SessionStatus::Active(ref active) = model.session_status {
        assert_eq!(active.current_index, 1);
        assert_eq!(active.entries[0].duration_secs, 0);
        assert_eq!(active.entries[0].status, EntryStatus::Skipped);
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_skip_last_item_transitions_to_summary() {
    let (mut model, start) = model_with_active_session(1);
    let now = start + chrono::Duration::seconds(10);

    update(&mut model, Event::Session(SessionEvent::SkipItem { now }));

    assert!(model.last_error.is_none());
    if let SessionStatus::Summary(ref summary) = model.session_status {
        assert_eq!(summary.entries[0].status, EntryStatus::Skipped);
        assert_eq!(summary.entries[0].duration_secs, 0);
    } else {
        panic!("Expected Summary state");
    }
}

// --- Summary Phase Tests ---

fn model_with_summary() -> Model {
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(30);
    let t2 = t1 + chrono::Duration::seconds(45);

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading: TempoReading::silent(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t2,
            next_item_started_at: t2,
            reading: TempoReading::silent(),
        }),
    );
    model
}

#[test]
fn test_update_entry_notes() {
    let mut model = model_with_summary();

    let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
        s.entries[0].id.clone()
    } else {
        panic!("Expected Summary state");
    };

    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryNotes {
            entry_id,
            notes: Some("Needs more practice".to_string()),
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(s.entries[0].notes, Some("Needs more practice".to_string()));
    } else {
        panic!("Expected Summary state");
    }
}

#[test]
fn test_update_entry_notes_too_long() {
    let mut model = model_with_summary();

    let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
        s.entries[0].id.clone()
    } else {
        panic!("Expected Summary state");
    };

    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryNotes {
            entry_id,
            notes: Some("x".repeat(5001)),
        }),
    );

    assert!(model.last_error.is_some());
}

#[test]
fn test_update_session_notes() {
    let mut model = model_with_summary();

    update(
        &mut model,
        Event::Session(SessionEvent::UpdateSessionNotes {
            notes: Some("Great practice today".to_string()),
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(s.session_notes, Some("Great practice today".to_string()));
    } else {
        panic!("Expected Summary state");
    }
}

#[test]
fn test_update_session_notes_rejected_outside_summary() {
    let (mut model, _start) = model_with_active_session(2);

    update(
        &mut model,
        Event::Session(SessionEvent::UpdateSessionNotes {
            notes: Some("mid-session thought".to_string()),
        }),
    );

    assert_eq!(model.last_error, Some("Not in summary state".to_string()));
    let SessionStatus::Active(_) = model.session_status else {
        panic!("Active session must be untouched");
    };
}

fn saves_session(cmd: &mut Command<Effect, Event>) -> bool {
    cmd.effects().any(|e| {
        matches!(e, Effect::Persistence(req)
        if matches!(&req.operation, crate::persistence::PersistenceOperation::SaveSession(_)))
    })
}

fn clears_recovery_copy(cmd: &mut Command<Effect, Event>) -> bool {
    cmd.effects().any(|e| {
        matches!(e, Effect::App(req)
        if matches!(req.operation, AppEffect::ClearSessionInProgress))
    })
}

fn reloads_sessions(cmd: &mut Command<Effect, Event>) -> bool {
    cmd.effects().any(|e| {
        matches!(e, Effect::Persistence(req)
        if req.operation == crate::persistence::PersistenceOperation::LoadSessions)
    })
}

#[test]
fn save_session_parks_the_practice_until_the_store_answers() {
    let mut model = model_with_summary();
    let summary_id = match model.session_status {
        SessionStatus::Summary(ref s) => s.id.clone(),
        _ => panic!("fixture is in Summary"),
    };
    let mut cmd = Intrada.update(
        Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
        &mut model,
    );

    assert!(saves_session(&mut cmd), "the write is sent");
    assert!(
        !clears_recovery_copy(&mut cmd),
        "the recovery copy outlives the write until the store confirms it (#974)"
    );
    assert!(matches!(model.session_status, SessionStatus::Summary(_)));
    assert!(
        model.sessions.is_empty(),
        "nothing is pushed before the ack"
    );
    assert_eq!(
        model.saving_session.as_ref().map(|s| s.id.as_str()),
        Some(summary_id.as_str())
    );
    assert!(model.last_error.is_none());
}

#[test]
fn acknowledged_save_pushes_the_session_clears_the_copy_and_closes_the_summary() {
    let mut model = model_with_summary();
    model.error_muted = true;
    update(
        &mut model,
        Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
    );
    let mut cmd = Intrada.update(
        Event::SessionStoreWritten(crate::persistence::PersistenceOutput::Ack),
        &mut model,
    );

    assert!(matches!(model.session_status, SessionStatus::Idle));
    assert_eq!(model.sessions.len(), 1);
    assert_eq!(model.sessions[0].total_duration_secs, 75); // 30 + 45
    assert_eq!(
        model.sessions[0].completion_status,
        CompletionStatus::Completed
    );
    assert!(model.saving_session.is_none());
    assert!(
        !model.practice_summaries.is_empty(),
        "practice data is visible without a re-fetch (#247)"
    );
    assert!(clears_recovery_copy(&mut cmd));
    assert!(
        !model.error_muted,
        "an acknowledged write lifts the mute (#1936)"
    );
    assert!(model.last_error.is_none());
}

#[test]
fn failed_save_keeps_the_summary_and_the_copy_and_shows_the_banner() {
    let mut model = model_with_summary();
    model.error_muted = true;
    update(
        &mut model,
        Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
    );
    let mut cmd = Intrada.update(
        Event::SessionStoreWritten(crate::persistence::PersistenceOutput::Failed),
        &mut model,
    );

    assert!(
        matches!(model.session_status, SessionStatus::Summary(_)),
        "the summary stays so Save can be tapped again"
    );
    assert!(model.sessions.is_empty());
    assert!(model.saving_session.is_none());
    assert_eq!(
        model.last_error.as_deref(),
        Some(SAVE_FAILED),
        "the musician's own tap is never muted"
    );
    assert!(
        !reloads_sessions(&mut cmd),
        "nothing was pushed, so there is nothing to roll back"
    );
    assert!(!clears_recovery_copy(&mut cmd));
}

#[test]
fn save_can_be_tapped_again_after_a_failure() {
    let mut model = model_with_summary();
    update(
        &mut model,
        Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
    );
    update(
        &mut model,
        Event::SessionStoreWritten(crate::persistence::PersistenceOutput::Failed),
    );
    let mut cmd = Intrada.update(
        Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
        &mut model,
    );
    assert!(saves_session(&mut cmd));
    assert!(model.last_error.is_none());
}

#[test]
fn a_second_save_while_one_is_in_flight_sends_nothing() {
    let mut model = model_with_summary();
    update(
        &mut model,
        Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
    );
    let mut cmd = Intrada.update(
        Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
        &mut model,
    );
    assert!(!saves_session(&mut cmd), "one write in flight at a time");
    assert!(model.saving_session.is_some());
    assert!(model.last_error.is_none());
}

#[test]
fn ack_after_discard_still_records_the_session() {
    let mut model = model_with_summary();
    update(
        &mut model,
        Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
    );
    update(&mut model, Event::Session(SessionEvent::DiscardSession));
    let mut cmd = Intrada.update(
        Event::SessionStoreWritten(crate::persistence::PersistenceOutput::Ack),
        &mut model,
    );
    assert_eq!(
        model.sessions.len(),
        1,
        "the row is on disk, so the model says what the disk says"
    );
    assert!(matches!(model.session_status, SessionStatus::Idle));
    assert!(
        !clears_recovery_copy(&mut cmd),
        "discard already cleared it; a later practice's copy must survive"
    );
}

#[test]
fn failed_save_after_discard_says_nothing() {
    let mut model = model_with_summary();
    update(
        &mut model,
        Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
    );
    update(&mut model, Event::Session(SessionEvent::DiscardSession));
    update(
        &mut model,
        Event::SessionStoreWritten(crate::persistence::PersistenceOutput::Failed),
    );
    assert!(model.last_error.is_none(), "the musician chose to drop it");
    assert!(model.saving_session.is_none());
    assert!(model.sessions.is_empty());
}

#[test]
fn test_save_session_updates_practice_summaries_in_view() {
    // Regression test for #247: practice data must be visible in the
    // ViewModel immediately after the save is acknowledged, without a re-fetch.
    let mut model = model_with_summary();

    // Score the first entry before saving
    if let SessionStatus::Summary(ref mut summary) = model.session_status {
        summary.entries[0]
            .open_play_mut()
            .expect("the practised entry has a play")
            .score = Some(4);
    }

    let now = Utc::now();
    update(
        &mut model,
        Event::Session(SessionEvent::SaveSession { now }),
    );
    update(
        &mut model,
        Event::SessionStoreWritten(crate::persistence::PersistenceOutput::Ack),
    );

    // Build the view: this is what the shell sees
    let app = crate::app::Intrada;
    let vm = <crate::app::Intrada as crux_core::App>::view(&app, &model);

    // Find the item that was practised (piece-1)
    let practised_item = vm.items.iter().find(|i| i.id == "piece-1");
    assert!(
        practised_item.is_some(),
        "piece-1 should be in the ViewModel"
    );

    let practice = practised_item.unwrap().practice.as_ref();
    assert!(
        practice.is_some(),
        "piece-1 should have practice data after SaveSession"
    );

    let practice = practice.unwrap();
    assert_eq!(practice.session_count, 1);
    assert_eq!(practice.latest_score, Some(4));
}

#[test]
fn test_discard_session() {
    let mut model = model_with_summary();

    update(&mut model, Event::Session(SessionEvent::DiscardSession));

    assert!(model.last_error.is_none());
    assert!(matches!(model.session_status, SessionStatus::Idle));
    assert!(model.sessions.is_empty());
}

// --- Recovery Tests ---

#[test]
fn test_recover_session() {
    let mut model = model_with_library();
    let now = Utc::now();

    let active = ActiveSession {
        id: "recovered-session".to_string(),
        entries: vec![create_entry(
            "piece-1",
            "Moonlight Sonata",
            ItemKind::Piece,
            0,
        )],
        current_index: 0,
        current_item_started_at: now,
        session_started_at: now,
    };

    update(
        &mut model,
        Event::Session(SessionEvent::RecoverSession {
            session: active,
            now,
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(a.id, "recovered-session");
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_recover_session_reanchors_current_item_timer() {
    let mut model = model_with_library();
    let started_yesterday = Utc::now() - chrono::Duration::hours(20);
    let now = Utc::now();

    let active = ActiveSession {
        id: "stale-session".to_string(),
        entries: vec![create_entry(
            "piece-1",
            "Moonlight Sonata",
            ItemKind::Piece,
            0,
        )],
        current_index: 0,
        current_item_started_at: started_yesterday,
        session_started_at: started_yesterday,
    };

    update(
        &mut model,
        Event::Session(SessionEvent::RecoverSession {
            session: active,
            now,
        }),
    );

    let SessionStatus::Active(ref a) = model.session_status else {
        panic!("Expected Active state");
    };
    assert_eq!(
        a.current_item_started_at, now,
        "resume must re-anchor the running item's timer, not show 20h elapsed"
    );
    assert_eq!(
        a.session_started_at, started_yesterday,
        "the session's historical start stays untouched"
    );
}

#[test]
fn test_recover_session_reanchors_open_play_so_close_excludes_dead_time() {
    let mut model = model_with_library();
    let started_yesterday = Utc::now() - chrono::Duration::hours(20);
    let now = Utc::now();
    let closed_seconds = 90;

    let mut entry = create_entry("piece-1", "Moonlight Sonata", ItemKind::Piece, 0);
    let mut earlier = VariationPlay::opened(None, None, started_yesterday);
    earlier.seconds = closed_seconds;
    entry.plays.push(earlier);
    entry
        .plays
        .push(VariationPlay::opened(None, None, started_yesterday));

    let active = ActiveSession {
        id: "stale-session".to_string(),
        entries: vec![entry],
        current_index: 0,
        current_item_started_at: started_yesterday,
        session_started_at: started_yesterday,
    };

    update(
        &mut model,
        Event::Session(SessionEvent::RecoverSession {
            session: active,
            now,
        }),
    );

    let closed_at = now + chrono::Duration::seconds(3);
    update(
        &mut model,
        Event::Session(SessionEvent::PrepareReflection {
            now: closed_at,
            reading: TempoReading::silent(),
        }),
    );

    let SessionStatus::Active(ref a) = model.session_status else {
        panic!("Expected Active state");
    };
    let plays = &a.entries[0].plays;
    assert_eq!(
        plays[1].seconds, 3,
        "the open play counts from the resume, not from before the kill (#1795)"
    );
    assert!(
        plays[1].is_incidental(),
        "three seconds after resume is a stray play, not one to offer a mark for"
    );
    assert_eq!(
        plays[0].seconds, closed_seconds,
        "a play closed before the kill keeps its recorded time"
    );
    assert_eq!(
        plays[0].started_at, started_yesterday,
        "a closed play keeps its historical start"
    );
}

#[test]
fn test_recover_session_reanchors_only_the_current_entry_play() {
    let mut model = model_with_library();
    let started_yesterday = Utc::now() - chrono::Duration::hours(20);
    let now = Utc::now();

    let mut done = create_entry("piece-1", "Moonlight Sonata", ItemKind::Piece, 0);
    done.status = EntryStatus::Completed;
    let mut done_play = VariationPlay::opened(None, None, started_yesterday);
    done_play.seconds = 90;
    done.plays.push(done_play);
    let mut current = create_entry("piece-2", "Clair de Lune", ItemKind::Piece, 1);
    current
        .plays
        .push(VariationPlay::opened(None, None, started_yesterday));

    let active = ActiveSession {
        id: "stale-session".to_string(),
        entries: vec![done, current],
        current_index: 1,
        current_item_started_at: started_yesterday,
        session_started_at: started_yesterday,
    };

    update(
        &mut model,
        Event::Session(SessionEvent::RecoverSession {
            session: active,
            now,
        }),
    );

    let SessionStatus::Active(ref a) = model.session_status else {
        panic!("Expected Active state");
    };
    assert_eq!(
        a.entries[1].plays[0].started_at, now,
        "the current entry's open play counts from the resume"
    );
    assert_eq!(
        a.entries[0].plays[0].started_at, started_yesterday,
        "a completed entry's play keeps its historical start"
    );
    assert_eq!(a.entries[0].plays[0].seconds, 90);
}

#[test]
fn test_recover_session_when_not_idle() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));

    let now = Utc::now();
    let active = ActiveSession {
        id: "recovered".to_string(),
        entries: vec![],
        current_index: 0,
        current_item_started_at: now,
        session_started_at: now,
    };

    update(
        &mut model,
        Event::Session(SessionEvent::RecoverSession {
            session: active,
            now,
        }),
    );

    assert!(model.last_error.is_some());
}

#[test]
fn test_recover_session_refuses_empty_setlist_and_clears_blob() {
    let mut model = model_with_library();
    let now = Utc::now();
    let active = ActiveSession {
        id: "empty-snapshot".to_string(),
        entries: vec![],
        current_index: 0,
        current_item_started_at: now,
        session_started_at: now,
    };

    let app = Intrada;
    let mut cmd = app.update(
        Event::Session(SessionEvent::RecoverSession {
            session: active,
            now,
        }),
        &mut model,
    );

    assert!(
        matches!(model.session_status, SessionStatus::Idle),
        "an empty setlist never becomes Active: current_entry would index it (#1807)"
    );
    assert!(model.last_error.is_some());
    assert!(
        cmd.effects().any(|e| matches!(e, Effect::App(req)
            if matches!(req.operation, AppEffect::ClearSessionInProgress))),
        "the unusable snapshot is cleared so Resume does not reappear at next launch"
    );
}

// --- Edge Case Tests ---

#[test]
fn test_all_items_skipped() {
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(5);
    let t2 = t1 + chrono::Duration::seconds(5);

    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem { now: t1 }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem { now: t2 }),
    );

    assert!(matches!(model.session_status, SessionStatus::Summary(_)));

    let save_time = Utc::now();
    update(
        &mut model,
        Event::Session(SessionEvent::SaveSession { now: save_time }),
    );
    update(
        &mut model,
        Event::SessionStoreWritten(crate::persistence::PersistenceOutput::Ack),
    );

    assert_eq!(model.sessions.len(), 1);
    assert_eq!(model.sessions[0].total_duration_secs, 0);
}

#[test]
fn test_single_item_setlist() {
    let (mut model, start) = model_with_active_session(1);
    let now = start + chrono::Duration::seconds(120);

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now,
            next_item_started_at: now,
            reading: TempoReading::silent(),
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(s.entries.len(), 1);
        assert_eq!(s.entries[0].duration_secs, 120);
    } else {
        panic!("Expected Summary state");
    }
}

#[test]
fn test_complete_lifecycle() {
    let mut model = model_with_library();
    let t0 = Utc::now();

    // 1. Start building
    update(&mut model, Event::Session(SessionEvent::StartBuilding));

    // 2. Add 3 items
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-2".to_string(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "exercise-1".to_string(),
        }),
    );

    // 3. Start session
    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now: t0 }),
    );

    // 4. Practice first item for 30s, then Next
    let t1 = t0 + chrono::Duration::seconds(30);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading: TempoReading::silent(),
        }),
    );

    // 5. Skip second item
    let t2 = t1 + chrono::Duration::seconds(5);
    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem { now: t2 }),
    );

    // 6. Finish on third item after 60s
    let t3 = t2 + chrono::Duration::seconds(60);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t3,
            next_item_started_at: t3,
            reading: TempoReading::silent(),
        }),
    );

    // 7. Add notes
    let entry_id_0 = if let SessionStatus::Summary(ref s) = model.session_status {
        s.entries[0].id.clone()
    } else {
        panic!("Expected Summary");
    };
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryNotes {
            entry_id: entry_id_0,
            notes: Some("Good tempo control".to_string()),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateSessionNotes {
            notes: Some("Focused session".to_string()),
        }),
    );

    // 8. Save
    let t_save = Utc::now();
    update(
        &mut model,
        Event::Session(SessionEvent::SaveSession { now: t_save }),
    );
    update(
        &mut model,
        Event::SessionStoreWritten(crate::persistence::PersistenceOutput::Ack),
    );

    // Verify final state
    assert!(matches!(model.session_status, SessionStatus::Idle));
    assert_eq!(model.sessions.len(), 1);

    let session = &model.sessions[0];
    assert_eq!(session.entries.len(), 3);
    assert_eq!(session.entries[0].duration_secs, 30);
    assert_eq!(session.entries[0].status, EntryStatus::Completed);
    assert_eq!(
        session.entries[0].notes,
        Some("Good tempo control".to_string())
    );
    assert_eq!(session.entries[1].duration_secs, 0);
    assert_eq!(session.entries[1].status, EntryStatus::Skipped);
    assert_eq!(session.entries[2].duration_secs, 60);
    assert_eq!(session.entries[2].status, EntryStatus::Completed);
    assert_eq!(session.session_notes, Some("Focused session".to_string()));
    assert_eq!(session.total_duration_secs, 90); // 30 + 0 + 60
    assert_eq!(session.completion_status, CompletionStatus::Completed);
}

// --- UpdateEntryScore Tests ---

#[test]
fn test_update_entry_score_on_completed_entry() {
    let mut model = model_with_summary();

    let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
        s.entries[0].id.clone()
    } else {
        panic!("Expected Summary state");
    };

    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: entry_id.clone(),
            play_id,
            score: Some(4),
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(play_of(&s.entries[0]).score, Some(4));
    } else {
        panic!("Expected Summary state");
    }
}

#[test]
fn test_update_entry_score_toggle_clears_score() {
    let mut model = model_with_summary();

    let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
        s.entries[0].id.clone()
    } else {
        panic!("Expected Summary state");
    };

    // Set score to 4
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: entry_id.clone(),
            play_id,
            score: Some(4),
        }),
    );

    // Clear score by setting to None (toggle)
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: entry_id.clone(),
            play_id,
            score: None,
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(play_of(&s.entries[0]).score, None);
    } else {
        panic!("Expected Summary state");
    }
}

#[test]
fn test_update_entry_score_ignored_on_skipped_entry() {
    // Create a session where one item is skipped
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(5);
    let t2 = t1 + chrono::Duration::seconds(30);

    // Skip the first item
    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem { now: t1 }),
    );
    // Finish the second item
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t2,
            next_item_started_at: t2,
            reading: TempoReading::silent(),
        }),
    );

    let (skipped_entry_id, practised_entry_id) =
        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].status, EntryStatus::Skipped);
            (s.entries[0].id.clone(), s.entries[1].id.clone())
        } else {
            panic!("Expected Summary state");
        };

    // A skipped entry keeps no play of its own (#1739 decision 3), so the
    // only real id the sheet could send is one from the practised item.
    let play_id = first_play_id(&model, &practised_entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: skipped_entry_id.clone(),
            play_id,
            score: Some(3),
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(s.entries[0].score_summary(), None, "score not set");
        assert!(s.entries[0].plays.is_empty());
    } else {
        panic!("Expected Summary state");
    }
}

#[test]
fn test_update_entry_score_out_of_range_rejected() {
    let mut model = model_with_summary();

    let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
        s.entries[0].id.clone()
    } else {
        panic!("Expected Summary state");
    };

    // Score 0: out of range
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: entry_id.clone(),
            play_id,
            score: Some(0),
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(play_of(&s.entries[0]).score, None); // Score not set
    }

    // Score 11: out of range
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: entry_id.clone(),
            play_id,
            score: Some(11),
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(play_of(&s.entries[0]).score, None); // Score still not set
    }
}

#[test]
fn test_update_entry_score_rejected_on_pending_entry() {
    // The current item (the one in progress) has status NotAttempted
    // until NextItem / SkipItem flips it. Scoring it would let the user
    // rate work they haven't done. Invariant: only Completed entries
    // can be scored, regardless of session phase.
    let (mut model, _start) = model_with_active_session(2);

    let entry_id = if let SessionStatus::Active(ref a) = model.session_status {
        a.entries[0].id.clone()
    } else {
        panic!("Expected Active state");
    };

    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: entry_id.clone(),
            play_id,
            score: Some(3),
        }),
    );

    // Score not set: entry is still NotAttempted
    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(play_of(&a.entries[0]).score, None);
        assert_eq!(a.entries[0].status, EntryStatus::NotAttempted);
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_update_entry_score_works_mid_session_on_completed_entry() {
    // The mid-session reflection sheet's flow: NextItem flips the
    // just-completed entry to Completed, THEN the sheet dispatches
    // UpdateEntryScore for that entry. Session is still Active (we're
    // moving on to item 2, not finishing). The score should land.
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(45);

    // Advance: entry[0] becomes Completed, current_index moves to 1
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading: TempoReading::silent(),
        }),
    );

    // Capture the just-completed entry id (still in Active phase)
    let entry_id = if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(a.entries[0].status, EntryStatus::Completed);
        assert_eq!(a.current_index, 1);
        a.entries[0].id.clone()
    } else {
        panic!("Expected Active state: only one of two items advanced");
    };

    // Score the just-completed entry mid-session
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: entry_id.clone(),
            play_id,
            score: Some(4),
        }),
    );

    // Score persisted, session still Active
    assert!(model.last_error.is_none());
    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(play_of(&a.entries[0]).score, Some(4));
    } else {
        panic!("Expected Active state: session shouldn't have ended");
    }
}

#[test]
fn test_mid_session_entry_score_survives_into_summary() {
    // A per-entry score set mid-session must still be present once the
    // session finishes and projects into the Summary: the reconciliation
    // the reflection hand-off (Phase 6) relies on.
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(45);
    let t2 = t1 + chrono::Duration::seconds(30);

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading: TempoReading::silent(),
        }),
    );
    let entry_id = if let SessionStatus::Active(ref a) = model.session_status {
        a.entries[0].id.clone()
    } else {
        panic!("Expected Active state after advancing one of two items");
    };

    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id,
            play_id,
            score: Some(4),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t2,
            next_item_started_at: t2,
            reading: TempoReading::silent(),
        }),
    );

    if let SessionStatus::Summary(ref summary) = model.session_status {
        assert_eq!(play_of(&summary.entries[0]).score, Some(4));
    } else {
        panic!("Expected Summary state after finishing the session");
    }
}

#[test]
fn test_update_entry_tempo_works_mid_session_on_completed_entry() {
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(45);

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading: TempoReading::silent(),
        }),
    );

    let entry_id = if let SessionStatus::Active(ref a) = model.session_status {
        a.entries[0].id.clone()
    } else {
        panic!("Expected Active state");
    };

    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id,
            play_id,
            tempo: Some(120),
            user_set: true,
            click: None,
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(play_of(&a.entries[0]).achieved_tempo, Some(120));
    }
}

#[test]
fn test_update_entry_notes_works_mid_session_on_completed_entry() {
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(45);

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading: TempoReading::silent(),
        }),
    );

    let entry_id = if let SessionStatus::Active(ref a) = model.session_status {
        a.entries[0].id.clone()
    } else {
        panic!("Expected Active state");
    };

    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryNotes {
            entry_id,
            notes: Some("felt solid".to_string()),
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(a.entries[0].notes.as_deref(), Some("felt solid"));
    }
}

#[test]
fn test_update_entry_score_works_on_last_item_after_finishing() {
    // The last-item path through the reflection sheet: NextItem to the
    // final item, NextItem again → transitions to Summary, then the sheet
    // dispatches UpdateEntryScore. `transition_to_summary` clones entries
    // into `summary.entries`, so the entry id must still resolve via
    // `entry_for_update_mut`. Pinning this so a future refactor of the
    // transition can't silently drop scoring on the last item.
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(45);
    let t2 = t1 + chrono::Duration::seconds(60);

    // Advance to the last item, then finish the session
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading: TempoReading::silent(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t2,
            next_item_started_at: t2,
            reading: TempoReading::silent(),
        }),
    );

    // Should be in Summary phase now
    let last_entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(s.entries[1].status, EntryStatus::Completed);
        s.entries[1].id.clone()
    } else {
        panic!("Expected Summary state after finishing on the last item");
    };

    // Score the last item: same code path the reflection sheet takes
    let play_id = first_play_id(&model, &last_entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: last_entry_id,
            play_id,
            score: Some(5),
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(play_of(&s.entries[1]).score, Some(5));
    } else {
        panic!("Expected Summary state");
    }
}

#[test]
fn test_update_entry_score_unknown_entry_id_is_silent_noop() {
    // The sheet snapshots the entry id at open time. If the session is
    // cleared (recovery, new session) before Continue, the id won't
    // match anything in the new model. `entry_for_update_mut` returns
    // None: the dispatch must be silent (no last_error, no panic).
    let mut model = model_with_summary();
    let live_entry_id = match model.session_status {
        SessionStatus::Summary(ref s) => s.entries[0].id.clone(),
        _ => panic!("Expected Summary state"),
    };
    let play_id = first_play_id(&model, &live_entry_id);

    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: "no-such-entry".to_string(),
            play_id: play_id.clone(),
            score: Some(3),
        }),
    );

    // Same shape for tempo
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: "no-such-entry".to_string(),
            play_id,
            tempo: Some(120),
            user_set: true,
            click: None,
        }),
    );

    // Score and tempo handlers don't surface an error for unknown ids
    // (they're silent no-ops, matching the existing pattern).
    assert!(model.last_error.is_none());
}

#[test]
fn test_update_entry_score_rejected_on_skipped_entry_in_active_phase() {
    // SkipItem produces EntryStatus::Skipped, not Completed. The
    // status gate in the handlers should reject scoring a skipped
    // entry mid-session, just as it does in the summary phase.
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(5);

    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem { now: t1 }),
    );

    let (skipped_entry_id, running_entry_id) =
        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].status, EntryStatus::Skipped);
            assert_eq!(a.current_index, 1, "Should have advanced past skipped item");
            (a.entries[0].id.clone(), a.entries[1].id.clone())
        } else {
            panic!("Expected Active state: second item should still be running");
        };

    // A skipped entry keeps no play of its own (#1739 decision 3).
    let play_id = first_play_id(&model, &running_entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: skipped_entry_id,
            play_id,
            score: Some(3),
        }),
    );

    // Score not applied: entry is Skipped, not Completed
    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(a.entries[0].score_summary(), None);
        assert!(a.entries[0].plays.is_empty());
        assert_eq!(a.entries[0].status, EntryStatus::Skipped);
    }
}

#[test]
fn test_update_entry_score_boundary_values() {
    let mut model = model_with_summary();

    let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
        s.entries[0].id.clone()
    } else {
        panic!("Expected Summary state");
    };

    // Score 1: minimum valid
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: entry_id.clone(),
            play_id,
            score: Some(1),
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(play_of(&s.entries[0]).score, Some(1));
    }

    // Score 10: maximum valid
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: entry_id.clone(),
            play_id,
            score: Some(10),
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(play_of(&s.entries[0]).score, Some(10));
    }
}

// --- SetEntryVariant Tests (#1083 C1) ---

fn give_exercise_a_ladder(model: &mut Model) -> (String, String) {
    let now = Utc::now();
    let ex = model
        .items
        .iter_mut()
        .find(|i| i.id == "exercise-1")
        .expect("fixture exercise");
    ex.variants = vec![
        crate::domain::variant::Variant {
            id: "v-c".to_string(),
            label: "C".to_string(),
            position: 0,
            updated_at: now,
            deleted_at: None,
        },
        crate::domain::variant::Variant {
            id: "v-f".to_string(),
            label: "F".to_string(),
            position: 1,
            updated_at: now,
            deleted_at: None,
        },
    ];
    ("v-c".to_string(), "v-f".to_string())
}

/// Building state whose single entry is the laddered exercise. Local-first,
/// as on iOS; variations are gated to that mode (#1083).
fn model_with_exercise_building() -> (Model, String) {
    let mut model = model_with_library();
    give_exercise_a_ladder(&mut model);
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "exercise-1".to_string(),
        }),
    );
    let entry_id = building_entries(&model)[0].id.clone();
    (model, entry_id)
}

/// The same setlist carried through to Summary.
fn model_with_exercise_summary() -> (Model, String) {
    let (mut model, _) = model_with_exercise_building();
    let now = Utc::now();
    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: now + chrono::Duration::seconds(60),
            next_item_started_at: now + chrono::Duration::seconds(60),
            reading: TempoReading::silent(),
        }),
    );
    let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
        s.entries[0].id.clone()
    } else {
        panic!("Expected Summary state");
    };
    (model, entry_id)
}

fn planned_variation(model: &Model) -> Option<String> {
    session_entries(model)[0].planned_variation_id.clone()
}

/// Planning a variation is a Building-phase move (#1739 decision 5): once
/// the entry has been practised the record is its plays, and
/// `SwitchVariation` is what changes them.
#[test]
fn set_entry_variant_is_rejected_on_a_completed_entry() {
    let (mut model, entry_id) = model_with_exercise_summary();

    update(
        &mut model,
        Event::Session(SessionEvent::SetEntryVariant {
            entry_id,
            variant_id: Some("v-c".to_string()),
        }),
    );

    assert!(model.last_error.is_some(), "surfaced, not silent");
    assert_eq!(planned_variation(&model), None);
}

#[test]
fn set_entry_variant_rejects_a_variant_of_another_item() {
    let (mut model, entry_id) = model_with_exercise_building();
    // A ladder on a different exercise.
    let now = Utc::now();
    model.items.push(Item {
        id: "exercise-2".to_string(),
        title: "Arpeggios".to_string(),
        kind: ItemKind::Exercise,
        composer: None,
        key: None,
        modality: None,
        tempo: None,
        notes: None,
        tags: vec![],
        created_at: now,
        updated_at: now,
        linked_exercise_ids: vec![],
        priority: false,
        chord_chart: None,
        variants: vec![crate::domain::variant::Variant {
            id: "v-other".to_string(),
            label: "C".to_string(),
            position: 0,
            updated_at: now,
            deleted_at: None,
        }],
        photo_id: None,
        metre: None,
    });

    update(
        &mut model,
        Event::Session(SessionEvent::SetEntryVariant {
            entry_id,
            variant_id: Some("v-other".to_string()),
        }),
    );

    assert!(model.last_error.is_some());
    assert_eq!(planned_variation(&model), None);
}

#[test]
fn set_entry_variant_rejects_a_tombstoned_variant() {
    let (mut model, entry_id) = model_with_exercise_building();
    model
        .items
        .iter_mut()
        .find(|i| i.id == "exercise-1")
        .unwrap()
        .variants
        .iter_mut()
        .find(|v| v.id == "v-f")
        .unwrap()
        .deleted_at = Some(Utc::now());

    update(
        &mut model,
        Event::Session(SessionEvent::SetEntryVariant {
            entry_id,
            variant_id: Some("v-f".to_string()),
        }),
    );

    assert!(model.last_error.is_some());
    assert_eq!(planned_variation(&model), None);
}

#[test]
fn a_skipped_entry_keeps_the_variation_it_was_planned_for() {
    // The plan outlives the skip: the entry still says which variation it
    // was meant to practise. It records no play, so it contributes nothing
    // to per-variation history and the stats stay clean.
    let mut model = model_with_library();
    give_exercise_a_ladder(&mut model);
    let now = Utc::now();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    for id in ["exercise-1", "piece-1"] {
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: id.to_string(),
            }),
        );
    }

    let entry_id = building_entries(&model)[0].id.clone();
    update(
        &mut model,
        Event::Session(SessionEvent::SetEntryVariant {
            entry_id,
            variant_id: Some("v-c".to_string()),
        }),
    );
    assert!(model.last_error.is_none());

    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem {
            now: now + chrono::Duration::seconds(5),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: now + chrono::Duration::seconds(60),
            next_item_started_at: now + chrono::Duration::seconds(60),
            reading: TempoReading::silent(),
        }),
    );

    let SessionStatus::Summary(ref s) = model.session_status else {
        panic!("Expected Summary state");
    };
    assert_eq!(
        s.entries[0].status,
        EntryStatus::Skipped,
        "fixture: exercise was skipped"
    );
    assert_eq!(
        s.entries[0].planned_variation_id.as_deref(),
        Some("v-c"),
        "the plan survives a skip"
    );
    assert!(
        s.entries[0].plays.is_empty(),
        "a skipped entry records no play, so it scores no variation"
    );
}

#[test]
fn set_entry_variant_flows_into_the_saved_session() {
    let (mut model, entry_id) = model_with_exercise_building();
    let now = Utc::now();
    update(
        &mut model,
        Event::Session(SessionEvent::SetEntryVariant {
            entry_id,
            variant_id: Some("v-c".to_string()),
        }),
    );

    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: now + chrono::Duration::seconds(60),
            next_item_started_at: now + chrono::Duration::seconds(60),
            reading: TempoReading::silent(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::SaveSession {
            now: now + chrono::Duration::seconds(65),
        }),
    );
    update(
        &mut model,
        Event::SessionStoreWritten(crate::persistence::PersistenceOutput::Ack),
    );

    assert_eq!(model.sessions.len(), 1);
    let saved = &model.sessions[0].entries[0];
    assert_eq!(
        saved.planned_variation_id.as_deref(),
        Some("v-c"),
        "the chosen variation rides the persisted session"
    );
    assert_eq!(
        play_of(saved).variation_id.as_deref(),
        Some("v-c"),
        "the first play is seeded from the plan, so the record names it too"
    );
}

#[test]
fn set_entry_variant_unknown_entry_surfaces_an_error() {
    let (mut model, _) = model_with_exercise_building();

    update(
        &mut model,
        Event::Session(SessionEvent::SetEntryVariant {
            entry_id: "no-such-entry".to_string(),
            variant_id: Some("v-c".to_string()),
        }),
    );

    assert!(model.last_error.is_some());
}

// --- UpdateEntryTempo Tests ---

// --- Tempo evidence (#1420, roadmap Q3) ---

fn tempo_entry_id(model: &Model) -> String {
    match model.session_status {
        SessionStatus::Summary(ref s) => s.entries[0].id.clone(),
        _ => panic!("Expected Summary state"),
    }
}

fn achieved_tempo(model: &Model, entry_id: &str) -> Option<u16> {
    match model.session_status {
        SessionStatus::Summary(ref s) => {
            let entry = s.entries.iter().find(|e| e.id == entry_id).unwrap();
            // Without this the negative tests would also pass against a
            // skipped entry, which the handler rejects for another reason.
            assert_eq!(entry.status, EntryStatus::Completed);
            play_of(entry).achieved_tempo
        }
        _ => panic!("Expected Summary state"),
    }
}

fn seven_eight_on_group_starts() -> ClickState {
    ClickState {
        metre: Metre {
            beats: 7,
            unit: 8,
            groups: Some(vec![3, 2, 2]),
        },
        sounding: 0b0101001,
    }
}

fn click_pattern(model: &Model, entry_id: &str) -> Option<ClickState> {
    match model.session_status {
        SessionStatus::Summary(ref s) => {
            let entry = s.entries.iter().find(|e| e.id == entry_id).unwrap();
            play_of(entry).click_pattern.clone()
        }
        _ => panic!("Expected Summary state"),
    }
}

// --- A tempo is stamped as its play closes (#1761) ---

fn sounding(bpm: u16, click: Option<ClickState>) -> TempoReading {
    TempoReading {
        bpm,
        click_sounding: true,
        click,
    }
}

fn every_beat_in_common_time() -> ClickState {
    ClickState {
        metre: Metre::default(),
        sounding: 0b1111,
    }
}

/// Two pieces, the first closed with `reading` and the second finished
/// silently: the summary the item-complete sheet writes to.
fn summary_closed_with(reading: TempoReading) -> Model {
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(30);
    let t2 = t1 + chrono::Duration::seconds(45);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading,
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t2,
            next_item_started_at: t2,
            reading: TempoReading::silent(),
        }),
    );
    model
}

fn switch_to_d(model: &mut Model, at: DateTime<Utc>, reading: TempoReading) {
    let entry_id = only_entry(model).id.clone();
    update(
        model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id,
            variation_id: Some("v-d".to_string()),
            now: at,
            reading,
        }),
    );
}

fn hand_off(model: &mut Model, at: DateTime<Utc>, reading: TempoReading) {
    update(
        model,
        Event::Session(SessionEvent::PrepareReflection {
            now: at,
            reading: reading.clone(),
        }),
    );
    update(
        model,
        Event::Session(SessionEvent::NextItem {
            now: at,
            next_item_started_at: at,
            reading,
        }),
    );
}

fn set_tempo(model: &mut Model, tempo: Option<u16>, user_set: bool, click: Option<ClickState>) {
    let entry_id = tempo_entry_id(model);
    let play_id = first_play_id(model, &entry_id);
    update(
        model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id,
            play_id,
            tempo,
            user_set,
            click,
        }),
    );
}

fn tempos(entry: &SetlistEntry) -> Vec<(Option<&str>, u64, Option<u16>)> {
    entry
        .plays
        .iter()
        .map(|p| (p.variation_id.as_deref(), p.seconds, p.achieved_tempo))
        .collect()
}

#[test]
fn a_switch_with_the_click_sounding_stamps_the_play_it_closes_not_the_one_it_opens() {
    let (mut model, start) = model_with_variations();

    switch_to_d(
        &mut model,
        start + chrono::Duration::seconds(180),
        sounding(108, Some(every_beat_in_common_time())),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.plays[0].achieved_tempo, Some(108));
    assert_eq!(
        entry.plays[0].click_pattern,
        Some(every_beat_in_common_time())
    );
    assert_eq!(
        entry.plays[1].achieved_tempo, None,
        "the play the switch opened has not closed yet"
    );
    assert_eq!(entry.plays[1].click_pattern, None);
}

#[test]
fn re_tapping_the_open_variation_with_the_click_sounding_stamps_nothing() {
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();

    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id,
            variation_id: Some("v-c".to_string()),
            now: start + chrono::Duration::seconds(180),
            reading: sounding(108, Some(every_beat_in_common_time())),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(play_of(entry).achieved_tempo, None);
    assert_eq!(play_of(entry).seconds, 0, "nothing closed");
}

/// The issue's case: six minutes of C at 108, then thirty seconds of D.
/// D reads 108 only because the click was still sounding at the hand-off.
#[test]
fn each_play_is_credited_with_the_click_that_sounded_as_it_closed() {
    let (mut model, start) = model_with_variations();
    let click = Some(every_beat_in_common_time());

    switch_to_d(
        &mut model,
        start + chrono::Duration::seconds(360),
        sounding(108, click.clone()),
    );
    hand_off(
        &mut model,
        start + chrono::Duration::seconds(390),
        sounding(108, click),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.status, EntryStatus::Completed);
    assert_eq!(
        tempos(entry),
        vec![(Some("v-c"), 360, Some(108)), (Some("v-d"), 30, Some(108))]
    );
}

#[test]
fn a_click_stopped_before_the_hand_off_leaves_the_last_play_unstamped() {
    let (mut model, start) = model_with_variations();

    switch_to_d(
        &mut model,
        start + chrono::Duration::seconds(360),
        sounding(108, Some(every_beat_in_common_time())),
    );
    hand_off(
        &mut model,
        start + chrono::Duration::seconds(390),
        TempoReading::silent(),
    );

    assert_eq!(
        tempos(only_entry(&model)),
        vec![(Some("v-c"), 360, Some(108)), (Some("v-d"), 30, None)]
    );
}

#[test]
fn a_silent_close_after_a_stamp_leaves_the_stamp() {
    let (mut model, start) = model_with_active_session(1);
    let ended = start + chrono::Duration::seconds(60);

    update(
        &mut model,
        Event::Session(SessionEvent::PrepareReflection {
            now: ended,
            reading: sounding(96, Some(every_beat_in_common_time())),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: ended,
            next_item_started_at: ended,
            reading: TempoReading::silent(),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.status, EntryStatus::Completed);
    assert_eq!(play_of(entry).achieved_tempo, Some(96));
    assert_eq!(
        play_of(entry).click_pattern,
        Some(every_beat_in_common_time())
    );
}

#[test]
fn a_later_sounding_close_overwrites_an_earlier_stamp() {
    let (mut model, start) = model_with_active_session(1);
    let ended = start + chrono::Duration::seconds(60);

    update(
        &mut model,
        Event::Session(SessionEvent::PrepareReflection {
            now: ended,
            reading: sounding(96, Some(every_beat_in_common_time())),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: ended,
            next_item_started_at: ended,
            reading: sounding(100, None),
        }),
    );

    let play = play_of(only_entry(&model));
    assert_eq!(play.achieved_tempo, Some(100));
    assert_eq!(play.click_pattern, None, "the later instant wins whole");
}

#[test]
fn prepare_reflection_then_next_item_with_the_same_reading_stamps_once() {
    let (mut model, start) = model_with_active_session(1);
    let ended = start + chrono::Duration::seconds(60);
    let reading = sounding(168, Some(seven_eight_on_group_starts()));

    update(
        &mut model,
        Event::Session(SessionEvent::PrepareReflection {
            now: ended,
            reading: reading.clone(),
        }),
    );
    let prefilled = play_of(only_entry(&model)).clone();
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: ended,
            next_item_started_at: ended,
            reading,
        }),
    );

    assert_eq!(
        prefilled.achieved_tempo,
        Some(84),
        "the sheet's rows read the stamp before the terminal event"
    );
    let entry = only_entry(&model);
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(play_of(entry).achieved_tempo, prefilled.achieved_tempo);
    assert_eq!(play_of(entry).click_pattern, prefilled.click_pattern);
}

/// The row read `♪ = 168` in 7/8; the trend must see 84 crotchets, the
/// pattern that earned it rides alongside (#1499), and the sheet reads it
/// back in quavers (#1761).
#[test]
fn a_quaver_reading_is_stamped_as_crotchets_and_displayed_in_quavers() {
    let model = summary_closed_with(sounding(168, Some(seven_eight_on_group_starts())));
    let entry_id = tempo_entry_id(&model);

    assert_eq!(achieved_tempo(&model, &entry_id), Some(84));
    assert_eq!(
        click_pattern(&model, &entry_id),
        Some(seven_eight_on_group_starts())
    );
    let summary = Intrada.view(&model).summary.expect("in Summary");
    assert_eq!(summary.entries[0].plays[0].tempo_display, Some(168));
}

/// A sparse pattern never divides the bpm: the click on beat 4 at 120 is
/// still 120 (T19).
#[test]
fn a_sparse_pattern_leaves_a_crotchet_tempo_alone() {
    let model = summary_closed_with(sounding(
        120,
        Some(ClickState {
            metre: Metre::default(),
            sounding: 0b1000,
        }),
    ));

    assert_eq!(achieved_tempo(&model, &tempo_entry_id(&model)), Some(120));
}

#[test]
fn a_sounding_reading_with_no_click_state_is_stamped_in_crotchets() {
    let model = summary_closed_with(sounding(120, None));
    let entry_id = tempo_entry_id(&model);

    assert_eq!(achieved_tempo(&model, &entry_id), Some(120));
    assert_eq!(click_pattern(&model, &entry_id), None);
}

#[test]
fn ending_early_stamps_the_play_it_closes() {
    let (mut model, start) = model_with_active_session(2);

    update(
        &mut model,
        Event::Session(SessionEvent::EndSessionEarly {
            now: start + chrono::Duration::seconds(60),
            reading: sounding(92, None),
        }),
    );

    let entry = &session_entries(&model)[0];
    assert_eq!(entry.status, EntryStatus::Completed);
    assert_eq!(play_of(entry).achieved_tempo, Some(92));
}

/// A close is not a user action (#944): a reading that fails validation
/// stamps nothing and raises nothing, and the play still closes.
#[test]
fn an_invalid_reading_stamps_nothing_and_the_play_still_closes() {
    let minims = ClickState {
        metre: Metre {
            beats: 2,
            unit: 2,
            groups: None,
        },
        sounding: 0b11,
    };
    let no_beat = ClickState {
        metre: Metre::default(),
        sounding: 0,
    };
    let readings = [
        ("a click sounding no beat", sounding(108, Some(no_beat))),
        ("no tempo at all", sounding(0, None)),
        (
            "a tempo past the ceiling",
            sounding(validation::MAX_ACHIEVED_TEMPO + 1, None),
        ),
        (
            "a minim tempo past the ceiling once in crotchets",
            sounding(260, Some(minims)),
        ),
    ];
    for (case, reading) in readings {
        let (mut model, start) = model_with_variations();

        switch_to_d(&mut model, start + chrono::Duration::seconds(180), reading);

        let entry = only_entry(&model);
        assert_eq!(entry.plays.len(), 2, "{case}: the switch still happened");
        assert_eq!(entry.plays[0].seconds, 180, "{case}");
        assert_eq!(entry.plays[0].achieved_tempo, None, "{case}");
        assert_eq!(entry.plays[0].click_pattern, None, "{case}");
        assert!(
            model.last_error.is_none(),
            "{case}: a close raises no error"
        );
        assert_eq!(
            model.last_notice.as_deref(),
            Some(UNUSABLE_TEMPO_NOTICE),
            "{case}: the musician hears why the play has no tempo (#1325)"
        );
    }
}

fn minims_past_the_ceiling() -> TempoReading {
    sounding(
        260,
        Some(ClickState {
            metre: Metre {
                beats: 2,
                unit: 2,
                groups: None,
            },
            sounding: 0b11,
        }),
    )
}

#[test]
fn every_close_path_reports_a_reading_it_could_not_keep() {
    type Close = fn(&mut Model, DateTime<Utc>, TempoReading);
    let closes: [(&str, Close); 4] = [
        ("a switch", switch_to_d),
        ("the reflection sheet opening", |model, at, reading| {
            update(
                model,
                Event::Session(SessionEvent::PrepareReflection { now: at, reading }),
            );
        }),
        ("the last item", |model, at, reading| {
            update(
                model,
                Event::Session(SessionEvent::NextItem {
                    now: at,
                    next_item_started_at: at,
                    reading,
                }),
            );
        }),
        ("ending early", |model, at, reading| {
            update(
                model,
                Event::Session(SessionEvent::EndSessionEarly { now: at, reading }),
            );
        }),
    ];
    for (case, close) in closes {
        let (mut model, start) = model_with_variations();
        let error_seq = model.error_seq;
        let notice_seq = model.notice_seq;

        close(
            &mut model,
            start + chrono::Duration::seconds(180),
            minims_past_the_ceiling(),
        );

        assert_eq!(
            model.last_notice.as_deref(),
            Some(UNUSABLE_TEMPO_NOTICE),
            "{case}"
        );
        assert!(model.notice_seq > notice_seq, "{case}: a notice is a raise");
        assert_eq!(
            model.error_seq, error_seq,
            "{case}: a notice is not a refusal, so the haptic still fires"
        );
        assert!(model.last_error.is_none(), "{case}");
    }
}

#[test]
fn the_next_item_mid_session_reports_a_reading_it_could_not_keep() {
    let (mut model, start) = model_with_active_session(2);
    let error_seq = model.error_seq;

    let at = start + chrono::Duration::seconds(180);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: at,
            next_item_started_at: at,
            reading: minims_past_the_ceiling(),
        }),
    );

    assert!(
        matches!(model.session_status, SessionStatus::Active(_)),
        "the session went on to the second item"
    );
    assert_eq!(model.last_notice.as_deref(), Some(UNUSABLE_TEMPO_NOTICE));
    assert_eq!(model.notice_seq, 1);
    assert_eq!(model.error_seq, error_seq);
}

#[test]
fn a_reading_the_core_keeps_or_ignores_raises_no_notice() {
    let readings = [
        ("silent", TempoReading::silent()),
        (
            "crotchets",
            sounding(108, Some(every_beat_in_common_time())),
        ),
        ("no click state", sounding(108, None)),
    ];
    for (case, reading) in readings {
        let (mut model, start) = model_with_variations();

        switch_to_d(&mut model, start + chrono::Duration::seconds(180), reading);

        assert_eq!(model.last_notice, None, "{case}");
        assert_eq!(model.notice_seq, 0, "{case}");
    }
}

#[test]
fn a_skip_carries_no_reading_and_raises_no_notice() {
    let (mut model, start) = model_with_variations();

    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem {
            now: start + chrono::Duration::seconds(180),
        }),
    );

    assert_eq!(model.last_notice, None);
    assert_eq!(model.notice_seq, 0);
}

#[test]
fn a_stamped_stray_tap_is_still_dropped() {
    let (mut model, start) = model_with_variations();
    let opened = only_entry(&model).plays[0].id.clone();
    switch_to_d(
        &mut model,
        start + chrono::Duration::seconds(298),
        TempoReading::silent(),
    );

    let ended = start + chrono::Duration::seconds(300);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: ended,
            next_item_started_at: ended,
            reading: sounding(108, None),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.plays.len(), 1, "a stamp is not a record");
    assert_eq!(entry.plays[0].id, opened);
}

/// Only a completed entry carries a tempo, so a skip clears the stamp a
/// switch made on a play it keeps, and keeps that play's repetitions.
#[test]
fn a_skip_clears_the_stamp_on_a_play_it_keeps_and_keeps_its_repetitions() {
    let (mut model, start) = model_with_variations();
    update(
        &mut model,
        Event::Session(SessionEvent::RepGotIt {
            now: start + chrono::Duration::seconds(10),
        }),
    );
    switch_to_d(
        &mut model,
        start + chrono::Duration::seconds(60),
        sounding(108, Some(every_beat_in_common_time())),
    );
    assert_eq!(
        only_entry(&model).plays[0].achieved_tempo,
        Some(108),
        "fixture: the switch stamped C"
    );

    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem {
            now: start + chrono::Duration::seconds(90),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.status, EntryStatus::Skipped);
    assert_eq!(
        entry.plays.len(),
        1,
        "C banked a repetition, D recorded nothing"
    );
    assert_eq!(entry.plays[0].rep_count, Some(1));
    assert_eq!(entry.plays[0].achieved_tempo, None);
    assert_eq!(entry.plays[0].click_pattern, None);
}

#[test]
fn a_tempo_nobody_set_leaves_a_stamp_alone() {
    let mut model = summary_closed_with(sounding(108, Some(every_beat_in_common_time())));

    set_tempo(&mut model, Some(96), false, None);

    let entry_id = tempo_entry_id(&model);
    assert_eq!(achieved_tempo(&model, &entry_id), Some(108));
    assert_eq!(
        click_pattern(&model, &entry_id),
        Some(every_beat_in_common_time())
    );
}

#[test]
fn a_tempo_the_musician_set_overwrites_a_stamp_and_keeps_its_pattern() {
    let mut model = summary_closed_with(sounding(168, Some(seven_eight_on_group_starts())));

    set_tempo(
        &mut model,
        Some(176),
        true,
        Some(every_beat_in_common_time()),
    );

    let entry_id = tempo_entry_id(&model);
    assert_eq!(
        achieved_tempo(&model, &entry_id),
        Some(176),
        "counted in the crotchets the row sent, the stamp's pattern kept"
    );
    assert_eq!(
        click_pattern(&model, &entry_id),
        Some(seven_eight_on_group_starts())
    );
}

#[test]
fn clearing_the_tempo_clears_a_stamped_pattern_with_it() {
    let mut model = summary_closed_with(sounding(168, Some(seven_eight_on_group_starts())));

    set_tempo(&mut model, None, false, Some(seven_eight_on_group_starts()));

    let entry_id = tempo_entry_id(&model);
    assert_eq!(achieved_tempo(&model, &entry_id), None);
    assert_eq!(click_pattern(&model, &entry_id), None);
}

#[test]
fn the_four_closing_events_round_trip_a_reading_on_the_bincode_wire() {
    let reading = sounding(168, Some(seven_eight_on_group_starts()));
    let events = [
        SessionEvent::SwitchVariation {
            entry_id: "e1".to_string(),
            variation_id: Some("v-d".to_string()),
            now: tap_at(),
            reading: reading.clone(),
        },
        SessionEvent::NextItem {
            now: tap_at(),
            next_item_started_at: tap_at(),
            reading: reading.clone(),
        },
        SessionEvent::EndSessionEarly {
            now: tap_at(),
            reading: reading.clone(),
        },
        SessionEvent::PrepareReflection {
            now: tap_at(),
            reading,
        },
    ];
    for event in events {
        crate::domain::types::assert_round_trips(Event::Session(event));
    }
}

/// A tempo set by hand records no pattern, since only a close stamps one
/// (#1761); the unit still names what the row displayed.
#[test]
fn a_silent_click_normalises_the_unit_but_records_no_pattern() {
    let mut model = model_with_summary();
    let entry_id = tempo_entry_id(&model);

    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: entry_id.clone(),
            play_id,
            tempo: Some(169),
            user_set: true,
            click: Some(seven_eight_on_group_starts()),
        }),
    );

    assert_eq!(achieved_tempo(&model, &entry_id), Some(85));
    assert_eq!(click_pattern(&model, &entry_id), None);
}

#[test]
fn a_click_that_sounds_no_beat_is_rejected_and_writes_nothing() {
    let mut model = model_with_summary();
    let entry_id = tempo_entry_id(&model);

    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: entry_id.clone(),
            play_id,
            tempo: Some(120),
            user_set: true,
            click: Some(ClickState {
                metre: Metre::default(),
                sounding: 0,
            }),
        }),
    );

    assert!(model.last_error.is_some());
    assert_eq!(achieved_tempo(&model, &entry_id), None);
}

#[test]
fn tempo_the_user_set_themselves_is_recorded() {
    let mut model = model_with_summary();
    let entry_id = tempo_entry_id(&model);

    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: entry_id.clone(),
            play_id,
            tempo: Some(120),
            user_set: true,
            click: None,
        }),
    );

    assert_eq!(achieved_tempo(&model, &entry_id), Some(120));
}

#[test]
fn an_untouched_default_is_not_recorded() {
    let mut model = model_with_summary();
    let entry_id = tempo_entry_id(&model);

    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: entry_id.clone(),
            play_id,
            tempo: Some(96),
            user_set: false,
            click: None,
        }),
    );

    assert_eq!(
        achieved_tempo(&model, &entry_id),
        None,
        "a pre-fill nobody looked at is not a measurement"
    );
}

#[test]
fn an_untouched_default_does_not_erase_a_measured_tempo() {
    let mut model = model_with_summary();
    let entry_id = tempo_entry_id(&model);

    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: entry_id.clone(),
            play_id,
            tempo: Some(120),
            user_set: true,
            click: None,
        }),
    );
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: entry_id.clone(),
            play_id,
            tempo: Some(96),
            user_set: false,
            click: None,
        }),
    );

    assert_eq!(
        achieved_tempo(&model, &entry_id),
        Some(120),
        "a report that is not a measurement must not destroy one that was"
    );
}

#[test]
fn clearing_the_tempo_needs_no_evidence() {
    let mut model = model_with_summary();
    let entry_id = tempo_entry_id(&model);

    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: entry_id.clone(),
            play_id,
            tempo: Some(120),
            user_set: true,
            click: None,
        }),
    );
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: entry_id.clone(),
            play_id,
            tempo: None,
            user_set: false,
            click: None,
        }),
    );

    assert_eq!(achieved_tempo(&model, &entry_id), None);
}

#[test]
fn update_entry_tempo_round_trips_on_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(Event::Session(SessionEvent::UpdateEntryTempo {
        entry_id: "entry-1".to_string(),
        play_id: "play-1".to_string(),
        tempo: Some(132),
        user_set: true,
        click: None,
    }));
}

#[test]
fn test_update_entry_tempo_none_clears() {
    let mut model = model_with_summary();

    let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
        s.entries[0].id.clone()
    } else {
        panic!("Expected Summary state");
    };

    // Set tempo to 120
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: entry_id.clone(),
            play_id,
            tempo: Some(120),
            user_set: true,
            click: None,
        }),
    );

    // Clear tempo by setting to None
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: entry_id.clone(),
            play_id,
            tempo: None,
            user_set: true,
            click: None,
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(play_of(&s.entries[0]).achieved_tempo, None);
    } else {
        panic!("Expected Summary state");
    }
}

#[test]
fn test_update_entry_tempo_rejected_on_skipped() {
    // Build a summary with a skipped entry: skip item 1, complete item 2
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(10);
    let t2 = t1 + chrono::Duration::seconds(30);

    // Skip first item
    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem { now: t1 }),
    );
    // Finish session (completes second item, transitions to summary)
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t2,
            next_item_started_at: t2,
            reading: TempoReading::silent(),
        }),
    );

    // Find the skipped entry
    let (skipped_entry_id, practised_entry_id) =
        if let SessionStatus::Summary(ref s) = model.session_status {
            let skipped = s
                .entries
                .iter()
                .find(|e| e.status == EntryStatus::Skipped)
                .expect("Should have a skipped entry");
            let practised = s
                .entries
                .iter()
                .find(|e| e.status == EntryStatus::Completed)
                .expect("Should have a completed entry");
            (skipped.id.clone(), practised.id.clone())
        } else {
            panic!("Expected Summary state");
        };

    // A skipped entry keeps no play of its own (#1739 decision 3), so the
    // sheet can only ever send a play id from the practised item.
    let play_id = first_play_id(&model, &practised_entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: skipped_entry_id.clone(),
            play_id,
            tempo: Some(100),
            user_set: true,
            click: None,
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        let skipped = s.entries.iter().find(|e| e.id == skipped_entry_id).unwrap();
        assert!(skipped.plays.is_empty(), "no play means no tempo recorded");
    }
}

#[test]
fn test_update_entry_tempo_rejected_out_of_range() {
    let mut model = model_with_summary();

    let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
        s.entries[0].id.clone()
    } else {
        panic!("Expected Summary state");
    };

    // Tempo 0: out of range
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: entry_id.clone(),
            play_id,
            tempo: Some(0),
            user_set: true,
            click: None,
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(play_of(&s.entries[0]).achieved_tempo, None);
    }

    // Tempo 501: out of range
    let play_id = first_play_id(&model, &entry_id);
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryTempo {
            entry_id: entry_id.clone(),
            play_id,
            tempo: Some(501),
            user_set: true,
            click: None,
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(play_of(&s.entries[0]).achieved_tempo, None);
    }
}

// --- SessionsData Serialization Test ---

#[test]
fn test_sessions_data_serialization() {
    use crate::domain::types::SessionsData;

    let data = SessionsData { sessions: vec![] };
    let json = serde_json::to_string(&data).unwrap();
    let parsed: SessionsData = serde_json::from_str(&json).unwrap();
    assert!(parsed.sessions.is_empty());
}

// --- Intention Tests ---

#[test]
fn test_set_entry_intention() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );

    let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
        b.entries[0].id.clone()
    } else {
        panic!("Expected Building state");
    };

    update(
        &mut model,
        Event::Session(SessionEvent::SetEntryIntention {
            entry_id,
            intention: Some("Work on left hand".to_string()),
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Building(ref b) = model.session_status {
        assert_eq!(
            b.entries[0].intention,
            Some("Work on left hand".to_string())
        );
    } else {
        panic!("Expected Building state");
    }
}

#[test]
fn test_set_entry_variant_tags_the_rung_while_building() {
    let mut model = model_with_library();
    give_exercise_a_ladder(&mut model);
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "exercise-1".to_string(),
        }),
    );

    let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
        assert_eq!(
            b.entries[0].planned_variation_id, None,
            "entries start with no rung"
        );
        b.entries[0].id.clone()
    } else {
        panic!("Expected Building state");
    };

    update(
        &mut model,
        Event::Session(SessionEvent::SetEntryVariant {
            entry_id: entry_id.clone(),
            variant_id: Some("v-c".to_string()),
        }),
    );
    assert!(model.last_error.is_none());
    if let SessionStatus::Building(ref b) = model.session_status {
        assert_eq!(b.entries[0].planned_variation_id, Some("v-c".to_string()));
    } else {
        panic!("Expected Building state");
    }

    update(
        &mut model,
        Event::Session(SessionEvent::SetEntryVariant {
            entry_id,
            variant_id: None,
        }),
    );
    if let SessionStatus::Building(ref b) = model.session_status {
        assert_eq!(
            b.entries[0].planned_variation_id, None,
            "None clears the rung"
        );
    } else {
        panic!("Expected Building state");
    }
}

#[test]
fn test_set_entry_variant_unknown_entry_surfaces_error() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::SetEntryVariant {
            entry_id: "nope".to_string(),
            variant_id: Some("var-1".to_string()),
        }),
    );
    assert!(model.last_error.is_some());
}

// --- Rep Counter Tests ---

const TAP_AT: &str = "2026-09-03T09:00:00Z";

fn tap_at() -> DateTime<Utc> {
    TAP_AT.parse().expect("fixed timestamp")
}

fn got_it() -> Event {
    Event::Session(SessionEvent::RepGotIt { now: tap_at() })
}

fn missed() -> Event {
    Event::Session(SessionEvent::RepMissed { now: tap_at() })
}

fn actions(entry: &SetlistEntry) -> Option<Vec<RepAction>> {
    play_of(entry)
        .rep_history
        .as_ref()
        .map(|h| h.iter().map(|e| e.action).collect())
}

fn active_entry(model: &Model, index: usize) -> &SetlistEntry {
    let SessionStatus::Active(ref a) = model.session_status else {
        panic!("Expected Active state");
    };
    &a.entries[index]
}

/// Helper: create an active session with a rep target on the first item.
fn model_with_active_session_and_rep(target: u8) -> (Model, DateTime<Utc>) {
    let mut model = model_with_library();
    let now = Utc::now();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-2".to_string(),
        }),
    );

    // Set rep target on first entry during building
    if let SessionStatus::Building(ref mut b) = model.session_status {
        b.entries[0].planned_rep_target = Some(target);
    } else {
        panic!("Expected Building state");
    }

    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );
    (model, now)
}

/// An untouched counter records nothing (T19): a builder target alone must
/// not bank a zero and an empty history on session start.
#[test]
fn test_start_session_does_not_bank_rep_state() {
    let (model, _now) = model_with_active_session_and_rep(5);

    let targeted = active_entry(&model, 0);
    assert_eq!(play_of(targeted).rep_target, Some(5));
    assert_eq!(play_of(targeted).rep_count, None);
    assert_eq!(play_of(targeted).rep_target_reached, None);
    assert_eq!(play_of(targeted).rep_history, None);

    let untouched = active_entry(&model, 1);
    assert!(
        untouched.plays.is_empty(),
        "an item not yet reached has opened no play, so it banks nothing"
    );
}

#[test]
fn test_rep_got_it_increments() {
    let (mut model, _now) = model_with_active_session_and_rep(5);

    update(&mut model, got_it());
    update(&mut model, got_it());
    update(&mut model, got_it());

    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(play_of(&a.entries[0]).rep_count, Some(3));
        assert_eq!(play_of(&a.entries[0]).rep_target_reached, Some(false));
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_rep_got_it_reaches_target() {
    let (mut model, _now) = model_with_active_session_and_rep(3);

    update(&mut model, got_it());
    update(&mut model, got_it());
    update(&mut model, got_it());

    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(play_of(&a.entries[0]).rep_count, Some(3));
        assert_eq!(play_of(&a.entries[0]).rep_target_reached, Some(true));
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_rep_got_it_frozen_after_target_reached() {
    let (mut model, _now) = model_with_active_session_and_rep(3);

    // Reach target
    for _ in 0..3 {
        update(&mut model, got_it());
    }

    // Additional got-it should not increase count
    update(&mut model, got_it());

    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(play_of(&a.entries[0]).rep_count, Some(3));
        assert_eq!(play_of(&a.entries[0]).rep_target_reached, Some(true));
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_rep_missed_decrements() {
    let (mut model, _now) = model_with_active_session_and_rep(5);

    update(&mut model, got_it());
    update(&mut model, got_it());
    update(&mut model, got_it());

    update(&mut model, missed());

    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(play_of(&a.entries[0]).rep_count, Some(2));
        assert_eq!(play_of(&a.entries[0]).rep_target_reached, Some(false));
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_rep_missed_floor_zero() {
    let (mut model, _now) = model_with_active_session_and_rep(5);

    // Miss with count at 0
    update(&mut model, missed());

    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(play_of(&a.entries[0]).rep_count, Some(0));
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_rep_missed_at_target_steps_back_and_can_be_re_earned() {
    let (mut model, _now) = model_with_active_session_and_rep(3);
    for _ in 0..3 {
        update(&mut model, got_it());
    }

    update(&mut model, missed());

    let entry = active_entry(&model, 0);
    assert_eq!(play_of(entry).rep_count, Some(2));
    assert_eq!(play_of(entry).rep_target_reached, Some(false));
    assert_eq!(
        actions(entry).map(|a| a.last().copied()),
        Some(Some(RepAction::Missed))
    );

    update(&mut model, got_it());

    let entry = active_entry(&model, 0);
    assert_eq!(play_of(entry).rep_count, Some(3));
    assert_eq!(play_of(entry).rep_target_reached, Some(true));
}

#[test]
fn test_first_got_it_on_untouched_entry_writes_all_four_fields() {
    let (mut model, _now) = model_with_active_session(2);
    assert_eq!(play_of(active_entry(&model, 0)).rep_target, None);

    update(&mut model, got_it());

    let entry = active_entry(&model, 0);
    assert_eq!(
        play_of(entry).rep_target,
        Some(validation::DEFAULT_REP_TARGET)
    );
    assert_eq!(play_of(entry).rep_count, Some(1));
    assert_eq!(play_of(entry).rep_target_reached, Some(false));
    assert_eq!(
        play_of(entry).rep_history,
        Some(vec![RepEvent {
            action: RepAction::Success,
            at: tap_at()
        }])
    );
}

#[test]
fn test_first_missed_on_untouched_entry_records_the_miss_at_zero() {
    let (mut model, _now) = model_with_active_session(2);

    update(&mut model, missed());

    let entry = active_entry(&model, 0);
    assert_eq!(
        play_of(entry).rep_target,
        Some(validation::DEFAULT_REP_TARGET)
    );
    assert_eq!(play_of(entry).rep_count, Some(0));
    assert_eq!(play_of(entry).rep_target_reached, Some(false));
    assert_eq!(actions(entry), Some(vec![RepAction::Missed]));
}

#[test]
fn test_first_tap_keeps_a_builder_target() {
    let (mut model, _now) = model_with_active_session_and_rep(7);

    update(&mut model, got_it());

    let entry = active_entry(&model, 0);
    assert_eq!(play_of(entry).rep_target, Some(7));
    assert_eq!(play_of(entry).rep_count, Some(1));
}

#[test]
fn test_rep_history_carries_each_taps_time_in_order() {
    let (mut model, _now) = model_with_active_session(2);
    let first = tap_at();
    let second = first + chrono::Duration::seconds(40);
    let third = second + chrono::Duration::seconds(55);

    update(
        &mut model,
        Event::Session(SessionEvent::RepGotIt { now: first }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::RepMissed { now: second }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::RepGotIt { now: third }),
    );

    assert_eq!(
        play_of(active_entry(&model, 0)).rep_history,
        Some(vec![
            RepEvent {
                action: RepAction::Success,
                at: first
            },
            RepEvent {
                action: RepAction::Missed,
                at: second
            },
            RepEvent {
                action: RepAction::Success,
                at: third
            },
        ])
    );
}

#[test]
fn test_rep_state_frozen_on_next_item() {
    let (mut model, start) = model_with_active_session_and_rep(5);

    // Got-it 3 times (partial progress)
    update(&mut model, got_it());
    update(&mut model, got_it());
    update(&mut model, got_it());

    let now = start + chrono::Duration::seconds(30);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now,
            next_item_started_at: now,
            reading: TempoReading::silent(),
        }),
    );

    if let SessionStatus::Active(ref a) = model.session_status {
        // First entry frozen: 3/5, target not reached
        assert_eq!(play_of(&a.entries[0]).rep_count, Some(3));
        assert_eq!(play_of(&a.entries[0]).rep_target_reached, Some(false));
        // Now on second item
        assert_eq!(a.current_index, 1);
    } else {
        panic!("Expected Active state");
    }
}

/// Repetitions banked before a skip are a record of practice and survive
/// it, frozen, as they did before plays existed. What a skip discards is
/// the play that recorded nothing (#1739 decision 3).
#[test]
fn test_rep_state_frozen_on_skip_item() {
    let (mut model, start) = model_with_active_session_and_rep(5);

    // Got-it once
    update(&mut model, got_it());

    let now = start + chrono::Duration::seconds(10);
    update(&mut model, Event::Session(SessionEvent::SkipItem { now }));

    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(a.entries[0].status, EntryStatus::Skipped);
        assert_eq!(a.entries[0].plays.len(), 1);
        assert_eq!(a.entries[0].plays[0].rep_count, Some(1));
        assert_eq!(a.entries[0].plays[0].rep_target, Some(5));
        assert_eq!(a.entries[0].plays[0].rep_target_reached, Some(false));
        assert_eq!(a.entries[0].planned_rep_target, Some(5));
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_rep_state_in_summary_after_finish() {
    let (mut model, start) = model_with_active_session_and_rep(3);

    // Reach target
    for _ in 0..3 {
        update(&mut model, got_it());
    }
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start,
            next_item_started_at: start,
            reading: TempoReading::silent(),
        }),
    );

    let now = start + chrono::Duration::seconds(60);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now,
            next_item_started_at: now,
            reading: TempoReading::silent(),
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(play_of(&s.entries[0]).rep_target, Some(3));
        assert_eq!(play_of(&s.entries[0]).rep_count, Some(3));
        assert_eq!(play_of(&s.entries[0]).rep_target_reached, Some(true));
    } else {
        panic!("Expected Summary state");
    }
}

#[test]
fn test_rep_state_persisted_through_save() {
    let (mut model, start) = model_with_active_session_and_rep(3);

    // Reach target
    for _ in 0..3 {
        update(&mut model, got_it());
    }
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start,
            next_item_started_at: start,
            reading: TempoReading::silent(),
        }),
    );

    let t1 = start + chrono::Duration::seconds(60);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading: TempoReading::silent(),
        }),
    );

    let t2 = t1 + chrono::Duration::seconds(5);
    update(
        &mut model,
        Event::Session(SessionEvent::SaveSession { now: t2 }),
    );
    update(
        &mut model,
        Event::SessionStoreWritten(crate::persistence::PersistenceOutput::Ack),
    );

    assert_eq!(model.sessions.len(), 1);
    assert_eq!(play_of(&model.sessions[0].entries[0]).rep_target, Some(3));
    assert_eq!(play_of(&model.sessions[0].entries[0]).rep_count, Some(3));
    assert_eq!(
        play_of(&model.sessions[0].entries[0]).rep_target_reached,
        Some(true)
    );
}

#[test]
fn test_a_tap_touches_only_the_current_entry() {
    let (mut model, _now) = model_with_active_session(2);

    update(&mut model, got_it());

    let other = active_entry(&model, 1);
    assert!(
        other.plays.is_empty(),
        "the tap opened no play on an item that is not current"
    );
}

#[test]
fn test_rep_got_it_capped_at_target() {
    let (mut model, _now) = model_with_active_session_and_rep(3);

    // Try to go beyond target
    for _ in 0..10 {
        update(&mut model, got_it());
    }

    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(play_of(&a.entries[0]).rep_count, Some(3)); // capped at target
        assert_eq!(play_of(&a.entries[0]).rep_target_reached, Some(true));
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_rep_state_frozen_on_end_session_early() {
    let (mut model, now) = model_with_active_session_and_rep(5);

    // Increment rep count twice on item 1
    update(&mut model, got_it());
    update(&mut model, got_it());

    // End session early (item 2 is never reached)
    let t1 = now + chrono::Duration::seconds(30);
    update(
        &mut model,
        Event::Session(SessionEvent::EndSessionEarly {
            now: t1,
            reading: TempoReading::silent(),
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        // Item 1: rep state frozen at 2/5, not reached
        assert_eq!(play_of(&s.entries[0]).rep_target, Some(5));
        assert_eq!(play_of(&s.entries[0]).rep_count, Some(2));
        assert_eq!(play_of(&s.entries[0]).rep_target_reached, Some(false));
        assert_eq!(s.entries[0].status, EntryStatus::Completed);

        // Item 2: never reached, so it records nothing at all
        assert!(s.entries[1].plays.is_empty());
        assert_eq!(s.entries[1].status, EntryStatus::NotAttempted);
    } else {
        panic!("Expected Summary state");
    }
}

// ── SetRepTarget (Building phase) tests ──────────────────────────

#[test]
fn test_set_rep_target_in_building() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );

    let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
        b.entries[0].id.clone()
    } else {
        panic!("Expected Building state");
    };

    update(
        &mut model,
        Event::Session(SessionEvent::SetRepTarget {
            entry_id: entry_id.clone(),
            target: Some(7),
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Building(ref b) = model.session_status {
        assert_eq!(b.entries[0].planned_rep_target, Some(7));
        assert!(
            b.entries[0].plays.is_empty(),
            "a target is a plan: it banks no progress"
        );
    } else {
        panic!("Expected Building state");
    }
}

#[test]
fn test_set_rep_target_clear() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );

    let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
        b.entries[0].id.clone()
    } else {
        panic!("Expected Building state");
    };

    // Set target first
    update(
        &mut model,
        Event::Session(SessionEvent::SetRepTarget {
            entry_id: entry_id.clone(),
            target: Some(5),
        }),
    );

    // Then clear it
    update(
        &mut model,
        Event::Session(SessionEvent::SetRepTarget {
            entry_id,
            target: None,
        }),
    );

    assert!(model.last_error.is_none());
    if let SessionStatus::Building(ref b) = model.session_status {
        assert_eq!(b.entries[0].planned_rep_target, None);
    } else {
        panic!("Expected Building state");
    }
}

#[test]
fn test_set_rep_target_invalid_value() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );

    let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
        b.entries[0].id.clone()
    } else {
        panic!("Expected Building state");
    };

    // Target below minimum (3)
    update(
        &mut model,
        Event::Session(SessionEvent::SetRepTarget {
            entry_id: entry_id.clone(),
            target: Some(1),
        }),
    );

    assert!(model.last_error.is_some());
    if let SessionStatus::Building(ref b) = model.session_status {
        assert_eq!(b.entries[0].planned_rep_target, None); // unchanged
    } else {
        panic!("Expected Building state");
    }

    // Target above maximum (10)
    model.last_error = None;
    update(
        &mut model,
        Event::Session(SessionEvent::SetRepTarget {
            entry_id,
            target: Some(15),
        }),
    );

    assert!(model.last_error.is_some());
}

#[test]
fn test_set_rep_target_flows_to_active() {
    let mut model = model_with_library();
    let now = Utc::now();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );

    let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
        b.entries[0].id.clone()
    } else {
        panic!("Expected Building state");
    };

    update(
        &mut model,
        Event::Session(SessionEvent::SetRepTarget {
            entry_id,
            target: Some(8),
        }),
    );

    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );

    let entry = active_entry(&model, 0);
    assert_eq!(play_of(entry).rep_target, Some(8));
    assert_eq!(
        play_of(entry).rep_count,
        None,
        "nothing is banked before the first tap"
    );
    assert_eq!(play_of(entry).rep_target_reached, None);
}

// ── SetEntryDuration (Building phase) tests ──────────────────────

fn building_with_one_entry() -> (Model, String) {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    add(&mut model, "piece-1");
    let entry_id = building_entries(&model)[0].id.clone();
    (model, entry_id)
}

fn set_duration(model: &mut Model, entry_id: &str, duration_secs: Option<u32>) {
    update(
        model,
        Event::Session(SessionEvent::SetEntryDuration {
            entry_id: entry_id.to_string(),
            duration_secs,
        }),
    );
}

#[test]
fn set_entry_duration_lands_at_either_bound() {
    let (mut model, entry_id) = building_with_one_entry();
    assert_eq!(building_entries(&model)[0].planned_duration_secs, None);

    set_duration(&mut model, &entry_id, Some(60));
    assert!(model.last_error.is_none());
    assert_eq!(building_entries(&model)[0].planned_duration_secs, Some(60));

    set_duration(&mut model, &entry_id, Some(3600));
    assert!(model.last_error.is_none());
    assert_eq!(
        building_entries(&model)[0].planned_duration_secs,
        Some(3600)
    );
}

#[test]
fn set_entry_duration_outside_the_range_is_refused_and_keeps_the_plan() {
    for out_of_range in [0, 59, 3601] {
        let (mut model, entry_id) = building_with_one_entry();
        set_duration(&mut model, &entry_id, Some(600));

        set_duration(&mut model, &entry_id, Some(out_of_range));

        assert!(model.last_error.is_some(), "{out_of_range} was accepted");
        assert_eq!(building_entries(&model)[0].planned_duration_secs, Some(600));
    }
}

#[test]
fn set_entry_duration_none_clears_the_plan() {
    let (mut model, entry_id) = building_with_one_entry();
    set_duration(&mut model, &entry_id, Some(600));
    assert_eq!(building_entries(&model)[0].planned_duration_secs, Some(600));

    set_duration(&mut model, &entry_id, None);

    assert!(model.last_error.is_none());
    assert_eq!(building_entries(&model)[0].planned_duration_secs, None);
}

// --- Rep History Tests (US1) ---

#[test]
fn test_rep_history_appended_on_got_it() {
    let (mut model, _now) = model_with_active_session_and_rep(5);

    update(&mut model, got_it());
    update(&mut model, got_it());

    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(
            actions(&a.entries[0]),
            Some(vec![RepAction::Success, RepAction::Success])
        );
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_rep_history_appended_on_missed() {
    let (mut model, _now) = model_with_active_session_and_rep(5);

    update(&mut model, got_it());
    update(&mut model, missed());

    if let SessionStatus::Active(ref a) = model.session_status {
        assert_eq!(
            actions(&a.entries[0]),
            Some(vec![RepAction::Success, RepAction::Missed])
        );
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_rep_history_frozen_on_next_item() {
    let (mut model, start) = model_with_active_session_and_rep(5);

    // Record some actions
    update(&mut model, got_it());
    update(&mut model, missed());
    update(&mut model, got_it());

    // Move to next item
    let next_time = start + chrono::Duration::seconds(60);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: next_time,
            next_item_started_at: next_time,
            reading: TempoReading::silent(),
        }),
    );

    if let SessionStatus::Active(ref a) = model.session_status {
        // First entry's history should be frozen with the recorded actions
        assert_eq!(
            actions(&a.entries[0]),
            Some(vec![
                RepAction::Success,
                RepAction::Missed,
                RepAction::Success
            ])
        );
    } else {
        panic!("Expected Active state");
    }
}

#[test]
fn test_update_session_score_sets_and_validates() {
    let mut model = model_with_summary();

    update(
        &mut model,
        Event::Session(SessionEvent::UpdateSessionScore { score: Some(8) }),
    );
    assert!(model.last_error.is_none());
    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(s.session_score, Some(8));
    } else {
        panic!("Expected Summary state");
    }

    // Out of range is rejected, leaving the prior value intact.
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateSessionScore { score: Some(11) }),
    );
    assert!(model.last_error.is_none());
    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(s.session_score, Some(8));
    }
}

#[test]
fn test_rep_history_persisted_through_save() {
    let (mut model, start) = model_with_active_session_and_rep(3);

    // Hit got it 3 times to reach target
    update(&mut model, got_it());
    update(&mut model, got_it());
    update(&mut model, got_it());
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start,
            next_item_started_at: start,
            reading: TempoReading::silent(),
        }),
    );

    // Finish session
    let end_time = start + chrono::Duration::seconds(120);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: end_time,
            next_item_started_at: end_time,
            reading: TempoReading::silent(),
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(
            actions(&s.entries[0]),
            Some(vec![
                RepAction::Success,
                RepAction::Success,
                RepAction::Success
            ])
        );
    } else {
        panic!("Expected Summary state");
    }
}

/// `group_id` rides `SetlistEntry` across the bincode FFI wire (in the
/// persisted `PracticeSession` and the ViewModel). It is the sole input to
/// the B1 context derivation, so a silent drop on the wire would erase every
/// piece context. Guard the whole entry, with `group_id` set, against the
/// #846 class.
#[test]
fn setlist_entry_with_group_id_round_trips_on_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(SetlistEntry {
        id: "e1".to_string(),
        item_id: "ex-1".to_string(),
        item_title: "Scales".to_string(),
        item_type: ItemKind::Exercise,
        position: 0,
        duration_secs: 300,
        status: EntryStatus::Completed,
        notes: Some("warm up".to_string()),
        intention: Some("even tone".to_string()),
        planned_duration_secs: Some(300),
        group_id: Some("g1".to_string()),
        planned_variation_id: Some("v-1".to_string()),
        planned_rep_target: Some(5),
        plays: vec![VariationPlay {
            id: "e1-play".to_string(),
            variation_id: Some("v-1".to_string()),
            started_at: tap_at(),
            seconds: 300,
            rep_target: Some(5),
            rep_count: Some(5),
            rep_target_reached: Some(true),
            rep_history: Some(vec![
                RepEvent {
                    action: RepAction::Success,
                    at: tap_at(),
                },
                RepEvent {
                    action: RepAction::Missed,
                    at: tap_at(),
                },
            ]),
            achieved_tempo: Some(96),
            click_pattern: None,
            score: Some(6),
        }],
    });
}

fn pinned_active_session() -> ActiveSession {
    let started: DateTime<Utc> = "2026-09-03T08:47:00Z".parse().expect("fixed");
    let mut touched = create_entry("p1", "Clair de Lune", ItemKind::Piece, 0);
    touched.id = "e1".to_string();
    touched.status = EntryStatus::Completed;
    touched.duration_secs = 300;
    touched.notes = Some("phrasing".to_string());
    touched.intention = Some("evenness".to_string());
    touched.planned_duration_secs = Some(300);
    touched.group_id = Some("g1".to_string());
    touched.planned_variation_id = Some("v-1".to_string());
    touched.planned_rep_target = Some(10);
    touched.plays = vec![VariationPlay {
        id: "e1-play".to_string(),
        variation_id: Some("v-1".to_string()),
        started_at: tap_at(),
        seconds: 300,
        rep_target: Some(10),
        rep_count: Some(1),
        rep_target_reached: Some(false),
        rep_history: Some(vec![
            RepEvent {
                action: RepAction::Success,
                at: tap_at(),
            },
            RepEvent {
                action: RepAction::Missed,
                at: tap_at() + chrono::Duration::seconds(40),
            },
            RepEvent {
                action: RepAction::Success,
                at: tap_at() + chrono::Duration::seconds(95),
            },
        ]),
        achieved_tempo: Some(120),
        click_pattern: Some(seven_eight_on_group_starts()),
        score: Some(4),
    }];
    let mut untouched = create_entry("x1", "Scales", ItemKind::Exercise, 1);
    untouched.id = "e2".to_string();
    ActiveSession {
        id: "s1".to_string(),
        entries: vec![touched, untouched],
        current_index: 1,
        current_item_started_at: tap_at(),
        session_started_at: started,
    }
}

const PINNED_ACTIVE_SESSION_HEX: &str = concat!(
    "02000000000000007331020000000000000002000000000000006531020000000000000070310d",
    "00000000000000436c616972206465204c756e650000000000000000000000002c010000000000",
    "00000000000108000000000000007068726173696e670108000000000000006576656e6e657373",
    "012c0100000102000000000000006731010300000000000000762d31010a010000000000000007",
    "0000000000000065312d706c6179010300000000000000762d311400000000000000323032362d",
    "30392d30335430393a30303a30305a2c01000000000000010a0101010001030000000000000001",
    "0000001400000000000000323032362d30392d30335430393a30303a30305a0000000014000000",
    "00000000323032362d30392d30335430393a30303a34305a010000001400000000000000323032",
    "362d30392d30335430393a30313a33355a01780001070801030000000000000003020229000104",
    "020000000000000065320200000000000000783106000000000000005363616c65730100000001",
    "000000000000000000000000000000020000000000000000000000000000000000010000000000",
    "00001400000000000000323032362d30392d30335430393a30303a30305a140000000000000032",
    "3032362d30392d30335430383a34373a30305a",
);

const PINNED_BLOB_VERSION: u32 = 4;

/// The blob is positional bincode written by one build and read by the
/// next (#1345); the shell's storage key follows `BLOB_VERSION`, so a bump
/// here is what retires the old blob (#1116). The test cannot tell a
/// re-pin with a bump from one without: the failure message is the protocol.
#[test]
fn active_session_blob_wire_is_pinned() {
    use crux_core::bridge::{BincodeFfiFormat, FfiFormat};
    let session = pinned_active_session();
    let mut bytes = Vec::new();
    BincodeFfiFormat::serialize(&mut bytes, &session).expect("serialize");
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        (ActiveSession::BLOB_VERSION, hex.as_str()),
        (PINNED_BLOB_VERSION, PINNED_ACTIVE_SESSION_HEX),
        "the crash-recovery blob changed shape: bump ActiveSession::BLOB_VERSION and PINNED_BLOB_VERSION, then re-pin the hex. Never only the hex."
    );
    let back: ActiveSession =
        BincodeFfiFormat::deserialize(&bytes).expect("must decode on the FFI wire (#846)");
    assert_eq!(back, session);
}

/// The timestamped tap events and the history they write cross the bridge
/// (#846 class); pinned before any screen sends the new payloads.
#[test]
fn rep_tap_events_round_trip_on_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(got_it());
    crate::domain::types::assert_round_trips(missed());
    crate::domain::types::assert_round_trips(pinned_active_session());
}

/// The extended tempo payload crosses the bridge with the click state on
/// both its `Some` and `None` sides (#846 class), pinned before any screen
/// sends it.
#[test]
fn update_entry_tempo_with_click_state_round_trips_on_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(Event::Session(SessionEvent::UpdateEntryTempo {
        entry_id: "e1".to_string(),
        play_id: "play-1".to_string(),
        tempo: Some(168),
        user_set: false,
        click: Some(seven_eight_on_group_starts()),
    }));
    crate::domain::types::assert_round_trips(Event::Session(SessionEvent::UpdateEntryTempo {
        entry_id: "e1".to_string(),
        play_id: "play-1".to_string(),
        tempo: None,
        user_set: true,
        click: None,
    }));
}

/// The variation tag (#1083, #1739) is the sole input to per-variation
/// score derivation, and it now rides the wire twice: as the entry's plan
/// and as the play's record. A silent drop on either would unscore every
/// variation. Guard both, and the event that writes the plan, against the
/// #846 class.
#[test]
fn set_entry_variant_payloads_round_trip_on_ffi_bincode_wire() {
    crate::domain::types::assert_round_trips(SetlistEntry {
        id: "e1".to_string(),
        item_id: "ex-1".to_string(),
        item_title: "Scales".to_string(),
        item_type: ItemKind::Exercise,
        position: 0,
        duration_secs: 300,
        status: EntryStatus::Completed,
        notes: None,
        intention: None,
        planned_duration_secs: None,
        group_id: None,
        planned_variation_id: Some("v1".to_string()),
        planned_rep_target: None,
        plays: vec![VariationPlay {
            id: "e1-play".to_string(),
            variation_id: Some("v1".to_string()),
            started_at: tap_at(),
            seconds: 300,
            score: Some(8),
            ..VariationPlay::fixture()
        }],
    });

    crate::domain::types::assert_round_trips(crate::app::Event::Session(
        SessionEvent::SetEntryVariant {
            entry_id: "e1".to_string(),
            variant_id: Some("v1".to_string()),
        },
    ));
}

// ── Variations and plays (#1739) ───────────────────────────────────

/// An exercise with two live variations and one tombstoned, in a session
/// that has already started. `v-gone` is the dead one.
fn model_with_variations() -> (Model, DateTime<Utc>) {
    let (model, now) = building_with_variations();
    let mut model = model;
    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );
    (model, now)
}

fn building_with_variations() -> (Model, DateTime<Utc>) {
    use crate::domain::variant::Variant;

    let mut model = model_with_library();
    let now = Utc::now();
    let exercise = model
        .items
        .iter_mut()
        .find(|i| i.id == "exercise-1")
        .expect("the library fixture has exercise-1");
    exercise.variants = vec![
        Variant {
            id: "v-c".to_string(),
            label: "C".to_string(),
            position: 0,
            updated_at: now,
            deleted_at: None,
        },
        Variant {
            id: "v-d".to_string(),
            label: "D".to_string(),
            position: 1,
            updated_at: now,
            deleted_at: None,
        },
        Variant {
            id: "v-gone".to_string(),
            label: "E flat".to_string(),
            position: 2,
            updated_at: now,
            deleted_at: Some(now),
        },
    ];

    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "exercise-1".to_string(),
        }),
    );
    (model, now)
}

fn only_entry(model: &Model) -> &SetlistEntry {
    &session_entries(model)[0]
}

#[test]
fn starting_a_session_opens_one_play_seeded_from_the_plan() {
    let (mut model, now) = building_with_variations();
    let entry_id = only_entry(&model).id.clone();

    update(
        &mut model,
        Event::Session(SessionEvent::SetEntryVariant {
            entry_id,
            variant_id: Some("v-c".to_string()),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::StartSession { now }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(play_of(entry).variation_id.as_deref(), Some("v-c"));
    assert_eq!(entry.planned_variation_id.as_deref(), Some("v-c"));
}

#[test]
fn a_piece_records_exactly_one_unattributed_play() {
    let (model, _) = model_with_active_session(1);

    let entry = only_entry(&model);
    assert_eq!(entry.item_type, ItemKind::Piece);
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(play_of(entry).variation_id, None);
}

#[test]
fn switching_mid_item_closes_the_open_play_and_opens_another() {
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();
    let switched_at = start + chrono::Duration::seconds(180);

    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id,
            variation_id: Some("v-d".to_string()),
            now: switched_at,
            reading: TempoReading::silent(),
        }),
    );

    assert!(model.last_error.is_none());
    let entry = only_entry(&model);
    assert_eq!(entry.plays.len(), 2);
    assert_eq!(entry.plays[0].seconds, 180);
    assert_eq!(entry.plays[1].variation_id.as_deref(), Some("v-d"));
    assert_eq!(entry.plays[1].seconds, 0);
}

#[test]
fn switching_to_the_variation_already_open_writes_nothing() {
    // StartSession seeds the open play with v-c, the first live variation
    // (#1758), so the genuine switch under test is to v-d; switching to
    // v-c here would itself be a no-op rather than exercising one.
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();
    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id: entry_id.clone(),
            variation_id: Some("v-d".to_string()),
            now: start + chrono::Duration::seconds(60),
            reading: TempoReading::silent(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::RepGotIt {
            now: start + chrono::Duration::seconds(70),
        }),
    );

    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id,
            variation_id: Some("v-d".to_string()),
            now: start + chrono::Duration::seconds(80),
            reading: TempoReading::silent(),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.plays.len(), 2, "the stray tap opened nothing");
    assert_eq!(
        play_of(entry).rep_count,
        Some(1),
        "the stray tap did not clear the dots"
    );
}

#[test]
fn repetitions_and_tempo_land_on_the_open_play_and_reset_on_a_switch() {
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();

    update(
        &mut model,
        Event::Session(SessionEvent::RepGotIt {
            now: start + chrono::Duration::seconds(10),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::RepGotIt {
            now: start + chrono::Duration::seconds(20),
        }),
    );
    assert_eq!(play_of(only_entry(&model)).rep_count, Some(2));

    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id,
            variation_id: Some("v-d".to_string()),
            now: start + chrono::Duration::seconds(30),
            reading: TempoReading::silent(),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(
        entry.plays[0].rep_count,
        Some(2),
        "the first play keeps its"
    );
    assert_eq!(play_of(entry).rep_count, None, "the new play starts empty");
}

#[test]
fn scoring_names_the_play_it_marks() {
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();
    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id: entry_id.clone(),
            variation_id: Some("v-d".to_string()),
            now: start + chrono::Duration::seconds(60),
            reading: TempoReading::silent(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start + chrono::Duration::seconds(600),
            next_item_started_at: start + chrono::Duration::seconds(600),
            reading: TempoReading::silent(),
        }),
    );
    let first = only_entry(&model).plays[0].id.clone();

    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id,
            play_id: first,
            score: Some(7),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.plays[0].score, Some(7));
    assert_eq!(entry.plays[1].score, None, "only the named play is marked");
}

#[test]
fn a_play_id_from_another_entry_is_rejected() {
    let (mut model, start) = model_with_active_session(2);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start + chrono::Duration::seconds(300),
            next_item_started_at: start + chrono::Duration::seconds(300),
            reading: TempoReading::silent(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start + chrono::Duration::seconds(600),
            next_item_started_at: start + chrono::Duration::seconds(600),
            reading: TempoReading::silent(),
        }),
    );
    let first_entry = session_entries(&model)[0].id.clone();
    let second_entry = session_entries(&model)[1].id.clone();
    let foreign = first_play_id(&model, &second_entry);

    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: first_entry.clone(),
            play_id: foreign,
            score: Some(7),
        }),
    );

    assert!(model.last_error.is_some());
    let entry = session_entries(&model)
        .iter()
        .find(|e| e.id == first_entry)
        .expect("the entry is there");
    assert!(entry.plays.iter().all(|p| p.score.is_none()));
}

#[test]
fn switching_to_a_dead_variation_is_rejected() {
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();

    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id,
            variation_id: Some("v-gone".to_string()),
            now: start + chrono::Duration::seconds(60),
            reading: TempoReading::silent(),
        }),
    );

    assert!(model.last_error.is_some());
    assert_eq!(only_entry(&model).plays.len(), 1);
}

#[test]
fn an_entry_stops_at_the_play_cap() {
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();

    // Alternate, so every switch is a real one rather than a no-op.
    for i in 0..validation::MAX_PLAYS_PER_ENTRY + 4 {
        let variation = if i % 2 == 0 { "v-d" } else { "v-c" };
        update(
            &mut model,
            Event::Session(SessionEvent::SwitchVariation {
                entry_id: entry_id.clone(),
                variation_id: Some(variation.to_string()),
                now: start + chrono::Duration::seconds(60 * (i as i64 + 1)),
                reading: TempoReading::silent(),
            }),
        );
    }

    assert_eq!(
        only_entry(&model).plays.len(),
        validation::MAX_PLAYS_PER_ENTRY
    );
    assert!(model.last_error.is_some());
}

#[test]
fn finishing_drops_a_play_that_recorded_nothing() {
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();
    let opened = only_entry(&model).plays[0].id.clone();
    // A stray tap on the picker, two seconds before the end.
    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id,
            variation_id: Some("v-d".to_string()),
            now: start + chrono::Duration::seconds(298),
            reading: TempoReading::silent(),
        }),
    );

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start + chrono::Duration::seconds(300),
            next_item_started_at: start + chrono::Duration::seconds(300),
            reading: TempoReading::silent(),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(
        entry.plays[0].id, opened,
        "the stray tap's play went, not the real one"
    );
}

/// The drop must never empty a practised entry, or the item-complete sheet
/// would hold a `play_id` the core has just deleted and every mark it sent
/// would be refused (decision 3's invariant).
#[test]
fn finishing_keeps_the_only_play_however_short() {
    let (mut model, start) = model_with_variations();
    let opened = only_entry(&model).plays[0].id.clone();

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start + chrono::Duration::seconds(2),
            next_item_started_at: start + chrono::Duration::seconds(2),
            reading: TempoReading::silent(),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.status, EntryStatus::Completed);
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(entry.plays[0].id, opened);
}

// --- PrepareReflection and play_would_survive_drop (#1758) -------------

fn play_by_id<'a>(entry: &'a SetlistEntry, id: &str) -> &'a VariationPlay {
    entry
        .plays
        .iter()
        .find(|p| p.id == id)
        .expect("the play is still in the entry")
}

#[test]
fn a_stray_tap_predicts_as_unmarkable_and_is_dropped() {
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();
    let opened = only_entry(&model).plays[0].id.clone();
    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id: entry_id.clone(),
            variation_id: Some("v-d".to_string()),
            now: start + chrono::Duration::seconds(298),
            reading: TempoReading::silent(),
        }),
    );
    let stray_tap = only_entry(&model).plays[1].id.clone();

    // The picker tap two seconds before the end: still the open play, so
    // its true duration is unknown until this stamps it.
    update(
        &mut model,
        Event::Session(SessionEvent::PrepareReflection {
            now: start + chrono::Duration::seconds(300),
            reading: TempoReading::silent(),
        }),
    );
    let entry = only_entry(&model);
    assert_eq!(
        play_by_id(entry, &stray_tap).seconds,
        2,
        "PrepareReflection stamped the real duration, not the open play's 0"
    );
    assert!(!play_would_survive_drop(
        entry,
        play_by_id(entry, &stray_tap)
    ));
    assert!(play_would_survive_drop(entry, play_by_id(entry, &opened)));

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start + chrono::Duration::seconds(300),
            next_item_started_at: start + chrono::Duration::seconds(300),
            reading: TempoReading::silent(),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(entry.plays[0].id, opened, "the prediction matched the drop");
}

#[test]
fn the_sole_play_predicts_as_markable_however_short() {
    let entry = SetlistEntry {
        plays: vec![VariationPlay {
            seconds: 2,
            ..VariationPlay::fixture()
        }],
        ..SetlistEntry::fixture()
    };

    assert!(play_would_survive_drop(&entry, &entry.plays[0]));
}

#[test]
fn when_every_play_is_incidental_only_the_first_predicts_as_markable() {
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();
    let opened = only_entry(&model).plays[0].id.clone();
    // A switch two seconds in, then PrepareReflection a second later:
    // both plays run under the five second threshold with no mark or
    // repetitions, so both are incidental and only the fallback survivor
    // should predict markable.
    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id,
            variation_id: Some("v-d".to_string()),
            now: start + chrono::Duration::seconds(2),
            reading: TempoReading::silent(),
        }),
    );
    let second = only_entry(&model).plays[1].id.clone();
    update(
        &mut model,
        Event::Session(SessionEvent::PrepareReflection {
            now: start + chrono::Duration::seconds(3),
            reading: TempoReading::silent(),
        }),
    );

    let entry = only_entry(&model);
    assert!(play_would_survive_drop(entry, play_by_id(entry, &opened)));
    assert!(!play_would_survive_drop(entry, play_by_id(entry, &second)));

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start + chrono::Duration::seconds(3),
            next_item_started_at: start + chrono::Duration::seconds(3),
            reading: TempoReading::silent(),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(entry.plays[0].id, opened, "the prediction matched the drop");
}

/// Mutation coverage for the fallback's `all(is_incidental)` conjunct: a
/// later play that genuinely ran must survive even though the first,
/// incidental play sits at `entry.plays[0]`, the fallback's own slot.
#[test]
fn a_later_real_play_predicts_as_markable_even_when_the_first_was_incidental() {
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();
    let opened = only_entry(&model).plays[0].id.clone();
    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id,
            variation_id: Some("v-d".to_string()),
            now: start + chrono::Duration::seconds(2),
            reading: TempoReading::silent(),
        }),
    );
    let real_play = only_entry(&model).plays[1].id.clone();
    update(
        &mut model,
        Event::Session(SessionEvent::PrepareReflection {
            now: start + chrono::Duration::seconds(302),
            reading: TempoReading::silent(),
        }),
    );

    let entry = only_entry(&model);
    assert!(!play_would_survive_drop(entry, play_by_id(entry, &opened)));
    assert!(play_would_survive_drop(
        entry,
        play_by_id(entry, &real_play)
    ));

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start + chrono::Duration::seconds(302),
            next_item_started_at: start + chrono::Duration::seconds(302),
            reading: TempoReading::silent(),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(
        entry.plays[0].id, real_play,
        "the prediction matched the drop"
    );
}

#[test]
fn prepare_reflection_does_not_advance_or_drop() {
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();
    // A stray tap, so drop_incidental_play would have something to drop
    // if PrepareReflection called it: proves the non-drop, not just an
    // entry too short to exercise it.
    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id: entry_id.clone(),
            variation_id: Some("v-d".to_string()),
            now: start + chrono::Duration::seconds(1),
            reading: TempoReading::silent(),
        }),
    );

    update(
        &mut model,
        Event::Session(SessionEvent::PrepareReflection {
            now: start + chrono::Duration::seconds(2),
            reading: TempoReading::silent(),
        }),
    );

    let SessionStatus::Active(active) = &model.session_status else {
        panic!("still active: PrepareReflection is not terminal");
    };
    assert_eq!(active.current_index, 0);
    let entry = session_entries(&model)
        .iter()
        .find(|e| e.id == entry_id)
        .expect("the entry is unchanged");
    assert_eq!(entry.status, EntryStatus::NotAttempted);
    assert_eq!(
        entry.plays.len(),
        2,
        "nothing was dropped early, though the open play is incidental"
    );
    assert_eq!(entry.plays[1].seconds, 1, "the open play's real duration");
}

#[test]
fn moving_off_an_item_keeps_its_only_play_however_short() {
    let (mut model, start) = model_with_active_session(2);
    let entry_id = session_entries(&model)[0].id.clone();
    let opened = session_entries(&model)[0].plays[0].id.clone();

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start + chrono::Duration::seconds(2),
            next_item_started_at: start + chrono::Duration::seconds(2),
            reading: TempoReading::silent(),
        }),
    );

    let entry = session_entries(&model)
        .iter()
        .find(|e| e.id == entry_id)
        .expect("the entry is still in the session");
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(entry.plays[0].id, opened);
}

#[test]
fn finishing_keeps_a_short_play_that_banked_a_repetition() {
    let (mut model, start) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();
    update(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id,
            variation_id: Some("v-d".to_string()),
            now: start + chrono::Duration::seconds(298),
            reading: TempoReading::silent(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::RepGotIt {
            now: start + chrono::Duration::seconds(299),
        }),
    );

    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: start + chrono::Duration::seconds(300),
            next_item_started_at: start + chrono::Duration::seconds(300),
            reading: TempoReading::silent(),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.plays.len(), 2);
    assert_eq!(entry.plays[1].rep_count, Some(1));
}

#[test]
fn a_skipped_entry_keeps_a_play_that_banked_repetitions() {
    let (mut model, start) = model_with_variations();
    update(
        &mut model,
        Event::Session(SessionEvent::RepGotIt {
            now: start + chrono::Duration::seconds(30),
        }),
    );

    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem {
            now: start + chrono::Duration::seconds(60),
        }),
    );

    let entry = only_entry(&model);
    assert_eq!(entry.status, EntryStatus::Skipped);
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(entry.plays[0].rep_count, Some(1));
}

#[test]
fn a_skipped_entry_that_recorded_nothing_keeps_no_play() {
    let (mut model, start) = model_with_variations();

    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem {
            now: start + chrono::Duration::seconds(60),
        }),
    );

    assert!(only_entry(&model).plays.is_empty());
}

#[test]
fn score_summary_is_the_mean_of_the_plays_that_carry_a_mark() {
    let mut entry = SetlistEntry::fixture();
    entry.plays = vec![
        VariationPlay {
            score: Some(8),
            ..VariationPlay::fixture()
        },
        VariationPlay {
            score: Some(5),
            ..VariationPlay::fixture()
        },
        VariationPlay {
            score: None,
            ..VariationPlay::fixture()
        },
    ];

    assert_eq!(entry.score_summary(), Some(7), "13 over 2 rounds to 7");
}

#[test]
fn score_summary_is_none_when_no_play_carries_a_mark() {
    let mut entry = SetlistEntry::fixture();
    entry.plays = vec![VariationPlay::fixture()];

    assert_eq!(entry.score_summary(), None);
}

#[test]
fn switch_variation_and_the_play_id_events_round_trip_on_the_bincode_wire() {
    crate::domain::types::assert_round_trips(crate::app::Event::Session(
        SessionEvent::SwitchVariation {
            entry_id: "e1".to_string(),
            variation_id: Some("v-d".to_string()),
            now: tap_at(),
            reading: TempoReading::silent(),
        },
    ));
    crate::domain::types::assert_round_trips(crate::app::Event::Session(
        SessionEvent::UpdateEntryScore {
            entry_id: "e1".to_string(),
            play_id: "e1-play".to_string(),
            score: Some(7),
        },
    ));
    crate::domain::types::assert_round_trips(crate::app::Event::Session(
        SessionEvent::UpdateEntryTempo {
            entry_id: "e1".to_string(),
            play_id: "e1-play".to_string(),
            tempo: Some(120),
            user_set: true,
            click: None,
        },
    ));
}

// ── Crash-recovery saves (#1997) ───────────────────────────────────

fn recovery_saves(cmd: &mut Command<Effect, Event>) -> Vec<ActiveSession> {
    cmd.effects()
        .filter_map(|e| match e {
            Effect::App(req) => match req.operation {
                AppEffect::SaveSessionInProgress(ref active) => Some(active.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn run(model: &mut Model, event: Event) -> Vec<ActiveSession> {
    let mut cmd = Intrada.update(event, model);
    recovery_saves(&mut cmd)
}

fn assert_saved_what_is_active(saves: &[ActiveSession], model: &Model) {
    let SessionStatus::Active(ref active) = model.session_status else {
        panic!("Expected Active state");
    };
    assert_eq!(saves, std::slice::from_ref(active));
}

#[test]
fn starting_a_session_saves_the_recovery_copy() {
    let mut model = model_with_library();
    update(&mut model, Event::Session(SessionEvent::StartBuilding));
    update(
        &mut model,
        Event::Session(SessionEvent::AddToSetlist {
            item_id: "piece-1".to_string(),
        }),
    );

    let saves = run(
        &mut model,
        Event::Session(SessionEvent::StartSession { now: Utc::now() }),
    );

    assert_saved_what_is_active(&saves, &model);
}

#[test]
fn moving_to_the_next_item_saves_the_recovery_copy() {
    let (mut model, start) = model_with_active_session(2);
    let now = start + chrono::Duration::seconds(30);
    let next_started = now + chrono::Duration::seconds(5);

    let saves = run(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now,
            next_item_started_at: next_started,
            reading: TempoReading::silent(),
        }),
    );

    assert_saved_what_is_active(&saves, &model);
    assert_eq!(saves[0].current_index, 1);
    assert_eq!(saves[0].current_item_started_at, next_started);
    assert_eq!(saves[0].entries[0].status, EntryStatus::Completed);
}

#[test]
fn skipping_an_item_saves_the_recovery_copy() {
    let (mut model, start) = model_with_active_session(2);
    let now = start + chrono::Duration::seconds(30);

    let saves = run(&mut model, Event::Session(SessionEvent::SkipItem { now }));

    assert_saved_what_is_active(&saves, &model);
    assert_eq!(saves[0].current_index, 1);
    assert_eq!(saves[0].current_item_started_at, now);
    assert_eq!(saves[0].entries[0].status, EntryStatus::Skipped);
}

#[test]
fn switching_variation_saves_the_recovery_copy() {
    let (mut model, now) = model_with_variations();
    let entry_id = only_entry(&model).id.clone();

    let saves = run(
        &mut model,
        Event::Session(SessionEvent::SwitchVariation {
            entry_id,
            variation_id: Some("v-d".to_string()),
            now: now + chrono::Duration::seconds(20),
            reading: TempoReading::silent(),
        }),
    );

    assert_saved_what_is_active(&saves, &model);
    let open = saves[0].entries[0].open_play().expect("a play is open");
    assert_eq!(open.variation_id.as_deref(), Some("v-d"));
}

#[test]
fn a_repetition_saves_the_recovery_copy() {
    let (mut model, now) = model_with_active_session_and_rep(5);

    let saves = run(
        &mut model,
        Event::Session(SessionEvent::RepGotIt {
            now: now + chrono::Duration::seconds(3),
        }),
    );

    assert_saved_what_is_active(&saves, &model);
    assert_eq!(play_of(&saves[0].entries[0]).rep_count, Some(1));
}

#[test]
fn finishing_a_session_saves_no_recovery_copy() {
    let finishes: [fn(DateTime<Utc>) -> Event; 3] = [
        |now| {
            Event::Session(SessionEvent::NextItem {
                now,
                next_item_started_at: now,
                reading: TempoReading::silent(),
            })
        },
        |now| Event::Session(SessionEvent::SkipItem { now }),
        |now| {
            Event::Session(SessionEvent::EndSessionEarly {
                now,
                reading: TempoReading::silent(),
            })
        },
    ];
    for finish in finishes {
        let (mut model, start) = model_with_active_session(1);
        let saves = run(&mut model, finish(start + chrono::Duration::seconds(30)));
        assert!(saves.is_empty());
        assert!(matches!(model.session_status, SessionStatus::Summary(_)));
    }
}

#[test]
fn skipping_the_last_item_drops_the_play_that_recorded_nothing() {
    let (mut model, start) = model_with_active_session(2);
    let t1 = start + chrono::Duration::seconds(30);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading: TempoReading::silent(),
        }),
    );

    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem {
            now: t1 + chrono::Duration::seconds(40),
        }),
    );

    let SessionStatus::Summary(ref summary) = model.session_status else {
        panic!("Expected Summary state");
    };
    let skipped = &summary.entries[1];
    assert_eq!(skipped.status, EntryStatus::Skipped);
    assert_eq!(skipped.duration_secs, 0);
    assert!(skipped.plays.is_empty());
    assert_eq!(summary.entries[0].status, EntryStatus::Completed);
}

#[test]
fn skipping_the_last_item_clears_the_tempo_of_a_play_that_survives() {
    let (mut model, start) = model_with_active_session_and_rep(5);
    let t1 = start + chrono::Duration::seconds(30);
    update(
        &mut model,
        Event::Session(SessionEvent::NextItem {
            now: t1,
            next_item_started_at: t1,
            reading: TempoReading::silent(),
        }),
    );
    update(
        &mut model,
        Event::Session(SessionEvent::RepGotIt { now: t1 }),
    );
    if let SessionStatus::Active(ref mut active) = model.session_status {
        let play = active.entries[1].open_play_mut().expect("a play is open");
        play.achieved_tempo = Some(96);
        play.click_pattern = Some(seven_eight_on_group_starts());
    }

    update(
        &mut model,
        Event::Session(SessionEvent::SkipItem {
            now: t1 + chrono::Duration::seconds(40),
        }),
    );

    let SessionStatus::Summary(ref summary) = model.session_status else {
        panic!("Expected Summary state");
    };
    let play = play_of(&summary.entries[1]);
    assert_eq!(play.rep_count, Some(1));
    assert_eq!(play.achieved_tempo, None);
    assert_eq!(play.click_pattern, None);
}

#[test]
fn planning_setters_are_refused_outside_building() {
    let setters: [fn(String) -> Event; 3] = [
        |entry_id| {
            Event::Session(SessionEvent::SetEntryIntention {
                entry_id,
                intention: Some("Slow hands".to_string()),
            })
        },
        |entry_id| {
            Event::Session(SessionEvent::SetRepTarget {
                entry_id,
                target: Some(10),
            })
        },
        |entry_id| {
            Event::Session(SessionEvent::SetEntryDuration {
                entry_id,
                duration_secs: Some(600),
            })
        },
    ];
    for setter in setters {
        let (mut model, _) = model_with_active_session(1);
        let before = active_entry(&model, 0).clone();

        update(&mut model, setter(before.id.clone()));

        assert!(model.last_error.is_some());
        assert_eq!(active_entry(&model, 0), &before);
    }
}

// --- A history load never overwrites a newer save (#2067) ---

#[test]
fn a_history_load_out_when_a_save_is_acknowledged_is_dropped_and_asked_again() {
    let mut model = model_with_summary();
    update(&mut model, Event::StartApp);
    update(
        &mut model,
        Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
    );
    update(
        &mut model,
        Event::SessionStoreWritten(crate::persistence::PersistenceOutput::Ack),
    );
    assert_eq!(model.sessions.len(), 1);

    let mut landed = Intrada.update(
        Event::SessionsStoreLoaded(crate::persistence::PersistenceOutput::Sessions(vec![])),
        &mut model,
    );
    assert_eq!(
        model.sessions.len(),
        1,
        "the older history must not drop the saved practice"
    );
    assert!(landed
        .effects()
        .any(|e| matches!(e, Effect::Persistence(req)
        if req.operation == crate::persistence::PersistenceOperation::LoadSessions)));
}

#[test]
fn a_history_load_that_brings_back_no_list_is_no_longer_out() {
    use crate::persistence::{PersistenceOperation, PersistenceOutput};
    for (arm, no_list) in [
        ("failed", PersistenceOutput::Failed),
        ("stray ack", PersistenceOutput::Ack),
    ] {
        let mut model = model_with_summary();
        update(&mut model, Event::StartApp);
        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
        );
        update(&mut model, Event::SessionsStoreLoaded(no_list));

        let mut acked = Intrada.update(
            Event::SessionStoreWritten(PersistenceOutput::Ack),
            &mut model,
        );
        assert!(
            acked.effects().any(|e| matches!(e, Effect::Persistence(req)
                if req.operation == PersistenceOperation::LoadSessions)),
            "{arm}"
        );
    }
}
