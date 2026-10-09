use chrono::{DateTime, Utc};
use crux_core::{App, Command};

use super::*;
use crate::app::{Effect, Event, Intrada};
use crate::domain::chart::{Bar, ChartChord, ChartSection, ChordChart, ChordQuality, ChordSymbol};
use crate::domain::item::Modality;
use crate::domain::key::{Accidental, Key, Letter};
use crate::domain::link::ExerciseLink;
use crate::domain::metre::Metre;
use crate::domain::section::{BarRange, ItemSection, SectionKind};
use crate::domain::session::{
    Away, ClickState, EntryStatus, Felt, FocusKind, IntentionFocus, IntentionMet, NotePoint,
    NotePointKind, NoteSpan, Obstacle, Play, RepAction, RepEvent, Segment, SetlistEntry,
    TempoChange,
};
use crate::domain::types::Tempo;
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
            "a delete arriving at a stale edit",
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

fn key(letter: Letter, accidental: Accidental, mode: Option<Modality>) -> Key {
    Key {
        letter,
        accidental,
        mode,
    }
}

/// Every optional field set and every list holding one, so the pin sees the
/// shape of each nested type.
fn full_item() -> Item {
    Item {
        composer: Some("Chopin".to_string()),
        key: Some(key(Letter::E, Accidental::Flat, Some(Modality::Major))),
        tempo: Some(Tempo {
            marking: Some("Andante".to_string()),
            bpm: Some(66),
        }),
        notes: Some("voicing".to_string()),
        tags: vec!["recital".to_string()],
        priority: true,
        chord_chart: Some(ChordChart {
            key: Some(key(Letter::G, Accidental::Natural, Some(Modality::Minor))),
            sections: vec![ChartSection {
                label: Some("A".to_string()),
                bars: vec![Bar {
                    chords: vec![ChartChord {
                        symbol: ChordSymbol {
                            root: 0,
                            quality: ChordQuality::Min7,
                            extensions: vec!["9".to_string()],
                            bass: Some(7),
                            raw: "Cm9/G".to_string(),
                        },
                    }],
                }],
            }],
        }),
        photo_id: Some("photo".to_string()),
        metre: Some(Metre {
            beats: 7,
            unit: 8,
            groups: Some(vec![2, 2, 3]),
        }),
        sections: vec![ItemSection {
            id: "s1".to_string(),
            name: "Coda".to_string(),
            bars: Some(BarRange { first: 1, last: 8 }),
            kind: SectionKind::TroubleSpot,
            target_bpm: Some(52),
            position: 0,
            updated_at: at(0),
            deleted_at: Some(at(5)),
        }],
        variation_ids: vec!["v".to_string()],
        keys: vec![key(Letter::C, Accidental::Sharp, Some(Modality::Minor))],
        exercise_links: vec![ExerciseLink {
            id: "l1".to_string(),
            exercise_id: "e1".to_string(),
            section_id: Some("s1".to_string()),
            position: 0,
            updated_at: at(0),
            deleted_at: Some(at(5)),
        }],
        ..Item::fixture("piece")
    }
}

