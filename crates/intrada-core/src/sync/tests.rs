use chrono::{DateTime, Utc};
use crux_core::{App, Command};

use super::*;
use crate::app::{Effect, Event, Intrada};
use crate::model::Model;
use crate::persistence::{
    self, PersistenceOperation, PersistenceOutput, StoredItem, StoredRecords,
};

fn at(secs: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(1_790_000_000 + secs, 0).expect("fixed time")
}

fn record(changed: i64, deleted: bool, body: &str) -> SyncRecord {
    SyncRecord {
        kind: RecordKind::Item,
        id: "piece".to_string(),
        schema_version: SCHEMA_VERSION,
        changed_at: at(changed),
        deleted_at: deleted.then(|| at(changed)),
        body: body.as_bytes().to_vec(),
    }
}

fn newer_app(r: SyncRecord) -> SyncRecord {
    SyncRecord {
        schema_version: SCHEMA_VERSION + 1,
        ..r
    }
}

// ── The merge ──

#[test]
fn two_device_cases_pick_the_copy_the_spec_names() {
    let cases: [(&str, Option<SyncRecord>, SyncRecord, Verdict); 11] = [
        ("nothing here", None, record(10, false, "a"), Verdict::Take),
        (
            "both edited offline, theirs later",
            Some(record(10, false, "a")),
            record(20, false, "b"),
            Verdict::Take,
        ),
        (
            "both edited offline, ours later",
            Some(record(20, false, "a")),
            record(10, false, "b"),
            Verdict::Keep,
        ),
        (
            "a delete against a stale edit",
            Some(record(30, true, "a")),
            record(20, false, "b"),
            Verdict::Keep,
        ),
        (
            "a stale edit arriving at a delete",
            Some(record(20, false, "a")),
            record(30, true, "a"),
            Verdict::Take,
        ),
        (
            "a newer edit against a stale delete",
            Some(record(30, true, "a")),
            record(40, false, "b"),
            Verdict::Take,
        ),
        (
            "exact tie, theirs sorts lower",
            Some(record(20, false, "b")),
            record(20, false, "a"),
            Verdict::Take,
        ),
        (
            "exact tie, ours sorts lower",
            Some(record(20, false, "a")),
            record(20, false, "b"),
            Verdict::Keep,
        ),
        (
            "a delete ties an edit",
            Some(record(20, false, "a")),
            record(20, true, "z"),
            Verdict::Take,
        ),
        (
            "the same record twice",
            Some(record(20, false, "a")),
            record(20, false, "a"),
            Verdict::Keep,
        ),
        (
            "from a newer app",
            Some(record(10, false, "a")),
            newer_app(record(5, false, "b")),
            Verdict::Park,
        ),
    ];
    for (case, local, arrived, expected) in cases {
        assert_eq!(decide(local.as_ref(), &arrived), expected, "{case}");
    }
}

#[test]
fn a_record_from_a_newer_app_parks_even_with_nothing_here() {
    assert_eq!(
        decide(None, &newer_app(record(10, false, "a"))),
        Verdict::Park
    );
}

/// Deleting the body tie-break leaves each device keeping its own copy.
#[test]
fn two_devices_merging_the_same_records_end_the_same() {
    let records = [
        record(10, false, "a"),
        record(20, false, "a"),
        record(20, false, "b"),
        record(20, true, "a"),
        record(20, true, "c"),
        record(30, true, "a"),
    ];
    let merged = |local: &SyncRecord, arrived: &SyncRecord| match decide(Some(local), arrived) {
        Verdict::Take => arrived.clone(),
        Verdict::Keep | Verdict::Park => local.clone(),
    };
    for x in &records {
        for y in &records {
            assert_eq!(merged(x, y), merged(y, x), "{x:?} against {y:?}");
        }
    }
}

// ── The record ──

#[test]
fn a_deleted_piece_changes_when_it_was_deleted() {
    let item = Item::fixture("piece");
    let record = Synced::Item {
        item: Box::new(item.clone()),
        deleted_at: Some(at(60)),
    }
    .record()
    .expect("encodes");
    assert_eq!(record.changed_at, at(60));
    assert_eq!(record.deleted_at, Some(at(60)));

    let live = Synced::Item {
        item: Box::new(item),
        deleted_at: None,
    }
    .record()
    .expect("encodes");
    assert_eq!(live.changed_at, at(0));
}