fn full_session() -> PracticeSession {
    let metre = Metre {
        beats: 3,
        unit: 4,
        groups: Some(vec![3]),
    };
    let play = Play {
        section_id: Some("s1".to_string()),
        key: Some(key(Letter::D, Accidental::Natural, Some(Modality::Major))),
        variation_ids: vec!["v".to_string()],
        rep_target: Some(5),
        rep_count: Some(3),
        rep_history: Some(vec![RepEvent {
            action: RepAction::Success,
            at: at(10),
            tempo: Some(66),
            click_sounding: Some(true),
        }]),
        tempo_changes: vec![TempoChange {
            at: at(10),
            tempo: 66,
            click_sounding: true,
        }],
        achieved_tempo: Some(66),
        click_pattern: Some(ClickState { metre, sounding: 1 }),
        score: Some(4),
        away: vec![Away {
            left_at: at(20),
            back_at: Some(at(30)),
            left_out: true,
        }],
        ..Play::fixture()
    };
    let entry = SetlistEntry {
        status: EntryStatus::Completed,
        duration_secs: 300,
        notes: Some("steady".to_string()),
        intention: Some("even semiquavers".to_string()),
        planned_duration_secs: Some(600),
        group_id: Some("g".to_string()),
        planned_variation_ids: vec!["v".to_string()],
        planned_rep_target: Some(5),
        plays: vec![play],
        segments: vec![Segment {
            section_id: "s1".to_string(),
            planned_secs: 300,
        }],
        focus: Some(IntentionFocus {
            kind: FocusKind::Tempo,
            section_id: Some("s1".to_string()),
            target: Some(80),
        }),
        intention_met: Some(IntentionMet::Partly),
        felt: Some(Felt::HardWork),
        got_in_the_way: vec![Obstacle::Rhythm],
        note_points: vec![NotePoint {
            kind: NotePointKind::Tempo { bpm: 66 },
            section_id: Some("s1".to_string()),
            span: NoteSpan { start: 0, end: 5 },
        }],
        planned_key: Some(key(Letter::D, Accidental::Natural, Some(Modality::Major))),
        ..SetlistEntry::fixture()
    };
    PracticeSession {
        entries: vec![entry],
        session_notes: Some("good day".to_string()),
        total_duration_secs: 300,
        session_score: Some(4),
        ..PracticeSession::fixture("s")
    }
}

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
            item: Box::new(full_item()),
            deleted_at: Some(at(3)),
        },
        Synced::Variation(variation),
        Synced::Session(full_session()),
    ] {
        let record = synced.record().expect("encodes");
        assert_eq!(record.key(), synced.key());
        assert_eq!(Synced::decode(&record).expect("decodes"), synced);
    }
}

const PINNED_ITEM: &str = r#"{"id":"piece","title":"Etude","kind":"piece","composer":"Chopin","key":{"letter":"E","accidental":"Flat","mode":"major"},"tempo":{"marking":"Andante","bpm":66},"notes":"voicing","tags":["recital"],"created_at":"2026-09-21T14:13:20Z","updated_at":"2026-09-21T14:13:20Z","priority":true,"chord_chart":{"key":{"letter":"G","accidental":"Natural","mode":"minor"},"sections":[{"label":"A","bars":[{"chords":[{"symbol":{"root":0,"quality":"Min7","extensions":["9"],"bass":7,"raw":"Cm9/G"}}]}]}]},"photo_id":"photo","metre":{"beats":7,"unit":8,"groups":[2,2,3]},"sections":[{"id":"s1","name":"Coda","bars":{"first":1,"last":8},"kind":"TroubleSpot","target_bpm":52,"position":0,"updated_at":"2026-09-21T14:13:20Z","deleted_at":"2026-09-21T14:13:25Z"}],"variation_ids":["v"],"keys":[{"letter":"C","accidental":"Sharp","mode":"minor"}],"exercise_links":[{"id":"l1","exercise_id":"e1","section_id":"s1","position":0,"updated_at":"2026-09-21T14:13:20Z","deleted_at":"2026-09-21T14:13:25Z"}]}"#;
const PINNED_VARIATION: &str = r#"{"id":"v","label":"Dotted rhythms","updated_at":"2026-09-21T14:13:20Z","deleted_at":"2026-09-21T14:13:25Z"}"#;
const PINNED_SESSION: &str = r#"{"id":"s","entries":[{"id":"entry-1","item_id":"item-1","item_title":"Item","item_type":"piece","position":0,"duration_secs":300,"status":"Completed","notes":"steady","intention":"even semiquavers","planned_duration_secs":600,"group_id":"g","planned_variation_ids":["v"],"planned_rep_target":5,"plays":[{"id":"play-1","section_id":"s1","key":{"letter":"D","accidental":"Natural","mode":"major"},"variation_ids":["v"],"started_at":"1970-01-01T00:00:00Z","seconds":60,"rep_target":5,"rep_count":3,"rep_history":[{"action":"Success","at":"2026-09-21T14:13:30Z","tempo":66,"click_sounding":true}],"tempo_changes":[{"at":"2026-09-21T14:13:30Z","tempo":66,"click_sounding":true}],"achieved_tempo":66,"click_pattern":{"metre":{"beats":3,"unit":4,"groups":[3]},"sounding":1},"score":4,"away":[{"left_at":"2026-09-21T14:13:40Z","back_at":"2026-09-21T14:13:50Z","left_out":true}]}],"segments":[{"section_id":"s1","planned_secs":300}],"focus":{"kind":"Tempo","section_id":"s1","target":80},"intention_met":"Partly","felt":"HardWork","got_in_the_way":["Rhythm"],"note_points":[{"kind":{"Tempo":{"bpm":66}},"section_id":"s1","span":{"start":0,"end":5}}],"planned_key":{"letter":"D","accidental":"Natural","mode":"major"}}],"session_notes":"good day","started_at":"2026-09-21T14:13:20Z","completed_at":"2026-09-21T14:13:20Z","total_duration_secs":300,"completion_status":"Completed","session_score":4,"capture_version":1}"#;

/// Another device reads these bytes, maybe on an older app.
#[test]
fn record_bodies_are_pinned() {
    let variation = Variation {
        id: "v".to_string(),
        label: "Dotted rhythms".to_string(),
        updated_at: at(0),
        deleted_at: Some(at(5)),
    };
    for (synced, pinned) in [
        (
            Synced::Item {
                item: Box::new(full_item()),
                deleted_at: None,
            },
            PINNED_ITEM,
        ),
        (Synced::Variation(variation), PINNED_VARIATION),
        (Synced::Session(full_session()), PINNED_SESSION),
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
    assert_round_trips(Event::Sync(SyncEvent::RecordsArrived(vec![r.clone()])));
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
        unreadable: vec![],
    }));
    assert_round_trips(PersistenceOutput::Records(StoredRecords {
        unreadable: vec![r.key()],
        ..Default::default()
    }));
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

    model.sync.parked.insert(piece.key(), at(0));
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
    assert!(model.sync.parked.contains_key(&too_new.key()));
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
    assert!(model.sync.parked.contains_key(&broken.key()));
}

/// What one merge sent, start to finish: every sync effect, and what went to
/// the store once the merge write was answered.
#[derive(Default)]
struct Run {
    sync: Vec<SyncOperation>,
    after_write: Vec<PersistenceOperation>,
}

/// Runs an arrived batch through the store and the merge write the core
/// itself asked for, answering the load with `stored` and the write with
/// `written`.
fn merge_through(
    model: &mut Model,
    records: Vec<SyncRecord>,
    stored: StoredRecords,
    written: PersistenceOutput,
) -> Run {
    let mut run = Run::default();
    let mut cmd = Intrada.update(Event::Sync(SyncEvent::RecordsArrived(records)), model);
    let mut load = None;
    for effect in cmd.effects() {
        match effect {
            Effect::Sync(req) => run.sync.push(req.operation),
            Effect::Persistence(req) => load = Some(req),
            _ => {}
        }
    }
    let Some(mut load) = load else { return run };
    load.resolve(PersistenceOutput::Records(stored))
        .expect("resolves once");
    let loaded = cmd.events().next().expect("the stored copies come back");
    let mut cmd = Intrada.update(loaded, model);
    let mut write = None;
    for effect in cmd.effects() {
        match effect {
            Effect::Sync(req) => run.sync.push(req.operation),
            Effect::Persistence(req) => write = Some(req),
            _ => {}
        }
    }
    let Some(mut write) = write else { return run };
    assert!(matches!(
        write.operation,
        PersistenceOperation::ApplyMerged(_)
    ));
    write.resolve(written).expect("resolves once");
    let landed = cmd.events().next().expect("the merge write lands");
    let mut cmd = Intrada.update(landed, model);
    for effect in cmd.effects() {
        match effect {
            Effect::Sync(req) => run.sync.push(req.operation),
            Effect::Persistence(req) => run.after_write.push(req.operation),
            _ => {}
        }
    }
    run
}

fn stored_piece(changed: i64) -> StoredRecords {
    StoredRecords {
        items: vec![StoredItem {
            item: Item {
                updated_at: at(changed),
                ..Item::fixture("piece")
            },
            deleted_at: None,
        }],
        ..Default::default()
    }
}