#[test]
fn every_kind_decodes_to_what_was_encoded() {
    let variation = Variation {
        id: "v".to_string(),
        label: "Dotted rhythms".to_string(),
        updated_at: at(5),
        deleted_at: Some(at(9)),
    };
    for synced in [
        Synced::Item {
            item: Box::new(Item::fixture("piece")),
            deleted_at: Some(at(3)),
        },
        Synced::Variation(variation),
        Synced::Session(PracticeSession::fixture("s")),
    ] {
        let record = synced.record().expect("encodes");
        assert_eq!(record.key(), synced.key());
        assert_eq!(Synced::decode(&record).expect("decodes"), synced);
    }
}

const PINNED_ITEM: &str = r#"{"id":"piece","title":"Etude","kind":"piece","composer":null,"key":null,"tempo":null,"notes":null,"tags":[],"created_at":"2026-09-21T14:13:20Z","updated_at":"2026-09-21T14:13:20Z","priority":false,"chord_chart":null,"photo_id":null,"metre":null,"sections":[],"variation_ids":[],"keys":[],"exercise_links":[]}"#;
const PINNED_VARIATION: &str =
    r#"{"id":"v","label":"Dotted rhythms","updated_at":"2026-09-21T14:13:20Z","deleted_at":null}"#;
const PINNED_SESSION: &str = r#"{"id":"s","entries":[],"session_notes":null,"started_at":"2026-09-21T14:13:20Z","completed_at":"2026-09-21T14:13:20Z","total_duration_secs":0,"completion_status":"Completed","session_score":null,"capture_version":1}"#;

/// Another device reads these bytes, maybe on an older app.
#[test]
fn record_bodies_are_pinned() {
    let variation = Variation {
        id: "v".to_string(),
        label: "Dotted rhythms".to_string(),
        updated_at: at(0),
        deleted_at: None,
    };
    for (synced, pinned) in [
        (
            Synced::Item {
                item: Box::new(Item::fixture("piece")),
                deleted_at: None,
            },
            PINNED_ITEM,
        ),
        (Synced::Variation(variation), PINNED_VARIATION),
        (
            Synced::Session(PracticeSession::fixture("s")),
            PINNED_SESSION,
        ),
    ] {
        let body = synced.record().expect("encodes").body;
        assert_eq!(
            String::from_utf8(body).expect("JSON is text"),
            pinned,
            "a synced type changed shape: bump sync::SCHEMA_VERSION, then re-pin"
        );
    }
}

#[test]
fn a_piece_from_before_its_later_fields_still_decodes() {
    let older = r#"{"id":"piece","title":"Etude","kind":"piece","composer":null,"key":null,"tempo":null,"notes":null,"tags":[],"created_at":"2026-09-21T14:13:20Z","updated_at":"2026-09-21T14:13:20Z"}"#;
    let record = SyncRecord {
        body: older.as_bytes().to_vec(),
        ..record(0, false, "")
    };
    let Synced::Item { item, .. } = Synced::decode(&record).expect("decodes") else {
        panic!("an item record decodes to an item");
    };
    assert_eq!(*item, Item::fixture("piece"));
}

#[test]
fn the_bridge_shapes_round_trip_on_the_ffi_wire() {
    use crate::domain::types::assert_round_trips;
    let r = record(0, true, "{}");
    assert_round_trips(SyncOperation::Upload(vec![r.clone()]));
    assert_round_trips(SyncOperation::Park(vec![r.clone()]));
    assert_round_trips(SyncOperation::Unpark(vec![r.key()]));
    assert_round_trips(Event::Sync(SyncEvent::RecordsArrived(vec![r])));
    assert_round_trips(Event::Sync(SyncEvent::UploadEverything));
    assert_round_trips(Event::Sync(SyncEvent::AccountChanged));
    assert_round_trips(PersistenceOperation::LoadRecords(vec![RecordKey {
        kind: RecordKind::Session,
        id: "s".to_string(),
    }]));
    assert_round_trips(PersistenceOperation::ApplyMerged(StoredRecords {
        items: vec![StoredItem {
            item: Item::fixture("piece"),
            deleted_at: Some(at(1)),
        }],
        variations: vec![],
        sessions: vec![PracticeSession::fixture("s")],
    }));
    assert_round_trips(PersistenceOutput::Records(StoredRecords::default()));
}

// ── Local writes upload ──

fn persistence_request(
    cmd: &mut Command<Effect, Event>,
) -> crux_core::Request<PersistenceOperation> {
    cmd.effects()
        .find_map(|e| match e {
            Effect::Persistence(req) => Some(req),
            _ => None,
        })
        .expect("a persistence request")
}

fn uploads(cmd: &mut Command<Effect, Event>) -> Vec<SyncRecord> {
    cmd.effects()
        .filter_map(|e| match e {
            Effect::Sync(req) => match req.operation {
                SyncOperation::Upload(records) => Some(records),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn a_kept_save_uploads_and_a_refused_one_does_not() {
    for (output, expected) in [(PersistenceOutput::Ack, 1), (PersistenceOutput::Failed, 0)] {
        let mut cmd = persistence::save_item(&mut Model::default(), Item::fixture("piece"));
        persistence_request(&mut cmd)
            .resolve(output.clone())
            .expect("resolves once");
        assert_eq!(uploads(&mut cmd).len(), expected, "{output:?}");
        assert!(matches!(cmd.events().next(), Some(Event::StoreWritten(o)) if o == output));
    }
}

#[test]
fn a_delete_uploads_the_tombstone() {
    let deleted_at = at(30);
    let mut cmd =
        persistence::delete_item(&mut Model::default(), Item::fixture("piece"), deleted_at);
    persistence_request(&mut cmd)
        .resolve(PersistenceOutput::Ack)
        .expect("resolves once");
    let sent = uploads(&mut cmd);
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].deleted_at, Some(deleted_at));
    assert_eq!(sent[0].changed_at, deleted_at);
}

#[test]
fn a_saved_session_and_a_variation_upload_too() {
    let mut model = Model::default();
    let mut cmd = persistence::save_session(&mut model, PracticeSession::fixture("s"));
    persistence_request(&mut cmd)
        .resolve(PersistenceOutput::Ack)
        .expect("resolves once");
    assert_eq!(uploads(&mut cmd)[0].kind, RecordKind::Session);

    let row = Variation {
        id: "v".to_string(),
        label: "Slow".to_string(),
        updated_at: at(0),
        deleted_at: None,
    };
    let mut cmd = persistence::save_variations(&mut model, vec![row]);
    persistence_request(&mut cmd)
        .resolve(PersistenceOutput::Ack)
        .expect("resolves once");
    assert_eq!(uploads(&mut cmd)[0].kind, RecordKind::Variation);
}

#[test]
fn a_parked_id_or_a_paused_account_uploads_nothing() {
    let piece = Synced::Item {
        item: Box::new(Item::fixture("piece")),
        deleted_at: None,
    };
    let mut model = Model::default();
    assert_eq!(upload_for(&model, std::slice::from_ref(&piece)).len(), 1);

    model.sync.parked.insert(piece.key());
    assert!(upload_for(&model, std::slice::from_ref(&piece)).is_empty());

    let mut model = Model::default();
    let _ = Intrada.update(Event::Sync(SyncEvent::AccountChanged), &mut model);
    assert!(upload_for(&model, &[piece]).is_empty());
}

// ── Records arrive ──

fn sync_effects(cmd: &mut Command<Effect, Event>) -> Vec<SyncOperation> {
    cmd.effects()
        .filter_map(|e| match e {
            Effect::Sync(req) => Some(req.operation),
            _ => None,
        })
        .collect()
}

fn arrived_piece(changed: i64, title: &str) -> SyncRecord {
    Synced::Item {
        item: Box::new(Item {
            title: title.to_string(),
            updated_at: at(changed),
            ..Item::fixture("piece")
        }),
        deleted_at: None,
    }
    .record()
    .expect("encodes")
}

/// Runs an arrived batch through the store, answering the load with `stored`,
/// and returns the merge the core asked the store to write.
fn arrive(
    model: &mut Model,
    records: Vec<SyncRecord>,
    stored: StoredRecords,
) -> Option<StoredRecords> {
    let mut cmd = Intrada.update(Event::Sync(SyncEvent::RecordsArrived(records)), model);
    let mut load = persistence_request(&mut cmd);
    assert!(matches!(
        load.operation,
        PersistenceOperation::LoadRecords(_)
    ));
    load.resolve(PersistenceOutput::Records(stored))
        .expect("resolves once");
    let loaded = cmd.events().next().expect("the stored copies come back");
    let mut cmd = Intrada.update(loaded, model);
    let written = cmd.effects().find_map(|e| match e {
        Effect::Persistence(req) => match req.operation {
            PersistenceOperation::ApplyMerged(records) => Some(records),
            _ => None,
        },
        _ => None,
    });
    written
}

#[test]
fn a_later_piece_from_the_other_device_is_written() {
    let mut model = Model::default();
    let stored = StoredRecords {
        items: vec![StoredItem {
            item: Item::fixture("piece"),
            deleted_at: None,
        }],
        ..Default::default()
    };
    let written = arrive(&mut model, vec![arrived_piece(60, "Nocturne")], stored)
        .expect("the later copy is written");
    assert_eq!(written.items[0].item.title, "Nocturne");
}

#[test]
fn a_stale_edit_never_brings_back_a_deleted_piece() {
    let mut model = Model::default();
    let stored = StoredRecords {
        items: vec![StoredItem {
            item: Item::fixture("piece"),
            deleted_at: Some(at(600)),
        }],
        ..Default::default()
    };
    assert_eq!(
        arrive(&mut model, vec![arrived_piece(60, "Nocturne")], stored),
        None
    );
}

#[test]
fn of_two_copies_in_one_batch_the_later_is_written() {
    let mut model = Model::default();
    let written = arrive(
        &mut model,
        vec![arrived_piece(60, "Later"), arrived_piece(30, "Earlier")],
        StoredRecords::default(),
    )
    .expect("written");
    assert_eq!(written.items.len(), 1);
    assert_eq!(written.items[0].item.title, "Later");
}

#[test]
fn a_record_from_a_newer_app_is_parked_and_never_loaded() {
    let mut model = Model::default();
    let too_new = newer_app(arrived_piece(60, "Nocturne"));
    let mut cmd = Intrada.update(
        Event::Sync(SyncEvent::RecordsArrived(vec![too_new.clone()])),
        &mut model,
    );
    assert_eq!(
        sync_effects(&mut cmd),
        vec![SyncOperation::Park(vec![too_new.clone()])]
    );
    assert!(model.sync.parked.contains(&too_new.key()));
}

#[test]
fn a_body_that_will_not_decode_is_parked() {
    let mut model = Model::default();
    let broken = SyncRecord {
        body: b"not json".to_vec(),
        ..arrived_piece(60, "x")
    };
    assert_eq!(
        arrive(&mut model, vec![broken.clone()], StoredRecords::default()),
        None
    );
    assert!(model.sync.parked.contains(&broken.key()));
}

#[test]
fn a_written_merge_reloads_the_list_and_unparks_what_it_took() {
    let mut model = Model::default();
    let key = arrived_piece(60, "Nocturne").key();
    model.sync.parked.insert(key.clone());
    let written = arrive(
        &mut model,
        vec![arrived_piece(60, "Nocturne")],
        StoredRecords::default(),
    )
    .expect("written");

    let mut cmd = Intrada.update(
        Event::Sync(SyncEvent::MergeWritten {
            touched: vec![RecordKind::Item],
            unpark: vec![key.clone()],
            output: PersistenceOutput::Ack,
        }),
        &mut model,
    );
    assert_eq!(written.items.len(), 1);
    assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
        if req.operation == PersistenceOperation::LoadItems)));
    assert!(!model.sync.parked.contains(&key));
}