#[test]
fn a_written_merge_reloads_the_list_and_unparks_what_it_took() {
    let mut model = Model::default();
    let key = arrived_piece(60, "Nocturne").key();
    model.sync.parked.insert(key.clone(), at(60));
    let run = merge_through(
        &mut model,
        vec![arrived_piece(60, "Nocturne")],
        StoredRecords::default(),
        PersistenceOutput::Ack,
    );
    assert!(run.after_write.contains(&PersistenceOperation::LoadItems));
    assert_eq!(run.sync, vec![SyncOperation::Unpark(vec![key.clone()])]);
    assert!(!model.sync.parked.contains_key(&key));
}

#[test]
fn a_refused_merge_shows_the_storage_error_and_keeps_parked_records() {
    let mut model = Model::default();
    let key = arrived_piece(60, "Nocturne").key();
    model.sync.parked.insert(key.clone(), at(60));
    let run = merge_through(
        &mut model,
        vec![arrived_piece(60, "Nocturne")],
        StoredRecords::default(),
        PersistenceOutput::Failed,
    );
    assert!(model.last_error.is_some());
    assert!(run.sync.is_empty());
    assert!(model.sync.parked.contains_key(&key));
}

#[test]
fn a_parked_id_whose_local_copy_wins_is_unparked_and_uploaded() {
    let mut model = Model::default();
    let key = arrived_piece(40, "Theirs").key();
    model.sync.parked.insert(key.clone(), at(30));
    let stored = stored_piece(60);
    let local = Synced::Item {
        item: Box::new(stored.items[0].item.clone()),
        deleted_at: None,
    }
    .record()
    .expect("encodes");

    let run = merge_through(
        &mut model,
        vec![arrived_piece(40, "Theirs")],
        stored,
        PersistenceOutput::Ack,
    );

    assert_eq!(
        run.sync,
        vec![
            SyncOperation::Unpark(vec![key.clone()]),
            SyncOperation::Upload(vec![local]),
        ]
    );
    assert!(!model.sync.parked.contains_key(&key));
}

#[test]
fn only_an_arrival_as_late_as_the_parked_copy_unparks_it() {
    let cases = [
        ("older arrival, taken", 50, 40, 10, true),
        ("older arrival, kept", 50, 40, 60, true),
        ("as late, taken", 40, 40, 10, false),
        ("later, kept", 30, 40, 60, false),
    ];
    for (case, parked_at, arrives, local, stays_parked) in cases {
        let mut model = Model::default();
        let key = arrived_piece(arrives, "Theirs").key();
        model.sync.parked.insert(key.clone(), at(parked_at));
        let run = merge_through(
            &mut model,
            vec![arrived_piece(arrives, "Theirs")],
            stored_piece(local),
            PersistenceOutput::Ack,
        );
        assert_eq!(model.sync.parked.contains_key(&key), stays_parked, "{case}");
        assert_eq!(
            run.sync
                .iter()
                .any(|op| matches!(op, SyncOperation::Unpark(_))),
            !stays_parked,
            "{case}"
        );
    }
}

#[test]
fn a_key_parked_twice_remembers_the_later_copy() {
    let mut model = Model::default();
    for changed in [30, 50, 20] {
        let _ = Intrada.update(
            Event::Sync(SyncEvent::RecordsArrived(vec![newer_app(arrived_piece(
                changed, "Theirs",
            ))])),
            &mut model,
        );
    }
    let key = arrived_piece(0, "x").key();
    assert_eq!(model.sync.parked.get(&key), Some(&at(50)));
}

#[test]
fn a_load_out_before_a_merge_never_lands_over_it() {
    let mut model = Model::default();
    let _load = persistence::load_items(&mut model);
    arrive(
        &mut model,
        vec![arrived_piece(60, "Nocturne")],
        StoredRecords::default(),
    )
    .expect("written");

    let _ = Intrada.update(
        Event::StoreLoaded(PersistenceOutput::Items(vec![Item::fixture("before")])),
        &mut model,
    );
    assert!(model.items.is_empty());
}