#[test]
fn a_refused_merge_shows_the_storage_error_and_keeps_parked_records() {
    let mut model = Model::default();
    let key = arrived_piece(60, "Nocturne").key();
    model.sync.parked.insert(key.clone());
    let _ = Intrada.update(
        Event::Sync(SyncEvent::MergeWritten {
            touched: vec![RecordKind::Item],
            unpark: vec![key.clone()],
            output: PersistenceOutput::Failed,
        }),
        &mut model,
    );
    assert!(model.last_error.is_some());
    assert!(model.sync.parked.contains(&key));
}

#[test]
fn upload_everything_sends_every_record_tombstones_included() {
    let mut model = Model::default();
    let mut cmd = Intrada.update(Event::Sync(SyncEvent::UploadEverything), &mut model);
    let mut load = persistence_request(&mut cmd);
    assert_eq!(load.operation, PersistenceOperation::LoadAllRecords);
    load.resolve(PersistenceOutput::Records(StoredRecords {
        items: vec![StoredItem {
            item: Item::fixture("gone"),
            deleted_at: Some(at(5)),
        }],
        variations: vec![],
        sessions: vec![PracticeSession::fixture("s")],
    }))
    .expect("resolves once");
    let loaded = cmd.events().next().expect("loaded");
    let mut cmd = Intrada.update(loaded, &mut model);
    let sent = uploads(&mut cmd);
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0].deleted_at, Some(at(5)));
}