#[test]
fn an_unreadable_row_here_parks_the_arrival_instead_of_writing_over_it() {
    let mut model = Model::default();
    let arrival = arrived_piece(60, "Nocturne");
    let stored = StoredRecords {
        unreadable: vec![arrival.key()],
        ..Default::default()
    };
    let run = merge_through(
        &mut model,
        vec![arrival.clone()],
        stored,
        PersistenceOutput::Ack,
    );
    assert_eq!(run.sync, vec![SyncOperation::Park(vec![arrival.clone()])]);
    assert!(run.after_write.is_empty());
    assert!(model.sync.parked.contains_key(&arrival.key()));
}

// ── Merges and local writes in flight ──

fn load_request(
    cmd: &mut Command<Effect, Event>,
) -> Option<crux_core::Request<PersistenceOperation>> {
    cmd.effects().find_map(|e| match e {
        Effect::Persistence(req)
            if matches!(req.operation, PersistenceOperation::LoadRecords(_)) =>
        {
            Some(req)
        }
        _ => None,
    })
}

#[test]
fn a_save_sent_while_the_stored_copy_loads_makes_the_merge_load_again() {
    let mut model = Model::default();
    let mut cmd = Intrada.update(
        Event::Sync(SyncEvent::RecordsArrived(vec![arrived_piece(
            60, "Nocturne",
        )])),
        &mut model,
    );
    let mut load = load_request(&mut cmd).expect("loads the stored copy");
    let _save = persistence::save_item(&mut model, Item::fixture("piece"));

    load.resolve(PersistenceOutput::Records(stored_piece(0)))
        .expect("resolves once");
    let loaded = cmd.events().next().expect("loaded");
    let mut cmd = Intrada.update(loaded, &mut model);
    let effects: Vec<_> = cmd
        .effects()
        .filter_map(|e| match e {
            Effect::Persistence(req) => Some(req.operation),
            _ => None,
        })
        .collect();
    assert_eq!(
        effects,
        vec![PersistenceOperation::LoadRecords(vec![arrived_piece(
            60, "Nocturne"
        )
        .key()])]
    );
}

#[test]
fn a_batch_arriving_mid_merge_waits_then_merges() {
    for (case, first_stored, expect_write) in [
        ("the first merge writes", StoredRecords::default(), true),
        ("the first merge keeps everything", stored_piece(600), false),
    ] {
        let mut model = Model::default();
        let first = arrived_piece(60, "First");
        let second = SyncRecord {
            id: "other".to_string(),
            ..arrived_piece(60, "Second")
        };
        let mut cmd = Intrada.update(
            Event::Sync(SyncEvent::RecordsArrived(vec![first])),
            &mut model,
        );
        let mut load = load_request(&mut cmd).expect("loads the first batch");
        let mut waiting = Intrada.update(
            Event::Sync(SyncEvent::RecordsArrived(vec![second.clone()])),
            &mut model,
        );
        assert!(waiting.effects().next().is_none(), "{case}");

        load.resolve(PersistenceOutput::Records(first_stored))
            .expect("resolves once");
        let loaded = cmd.events().next().expect("loaded");
        let mut cmd = Intrada.update(loaded, &mut model);
        let mut next = if expect_write {
            let mut write = cmd
                .effects()
                .find_map(|e| match e {
                    Effect::Persistence(req) => Some(req),
                    _ => None,
                })
                .expect("the first merge writes");
            write
                .resolve(PersistenceOutput::Ack)
                .expect("resolves once");
            let landed = cmd.events().next().expect("lands");
            Intrada.update(landed, &mut model)
        } else {
            cmd
        };
        let load = load_request(&mut next).expect(case);
        assert_eq!(
            load.operation,
            PersistenceOperation::LoadRecords(vec![second.key()]),
            "{case}"
        );
    }
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
        unreadable: vec![],
    }))
    .expect("resolves once");
    let loaded = cmd.events().next().expect("loaded");
    let mut cmd = Intrada.update(loaded, &mut model);
    let sent = uploads(&mut cmd);
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0].deleted_at, Some(at(5)));
}
