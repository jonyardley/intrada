//! Seeds are the shapes the iPhone's GRDB store wrote, copied from its
//! `LibraryStoreMigrationTests` and `LibraryStoreTests` (the Swift files the
//! headers below name were deleted in #2432), never written to match this port
//! (#1256).

use chrono::{DateTime, Utc};
use intrada_core::domain::chart::{
    Bar, ChartChord, ChartSection, ChordChart, ChordQuality, ChordSymbol,
};
use intrada_core::domain::link::ExerciseLink;
use intrada_core::domain::section::{BarRange, ItemSection, SectionKind};
use intrada_core::domain::session::{Play, RepAction, RepEvent};
use intrada_core::domain::{Metre, Variation};
use intrada_core::persistence::{StoredItem, StoredRecords};
use intrada_core::sync::{RecordKey, RecordKind};
use intrada_core::{
    Accidental, CompletionStatus, EntryStatus, Item, ItemKind, Key, Letter, Modality,
    PersistenceOperation, PersistenceOutput, PracticeSession, SetlistEntry, Tempo,
};
use rusqlite::Connection;

use crate::codec;
use crate::migrations::{self, MIGRATIONS};
use crate::Store;

const EARLY: &str = "2026-01-01T00:00:00Z";

fn at(text: &str) -> DateTime<Utc> {
    text.parse().expect("a time")
}

/// Migrated to `version`, seeded with raw rows at that schema, then opened,
/// which finishes the migrations: the GRDB store's `upgradeTestStore`.
fn upgraded(version: &str, seed: &str) -> Store {
    let mut conn = Connection::open_in_memory().expect("opens");
    migrations::migrate(&mut conn, version, &mut vec![]).expect("migrates");
    conn.execute_batch(seed).expect("seeds");
    Store::new(conn, migrations::latest()).expect("finishes")
}

impl Store {
    fn items(&mut self) -> Vec<Item> {
        match self.ok(&PersistenceOperation::LoadItems) {
            PersistenceOutput::Items(items) => items,
            other => panic!("expected items, got {other:?}"),
        }
    }

    fn item(&mut self, id: &str) -> Item {
        self.items()
            .into_iter()
            .find(|i| i.id == id)
            .unwrap_or_else(|| panic!("no item {id}"))
    }

    fn sessions(&mut self) -> Vec<PracticeSession> {
        match self.ok(&PersistenceOperation::LoadSessions) {
            PersistenceOutput::Sessions(sessions) => sessions,
            other => panic!("expected sessions, got {other:?}"),
        }
    }

    fn variations(&mut self) -> Vec<Variation> {
        match self.ok(&PersistenceOperation::LoadVariations) {
            PersistenceOutput::Variations(variations) => variations,
            other => panic!("expected variations, got {other:?}"),
        }
    }

    fn ok(&mut self, operation: &PersistenceOperation) -> PersistenceOutput {
        let answer = self.handle(operation);
        assert_eq!(answer.error, None);
        answer.output
    }

    fn save(&mut self, item: &Item) {
        assert_eq!(
            self.ok(&PersistenceOperation::SaveItem(item.clone())),
            PersistenceOutput::Ack
        );
    }

    fn raw(&self, sql: &str) -> Option<String> {
        self.conn
            .query_row(sql, [], |row| row.get(0))
            .expect("reads")
    }

    fn save_session(&mut self, session: &PracticeSession) {
        assert_eq!(
            self.ok(&PersistenceOperation::SaveSession(session.clone())),
            PersistenceOutput::Ack
        );
    }

    fn columns(&self, table: &str) -> Vec<String> {
        self.conn
            .prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))
            .expect("prepares")
            .query_map([], |row| row.get(0))
            .expect("queries")
            .collect::<rusqlite::Result<_>>()
            .expect("reads")
    }
}

fn key(letter: Letter, accidental: Accidental, mode: Option<Modality>) -> Key {
    Key {
        letter,
        accidental,
        mode,
    }
}

fn fixture() -> Item {
    Item {
        id: "p1".into(),
        title: "Nocturne in E flat".into(),
        kind: ItemKind::Piece,
        composer: Some("Chopin".into()),
        key: Some(key(Letter::E, Accidental::Flat, Some(Modality::Major))),
        tempo: Some(Tempo {
            marking: Some("Andante".into()),
            bpm: Some(66),
        }),
        notes: Some("left hand / voicing".into()),
        tags: vec!["romantic".into(), "recital".into()],
        created_at: at("2026-09-01T10:00:00Z"),
        updated_at: at("2026-09-02T11:30:00.123456789Z"),
        priority: true,
        chord_chart: Some(ChordChart {
            key: Some(key(Letter::G, Accidental::Natural, Some(Modality::Minor))),
            sections: vec![ChartSection {
                label: Some("A".into()),
                bars: vec![Bar {
                    chords: vec![ChartChord {
                        symbol: ChordSymbol {
                            root: 0,
                            quality: ChordQuality::Min7,
                            extensions: vec!["9".into()],
                            bass: Some(7),
                            raw: "Cm9/G".into(),
                        },
                    }],
                }],
            }],
        }),
        photo_id: Some("01JPHOTO".into()),
        metre: Some(Metre {
            beats: 7,
            unit: 8,
            groups: Some(vec![2, 2, 3]),
        }),
        sections: vec![
            ItemSection {
                id: "s1".into(),
                name: "A".into(),
                bars: Some(BarRange { first: 1, last: 8 }),
                kind: SectionKind::Form,
                target_bpm: None,
                position: 0,
                updated_at: at("2026-09-02T11:30:00Z"),
                deleted_at: None,
            },
            ItemSection {
                id: "s2".into(),
                name: String::new(),
                bars: Some(BarRange {
                    first: 12,
                    last: 14,
                }),
                kind: SectionKind::TroubleSpot,
                target_bpm: Some(52),
                position: 1,
                updated_at: at("2026-09-02T11:30:00Z"),
                deleted_at: Some(at("2026-09-03T09:00:00Z")),
            },
        ],
        variation_ids: vec!["00000000000000000000000001".into()],
        keys: vec![
            key(Letter::C, Accidental::Sharp, Some(Modality::Minor)),
            key(Letter::B, Accidental::Flat, None),
        ],
        exercise_links: vec![ExerciseLink {
            id: "l1".into(),
            exercise_id: "e1".into(),
            section_id: Some("s2".into()),
            position: 0,
            updated_at: at("2026-09-02T11:30:00Z"),
            deleted_at: None,
        }],
    }
}

fn bare(id: &str, kind: ItemKind) -> Item {
    Item {
        id: id.into(),
        title: id.into(),
        kind,
        composer: None,
        key: None,
        tempo: None,
        notes: None,
        tags: vec![],
        created_at: at(EARLY),
        updated_at: at(EARLY),
        priority: false,
        chord_chart: None,
        photo_id: None,
        metre: None,
        sections: vec![],
        variation_ids: vec![],
        keys: vec![],
        exercise_links: vec![],
    }
}

/// `LibraryStoreTests.item`.
fn etude(id: &str, kind: ItemKind) -> Item {
    Item {
        title: "Etude".into(),
        composer: Some("Chopin".into()),
        key: Some(key(Letter::C, Accidental::Natural, Some(Modality::Major))),
        tempo: Some(Tempo {
            marking: Some("Allegro".into()),
            bpm: Some(132),
        }),
        notes: Some("evenness".into()),
        tags: vec!["scale".into(), "warmup".into()],
        priority: true,
        ..bare(id, kind)
    }
}

/// `LibraryItemFixture.record`.
fn record(id: &str, title: &str) -> Item {
    Item {
        title: title.into(),
        ..bare(id, ItemKind::Piece)
    }
}

fn tap(action: RepAction, time: &str) -> RepEvent {
    RepEvent {
        action,
        at: at(time),
        tempo: None,
        click_sounding: None,
    }
}

/// `LibraryStoreTests.entry`.
fn entry(id: &str) -> SetlistEntry {
    SetlistEntry {
        id: id.into(),
        item_id: format!("item-{id}"),
        item_title: "Etude".into(),
        item_type: ItemKind::Exercise,
        position: 0,
        duration_secs: 300,
        status: EntryStatus::Completed,
        notes: Some("good".into()),
        intention: Some("evenness".into()),
        planned_duration_secs: Some(300),
        group_id: None,
        planned_variation_ids: vec![],
        planned_rep_target: Some(5),
        plays: vec![Play {
            id: format!("{id}-p1"),
            section_id: None,
            key: None,
            variation_ids: vec![],
            started_at: at(EARLY),
            seconds: 300,
            rep_target: Some(5),
            rep_count: Some(5),
            rep_history: Some(vec![
                tap(RepAction::Success, "2026-01-01T00:01:00Z"),
                tap(RepAction::Missed, "2026-01-01T00:02:00Z"),
                tap(RepAction::Success, "2026-01-01T00:03:30Z"),
            ]),
            tempo_changes: vec![],
            achieved_tempo: Some(120),
            click_pattern: None,
            score: Some(4),
            away: vec![],
        }],
        segments: vec![],
        focus: None,
        intention_met: None,
        felt: None,
        got_in_the_way: vec![],
        note_points: vec![],
        planned_key: None,
    }
}

/// `LibraryStoreTests.session`.
fn session(id: &str, completed_at: &str) -> PracticeSession {
    PracticeSession {
        id: id.into(),
        entries: vec![entry("a"), entry("b")],
        session_notes: Some("solid".into()),
        started_at: at(EARLY),
        completed_at: at(completed_at),
        total_duration_secs: 600,
        completion_status: CompletionStatus::Completed,
        session_score: None,
        capture_version: None,
    }
}

// ── Migrations ──

#[test]
fn every_earlier_version_reaches_the_current_one_with_its_rows() {
    for (version, _) in MIGRATIONS {
        let mut store = upgraded(
            version,
            &format!(
                "INSERT INTO item (id, title, kind, tags, created_at, updated_at)
                 VALUES ('p1', 'Waltz', 'piece', '[\"waltz\"]', '{EARLY}', '{EARLY}')"
            ),
        );
        let recorded: Vec<String> = store
            .conn
            .prepare("SELECT identifier FROM grdb_migrations ORDER BY rowid")
            .expect("prepares")
            .query_map([], |row| row.get(0))
            .expect("queries")
            .collect::<rusqlite::Result<_>>()
            .expect("reads");
        let all: Vec<String> = MIGRATIONS.iter().map(|(id, _)| id.to_string()).collect();
        assert_eq!(recorded, all, "from {version}");
        let item = store.item("p1");
        assert_eq!(
            (item.title.as_str(), item.tags),
            ("Waltz", vec!["waltz".to_string()]),
            "from {version}"
        );
    }
}

// Phones in testers' hands have these recorded in `grdb_migrations` by the
// GRDB store this replaced (#2432): one renamed or reordered runs again over
// their notebook.
#[test]
fn the_shipped_migrations_keep_their_ids_and_order() {
    let shipped = [
        "v1_item",
        "v2_add_modality",
        "v3_session",
        "v4_session_score",
        "v5_rescale_entry_scores",
        "v6_item_linked_exercises",
        "v7_session_reflections",
        "v8_item_chord_chart",
        "v9_variant",
        "v10_coach_records",
        "v11_built_session",
        "v12_block_origin",
        "v13_wander_item",
        "v14_unmonitored_play",
        "v15_reflection_steer",
        "v16_item_photo",
        "v17_item_metre",
        "v18_section",
        "v19_keys_and_variations",
        "v20_exercise_link",
    ];
    let here: Vec<&str> = MIGRATIONS.iter().map(|(id, _)| *id).collect();
    assert_eq!(here[..shipped.len()], shipped);
}

#[test]
fn a_recorded_migration_is_not_run_again() {
    let mut conn = Connection::open_in_memory().expect("opens");
    migrations::migrate(&mut conn, migrations::latest(), &mut vec![]).expect("migrates");
    migrations::migrate(&mut conn, migrations::latest(), &mut vec![])
        .expect("a second open finds nothing to do, where re-running v1 would fail");
}

#[test]
fn every_table_carries_the_sync_columns() {
    let store = Store::in_memory().expect("opens");
    let tables: Vec<String> = store
        .conn
        .prepare(
            "SELECT name FROM sqlite_master
             WHERE type = 'table' AND name NOT GLOB 'sqlite_*' AND name != 'grdb_migrations'",
        )
        .expect("prepares")
        .query_map([], |row| row.get(0))
        .expect("queries")
        .collect::<rusqlite::Result<_>>()
        .expect("reads");
    assert!(tables.contains(&"item".to_string()), "{tables:?}");
    for table in tables {
        let columns: Vec<String> = store
            .conn
            .prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))
            .expect("prepares")
            .query_map([], |row| row.get(0))
            .expect("queries")
            .collect::<rusqlite::Result<_>>()
            .expect("reads");
        assert!(columns.contains(&"updated_at".to_string()), "{table}");
        assert!(columns.contains(&"deleted_at".to_string()), "{table}");
    }
}

fn v3_session(id: &str, entries: &str) -> String {
    format!(
        "INSERT INTO session (id, started_at, completed_at, total_duration_secs,
           completion_status, session_notes, session_intention, entries, updated_at, deleted_at)
         VALUES ('{id}', '2026-01-01T00:00:00Z', '2026-01-01T00:01:00Z', 60, 'completed', NULL,
           NULL, '{entries}', '2026-01-01T00:00:00Z', NULL);"
    )
}

fn entries_of(store: &Store, id: &str) -> serde_json::Value {
    let text = store
        .raw(&format!("SELECT entries FROM session WHERE id = '{id}'"))
        .expect("a row");
    serde_json::from_str(&text).expect("json")
}

#[test]
fn v5_doubles_each_score_caps_at_ten_and_skips_what_it_cannot_read() {
    let seed = [
        v3_session(
            "good",
            r#"[{"id":"e1","score":2},{"id":"e2","score":4},{"id":"e3","score":7}]"#,
        ),
        v3_session(
            "kept",
            r#"[{"id":"e1","itemId":"i1","itemTitle":"Bach","itemType":"piece","position":0,"durationSecs":120,"status":"completed","score":5,"notes":"keep"},{"id":"e2","itemId":"i2","itemTitle":"Scales","itemType":"exercise","position":1,"durationSecs":60,"status":"completed"}]"#,
        ),
        v3_session("legacy", r#"[{"id":"e1","score":3,"legacyKey":"x"}]"#),
        v3_session("bad", "not json"),
        v3_session("object", r#"{"a":{"score":3}}"#),
    ]
    .concat();
    let mut store = upgraded("v4_session_score", &seed);

    let good = entries_of(&store, "good");
    let scores: Vec<_> = good
        .as_array()
        .expect("array")
        .iter()
        .map(|e| (e["id"].clone(), e["score"].clone()))
        .collect();
    assert_eq!(
        scores,
        vec![
            ("e1".into(), 4.into()),
            ("e2".into(), 8.into()),
            ("e3".into(), 10.into())
        ]
    );
    let kept = entries_of(&store, "kept");
    assert_eq!(kept[0]["score"], 10);
    assert_eq!(kept[0]["notes"], "keep");
    assert!(kept[1].get("score").is_none(), "no score gains none");
    let legacy = entries_of(&store, "legacy");
    assert_eq!(legacy[0]["score"], 6);
    assert_eq!(legacy[0]["legacyKey"], "x");
    assert!(legacy[0].get("itemId").is_none(), "nothing is added");
    assert_eq!(
        store.raw("SELECT entries FROM session WHERE id = 'bad'"),
        Some("not json".into())
    );
    assert_eq!(
        store.raw("SELECT entries FROM session WHERE id = 'object'"),
        Some(r#"{"a":{"score":3}}"#.into())
    );

    let first = store.handle(&PersistenceOperation::LoadVariations);
    assert!(
        first
            .unreadable
            .contains(&"v5_rescale_entry_scores skipped row bad".to_string()),
        "{:?}",
        first.unreadable
    );
    assert!(store
        .handle(&PersistenceOperation::LoadVariations)
        .unreadable
        .is_empty());
}

#[test]
fn v17_gives_a_charted_piece_its_metre_in_crotchets() {
    let chart = r#"{"key":"G","modality":"minor","metre":3,"sections":[]}"#;
    let mut store = upgraded(
        "v16_item_photo",
        &format!(
            "INSERT INTO item (id, title, kind, key, modality, tags, linked_exercise_ids,
               created_at, updated_at, priority, chord_chart)
             VALUES ('p1', 'Waltz', 'piece', 'G', 'minor', '[]', '[]', '{EARLY}', '{EARLY}', 0,
               '{chart}');
             INSERT INTO item (id, title, kind, tags, created_at, updated_at)
             VALUES ('p2', 'Plain', 'piece', '[]', '{EARLY}', '{EARLY}');"
        ),
    );
    let charted = store.item("p1");
    assert_eq!(
        charted.metre,
        Some(Metre {
            beats: 3,
            unit: 4,
            groups: None
        })
    );
    assert!(charted.chord_chart.is_some());
    assert_eq!(store.item("p2").metre, None);
}

#[test]
fn v19_keeps_every_key_chart_and_session() {
    let chart = r#"{"key":"G","modality":"minor","sections":[]}"#;
    let entries = r#"[{"id":"e1","itemId":"p1","itemTitle":"Nocturne","itemType":"piece","position":0,"durationSecs":300,"status":"completed","plannedVariationId":"v-old","plays":[{"id":"pl1","variationId":"v-old","startedAt":"2026-09-01T10:00:00Z","seconds":300,"repHistory":[{"action":"success","at":"2026-09-01T10:01:00Z"}],"score":7}]}]"#;
    let mut store = upgraded(
        "v18_section",
        &format!(
            "INSERT INTO item (id, title, kind, key, modality, tags, created_at, updated_at,
               chord_chart)
             VALUES ('p1', 'Nocturne', 'piece', 'Eb', 'major', '[]', '{EARLY}', '{EARLY}',
               '{chart}');
             INSERT INTO item (id, title, kind, key, modality, tags, created_at, updated_at)
             VALUES ('p2', 'Freeform', 'piece', 'F# major', NULL, '[]', '{EARLY}', '{EARLY}');
             INSERT INTO item (id, title, kind, key, modality, tags, created_at, updated_at)
             VALUES ('p3', 'Odd', 'piece', 'H dorian', NULL, '[]', '{EARLY}', '{EARLY}');
             INSERT INTO session (id, started_at, completed_at, total_duration_secs,
               completion_status, session_notes, session_intention, entries, updated_at,
               deleted_at)
             VALUES ('s1', '2026-09-01T10:00:00Z', '2026-09-01T10:05:00Z', 300, 'completed',
               NULL, NULL, '{entries}', '2026-09-01T10:05:00Z', NULL);"
        ),
    );
    assert_eq!(store.variations(), vec![]);
    let nocturne = store.item("p1");
    assert_eq!(
        nocturne.key,
        Some(key(Letter::E, Accidental::Flat, Some(Modality::Major)))
    );
    assert_eq!(
        nocturne.chord_chart.and_then(|c| c.key),
        Some(key(Letter::G, Accidental::Natural, Some(Modality::Minor)))
    );
    assert_eq!((nocturne.variation_ids, nocturne.keys), (vec![], vec![]));
    assert_eq!(
        store.item("p2").key,
        Some(key(Letter::F, Accidental::Sharp, Some(Modality::Major)))
    );

    let mut odd = store.item("p3");
    assert_eq!(odd.key, None);
    odd.title = "Odd, renamed".into();
    store.save(&odd);
    assert_eq!(
        store.raw("SELECT key FROM item WHERE id = 'p3'"),
        Some("H dorian".into()),
        "a key the core cannot read stays until one is picked"
    );

    for (table, column) in [
        ("variation", "label"),
        ("item", "variation_ids"),
        ("item", "keys"),
        ("session", "capture_version"),
    ] {
        assert!(
            store.columns(table).contains(&column.to_string()),
            "{table}.{column}"
        );
    }

    let session = store.sessions().pop().expect("a session");
    assert_eq!(session.capture_version, None);
    let entry = &session.entries[0];
    assert_eq!(entry.planned_variation_ids, Vec::<String>::new());
    let play = &entry.plays[0];
    assert_eq!(play.score, Some(7));
    assert_eq!(play.variation_ids, Vec::<String>::new());
    let taps = play.rep_history.clone().expect("taps");
    assert_eq!(
        taps.iter().map(|t| t.action).collect::<Vec<_>>(),
        vec![RepAction::Success]
    );
    assert_eq!(taps[0].tempo, None);
    assert_eq!(play.tempo_changes, vec![]);
}

#[test]
fn v20_copies_every_link_as_a_whole_piece_link() {
    let seed: String = [
        ("p1", "piece", r#"["b", "a", "b"]"#),
        ("p2", "piece", "{"),
        ("p3", "piece", r#"{"x": "y"}"#),
        ("p4", "piece", r#"[1, "a"]"#),
        ("x-y", "piece", r#"["z"]"#),
        ("x", "piece", r#"["y-z"]"#),
        ("a", "exercise", "[]"),
        ("b", "exercise", "[]"),
    ]
    .iter()
    .map(|(id, kind, links)| {
        format!(
            "INSERT INTO item (id, title, kind, tags, linked_exercise_ids, created_at, updated_at)
             VALUES ('{id}', '{id}', '{kind}', '[]', '{links}', '{EARLY}',
               '2026-01-02T00:00:00Z');"
        )
    })
    .collect();
    let mut store = upgraded("v19_keys_and_variations", &seed);
    let link = |piece: &str, exercise: &str, position: usize| ExerciseLink {
        id: format!("link|{piece}|{exercise}"),
        exercise_id: exercise.into(),
        section_id: None,
        position,
        updated_at: at("2026-01-02T00:00:00Z"),
        deleted_at: None,
    };
    let links = |store: &mut Store, id: &str| store.item(id).exercise_links;
    assert_eq!(
        links(&mut store, "p1"),
        vec![link("p1", "b", 0), link("p1", "a", 1)]
    );
    assert_eq!(links(&mut store, "p2"), vec![]);
    assert_eq!(links(&mut store, "p3"), vec![]);
    assert_eq!(links(&mut store, "p4"), vec![link("p4", "a", 1)]);
    assert_eq!(
        store.raw("SELECT linked_exercise_ids FROM item WHERE id = 'p2'"),
        Some("{".into())
    );
    assert_eq!(links(&mut store, "a"), vec![]);
    assert_eq!(links(&mut store, "x-y"), vec![link("x-y", "z", 0)]);
    assert_eq!(links(&mut store, "x"), vec![link("x", "y-z", 0)]);
}

// ── Round trips ──

#[test]
fn an_item_reads_back_as_written() {
    let mut store = Store::in_memory().expect("opens");
    let item = fixture();
    store.save(&item);
    assert_eq!(store.items(), vec![item]);
}

#[test]
fn a_save_revives_and_updates_by_id() {
    let mut store = Store::in_memory().expect("opens");
    let mut item = fixture();
    store.save(&item);
    store.ok(&PersistenceOperation::DeleteItem {
        id: item.id.clone(),
        deleted_at: at("2026-09-04T00:00:00Z"),
    });
    assert_eq!(store.items(), vec![]);
    assert_eq!(
        store.raw("SELECT deleted_at FROM item WHERE id = 'p1'"),
        Some("2026-09-04T00:00:00Z".into()),
        "a delete leaves a tombstone"
    );
    item.title = "Renamed".into();
    item.composer = None;
    item.chord_chart = None;
    item.metre = None;
    item.key = None;
    item.tempo = None;
    store.save(&item);
    assert_eq!(store.items(), vec![item]);
}

#[test]
fn items_load_newest_first() {
    let mut store = Store::in_memory().expect("opens");
    let mut older = bare("older", ItemKind::Exercise);
    older.created_at = at("2026-01-01T00:00:00Z");
    let mut newer = bare("newer", ItemKind::Piece);
    newer.created_at = at("2026-02-01T00:00:00Z");
    store.save(&older);
    store.save(&newer);
    let ids: Vec<String> = store.items().into_iter().map(|i| i.id).collect();
    assert_eq!(ids, vec!["newer", "older"]);
}

#[test]
fn a_batch_lands_whole() {
    let mut store = Store::in_memory().expect("opens");
    let batch = vec![bare("a", ItemKind::Exercise), bare("b", ItemKind::Exercise)];
    assert_eq!(
        store.ok(&PersistenceOperation::SaveItems(batch)),
        PersistenceOutput::Ack
    );
    let mut ids: Vec<String> = store.items().into_iter().map(|i| i.id).collect();
    ids.sort();
    assert_eq!(ids, vec!["a", "b"]);
}

#[test]
fn a_batch_that_fails_part_way_lands_nothing() {
    let mut store = Store::in_memory().expect("opens");
    let mut broken = bare("b", ItemKind::Piece);
    broken.sections = vec![ItemSection {
        position: usize::MAX,
        ..fixture().sections[0].clone()
    }];
    let answer = store.handle(&PersistenceOperation::SaveItems(vec![
        bare("a", ItemKind::Exercise),
        broken,
    ]));
    assert_eq!(answer.output, PersistenceOutput::Failed);
    assert!(answer.error.is_some());
    assert_eq!(
        store.items(),
        vec![],
        "the first item rolled back with the second"
    );
}

#[test]
fn a_session_reads_back_as_written() {
    let entries = r#"[{"id":"e1","itemId":"i1","itemTitle":"Major Scales","itemType":"exercise","position":0,"durationSecs":600,"status":"completed","score":7,"intention":"even tone","repTarget":10,"repCount":8,"repTargetReached":false,"repHistory":[{"action":"success","at":"2026-09-01T10:01:00Z"},{"action":"missed","at":"2026-09-01T10:02:00Z"}],"plannedDurationSecs":600,"achievedTempo":120,"clickPattern":{"metre":{"beats":3,"unit":4},"sounding":1},"groupId":"g1","variantId":"v-c"}]"#;
    let mut store = upgraded(
        migrations::latest(),
        &format!(
            "INSERT INTO session (id, started_at, completed_at, total_duration_secs,
               completion_status, entries, updated_at)
             VALUES ('s1', '2026-09-01T10:00:00Z', '2026-09-01T10:10:00Z', 600, 'completed',
               '{entries}', '2026-09-01T10:10:00Z');"
        ),
    );
    let session = store
        .sessions()
        .pop()
        .expect("the legacy row folds and reads");
    assert_eq!(
        store.ok(&PersistenceOperation::SaveSession(session.clone())),
        PersistenceOutput::Ack
    );
    assert_eq!(store.sessions(), vec![session]);
}

#[test]
fn a_session_the_core_refuses_is_skipped_and_named() {
    let mut store = upgraded(
        migrations::latest(),
        "INSERT INTO session (id, started_at, completed_at, total_duration_secs,
           completion_status, entries, updated_at)
         VALUES ('s1', 'yesterday', '2026-09-01T10:10:00Z', 600, 'completed', '[]',
           '2026-09-01T10:10:00Z');",
    );
    let answer = store.handle(&PersistenceOperation::LoadSessions);
    assert_eq!(answer.output, PersistenceOutput::Sessions(vec![]));
    assert!(
        answer
            .unreadable
            .iter()
            .any(|u| u.starts_with("session s1 skipped")),
        "{:?}",
        answer.unreadable
    );
}

#[test]
fn variations_read_back_with_their_tombstones_in_the_order_written() {
    let mut store = Store::in_memory().expect("opens");
    let rows = vec![
        Variation {
            id: "v2".into(),
            label: "Dotted rhythms".into(),
            updated_at: at(EARLY),
            deleted_at: None,
        },
        Variation {
            id: "v1".into(),
            label: "Hands separately".into(),
            updated_at: at(EARLY),
            deleted_at: Some(at("2026-02-01T00:00:00Z")),
        },
    ];
    store.ok(&PersistenceOperation::SaveVariations(rows.clone()));
    assert_eq!(store.variations(), rows);
    let renamed = Variation {
        label: "Swung".into(),
        ..rows[0].clone()
    };
    store.ok(&PersistenceOperation::SaveVariations(vec![renamed.clone()]));
    assert_eq!(store.variations(), vec![renamed, rows[1].clone()]);
}

// ── Values the core cannot read survive a save (#1117, #2097, #2106) ──

fn with_stored(columns: &str, values: &str) -> Store {
    upgraded(
        migrations::latest(),
        &format!(
            "INSERT INTO item (id, title, kind, created_at, updated_at, {columns})
             VALUES ('p1', 'Waltz', 'piece', '{EARLY}', '{EARLY}', {values})"
        ),
    )
}

#[test]
fn an_unreadable_key_in_the_list_survives_a_save_that_leaves_the_list_alone() {
    let stored = r#"[{"key":"H"},{"key":"C","modality":"major"}]"#;
    let mut store = with_stored("tags, keys", &format!("'[]', '{stored}'"));
    let mut item = store.item("p1");
    assert_eq!(
        item.keys,
        vec![key(Letter::C, Accidental::Natural, Some(Modality::Major))]
    );
    item.title = "Waltz, renamed".into();
    store.save(&item);
    assert_eq!(
        store.raw("SELECT keys FROM item WHERE id = 'p1'"),
        Some(stored.into())
    );

    item.keys = vec![key(Letter::D, Accidental::Natural, None)];
    store.save(&item);
    assert_eq!(
        store.item("p1").keys,
        item.keys,
        "a real change replaces it"
    );
}

#[test]
fn an_unreadable_list_survives_a_save_as_empty() {
    let mut store = with_stored("tags, variation_ids, keys", "'{broken', '[1]', 'nope'");
    let answer = store.handle(&PersistenceOperation::LoadItems);
    assert!(
        answer
            .unreadable
            .iter()
            .any(|u| u.starts_with("tags of p1")),
        "{:?}",
        answer.unreadable
    );
    let item = store.item("p1");
    assert_eq!(
        (&item.tags, &item.variation_ids, &item.keys),
        (&vec![], &vec![], &vec![])
    );
    store.save(&item);
    assert_eq!(
        store.raw("SELECT tags || ' ' || variation_ids || ' ' || keys FROM item"),
        Some("{broken [1] nope".into())
    );
}

#[test]
fn an_unreadable_chart_and_metre_survive_a_save() {
    let mut store = with_stored(
        "tags, chord_chart, metre",
        r#"'[]', '{"key":"G"}', '{"beats":"three"}'"#,
    );
    let item = store.item("p1");
    assert_eq!((&item.chord_chart, &item.metre), (&None, &None));
    store.save(&item);
    assert_eq!(
        store.raw("SELECT chord_chart || ' ' || metre FROM item"),
        Some(r#"{"key":"G"} {"beats":"three"}"#.into())
    );
}

#[test]
fn a_chart_keeps_a_key_the_core_cannot_read() {
    let chart = r#"{"key":"H","modality":"minor","sections":[]}"#;
    let mut store = with_stored("tags, chord_chart", &format!("'[]', '{chart}'"));
    let item = store.item("p1");
    assert_eq!(item.chord_chart.as_ref().and_then(|c| c.key), None);
    store.save(&item);
    let stored: serde_json::Value =
        serde_json::from_str(&store.raw("SELECT chord_chart FROM item").expect("a chart"))
            .expect("json");
    assert_eq!(
        (&stored["key"], &stored["modality"]),
        (&"H".into(), &"minor".into())
    );
}

#[test]
fn an_unknown_stored_value_falls_back_and_is_named() {
    let chart = r#"{"key":"C","modality":"major","sections":[{"bars":[{"chords":[{"symbol":{"root":0,"quality":"lydian","extensions":[],"raw":"Clyd"}}]}]}]}"#;
    let mut store = upgraded(
        migrations::latest(),
        &format!(
            "INSERT INTO item (id, title, kind, tags, created_at, updated_at, chord_chart)
             VALUES ('p1', 'Tune', 'etude', '[]', '{EARLY}', '{EARLY}', '{chart}');
             INSERT INTO section (id, item_id, name, kind, position, updated_at)
             VALUES ('s1', 'p1', 'A', 'coda', 0, '{EARLY}');"
        ),
    );
    let answer = store.handle(&PersistenceOperation::LoadItems);
    let PersistenceOutput::Items(items) = answer.output else {
        panic!("items")
    };
    let item = &items[0];
    assert_eq!(item.kind, ItemKind::Piece);
    assert_eq!(item.sections[0].kind, SectionKind::Form);
    let quality = item.chord_chart.as_ref().map(|c| c.changes()[0].quality);
    assert_eq!(quality, Some(ChordQuality::Other));
    for named in [
        "unknown ItemKind on decode: \"etude\"",
        "unknown SectionKind on decode: \"coda\"",
        "unknown ChordQuality on decode: \"lydian\"",
    ] {
        assert!(
            answer.unreadable.contains(&named.to_string()),
            "{named} in {:?}",
            answer.unreadable
        );
    }
}

#[test]
fn a_row_with_a_time_that_will_not_read_is_skipped_and_left_alone() {
    let mut store = upgraded(
        migrations::latest(),
        &format!(
            "INSERT INTO item (id, title, kind, tags, created_at, updated_at)
             VALUES ('bad', 'Bad', 'piece', '[]', 'last tuesday', '{EARLY}');
             INSERT INTO item (id, title, kind, tags, created_at, updated_at)
             VALUES ('good', 'Good', 'piece', '[]', '{EARLY}', '{EARLY}');"
        ),
    );
    let answer = store.handle(&PersistenceOperation::LoadItems);
    let PersistenceOutput::Items(items) = answer.output else {
        panic!("items")
    };
    assert_eq!(
        items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        vec!["good"]
    );
    assert!(
        answer
            .unreadable
            .iter()
            .any(|u| u.starts_with("item bad skipped")),
        "{:?}",
        answer.unreadable
    );
}

#[test]
fn a_time_is_stored_as_the_bridge_writes_it() {
    let mut store = Store::in_memory().expect("opens");
    store.save(&fixture());
    let bridged = serde_json::to_value(fixture().updated_at).expect("json");
    assert_eq!(
        store.raw("SELECT updated_at FROM item"),
        bridged.as_str().map(str::to_string)
    );
}

// ── A database the last GRDB build wrote, kept as written (#2432) ──

#[test]
fn a_database_the_iphone_wrote_opens_with_every_row_intact() {
    let fixture_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/iphone-v20.sqlite"
    );
    let path = std::env::temp_dir().join(format!("intrada-iphone-{}.sqlite", std::process::id()));
    std::fs::copy(fixture_path, &path).expect("copies the fixture");
    let mut store = Store::open(&path).expect("opens");

    let first = store.handle(&PersistenceOperation::LoadItems);
    assert_eq!(first.unreadable, Vec::<String>::new());
    let mut exercise = bare("e1", ItemKind::Exercise);
    exercise.title = "Scales in thirds".into();
    exercise.created_at = at("2026-08-01T09:00:00Z");
    exercise.updated_at = at("2026-08-01T09:00:00Z");
    assert_eq!(
        first.output,
        PersistenceOutput::Items(vec![fixture(), exercise]),
        "the deleted piece stays hidden"
    );
    assert_eq!(
        store.raw("SELECT deleted_at FROM item WHERE id = 'gone'"),
        Some("2026-09-05T08:00:00Z".into())
    );

    assert_eq!(
        store.variations(),
        vec![
            Variation {
                id: "00000000000000000000000001".into(),
                label: "Hands separately".into(),
                updated_at: at("2026-09-01T10:00:00Z"),
                deleted_at: None,
            },
            Variation {
                id: "00000000000000000000000002".into(),
                label: "Dotted rhythms".into(),
                updated_at: at("2026-09-01T10:00:00Z"),
                deleted_at: Some(at("2026-09-04T10:00:00Z")),
            },
        ]
    );

    let session = store.sessions().pop().expect("a session");
    assert_eq!(
        (
            session.id.as_str(),
            session.session_notes.as_deref(),
            session.total_duration_secs,
            session.session_score,
            session.capture_version
        ),
        ("sess-1", Some("good day"), 600, Some(7), Some(1))
    );
    let play = &session.entries[0].plays[0];
    assert_eq!(
        (
            play.section_id.as_deref(),
            play.key,
            play.score,
            play.achieved_tempo
        ),
        (
            Some("s1"),
            Some(key(Letter::E, Accidental::Flat, Some(Modality::Major))),
            Some(8),
            Some(60)
        )
    );

    let mut renamed = fixture();
    renamed.title = "Nocturne, renamed".into();
    store.save(&renamed);
    drop(store);
    let mut reopened = Store::open(&path).expect("opens again");
    assert_eq!(reopened.item("p1"), renamed);
    let _ = std::fs::remove_file(&path);
}

// ── Whole rows from older schemas (LibraryStoreMigrationTests.swift) ──

#[test]
fn an_item_from_each_older_schema_loads_whole() {
    let mut store = upgraded(
        "v1_item",
        "INSERT INTO item
           (id, title, kind, composer, key, tempo_marking, tempo_bpm, notes, tags,
            created_at, updated_at, priority, deleted_at)
         VALUES ('p1', 'Legacy Etude', 'piece', 'Bach', 'C', 'Allegro', 120, 'phrasing',
                 '[\"scale\"]', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, NULL)",
    );
    assert_eq!(
        store.items(),
        vec![Item {
            composer: Some("Bach".into()),
            key: Some(key(Letter::C, Accidental::Natural, None)),
            tempo: Some(Tempo {
                marking: Some("Allegro".into()),
                bpm: Some(120),
            }),
            notes: Some("phrasing".into()),
            tags: vec!["scale".into()],
            ..record("p1", "Legacy Etude")
        }]
    );

    let mut store = upgraded(
        "v5_rescale_entry_scores",
        "INSERT INTO item
           (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
            created_at, updated_at, priority, deleted_at)
         VALUES ('p1', 'Legacy Piece', 'piece', NULL, NULL, NULL, NULL, NULL, NULL, '[]',
                 '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, NULL)",
    );
    assert!(store
        .columns("item")
        .contains(&"linked_exercise_ids".to_string()));
    assert_eq!(store.items(), vec![record("p1", "Legacy Piece")]);

    let mut store = upgraded(
        "v7_session_reflections",
        "INSERT INTO item
           (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
            linked_exercise_ids, created_at, updated_at, priority, deleted_at)
         VALUES ('p1', 'Legacy Piece', 'piece', NULL, NULL, NULL, NULL, NULL, NULL, '[]',
                 '[]', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, NULL)",
    );
    assert!(store.columns("item").contains(&"chord_chart".to_string()));
    assert_eq!(store.items(), vec![record("p1", "Legacy Piece")]);

    let mut store = upgraded(
        "v8_item_chord_chart",
        "INSERT INTO item
           (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
            linked_exercise_ids, created_at, updated_at, priority, deleted_at, chord_chart)
         VALUES ('e1', 'Legacy Exercise', 'exercise', NULL, NULL, NULL, NULL, NULL, NULL, '[]',
                 '[]', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, NULL, NULL)",
    );
    let variant = store.columns("variant");
    for expected in [
        "id",
        "item_id",
        "label",
        "position",
        "updated_at",
        "deleted_at",
    ] {
        assert!(variant.contains(&expected.to_string()), "{variant:?}");
    }
    assert_eq!(
        store.items(),
        vec![Item {
            title: "Legacy Exercise".into(),
            ..bare("e1", ItemKind::Exercise)
        }]
    );

    let mut store = upgraded(
        "v15_reflection_steer",
        "INSERT INTO item
           (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
            linked_exercise_ids, created_at, updated_at, priority, deleted_at, chord_chart)
         VALUES ('p1', 'Legacy Piece', 'piece', 'Chopin', 'E', NULL, NULL, NULL, NULL, '[]',
                 '[]', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, NULL, NULL)",
    );
    assert!(store.columns("item").contains(&"photo_id".to_string()));
    assert_eq!(
        store.items(),
        vec![Item {
            composer: Some("Chopin".into()),
            key: Some(key(Letter::E, Accidental::Natural, None)),
            ..record("p1", "Legacy Piece")
        }]
    );
}

#[test]
fn an_item_written_at_v17_loads_whole_with_no_sections() {
    let mut store = upgraded(
        "v17_item_metre",
        "INSERT INTO item
           (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
            linked_exercise_ids, created_at, updated_at, priority, chord_chart, photo_id, metre,
            deleted_at)
         VALUES ('p1', 'Waltz', 'piece', 'Chopin', 'A', 'minor', 'Lento', 60, 'slow', '[\"rubato\"]',
                 '[\"e1\"]', '2026-01-01T00:00:00Z', '2026-01-02T00:00:00Z', 1, NULL, NULL,
                 '{\"beats\":3,\"unit\":4}', NULL);
         INSERT INTO item (id, title, kind, tags, created_at, updated_at)
         VALUES ('e1', 'Scales', 'exercise', '[]', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
         INSERT INTO variant (id, item_id, label, position, updated_at, deleted_at)
         VALUES ('v1', 'e1', 'C', 0, '2026-01-01T00:00:00Z', NULL)",
    );
    let section = store.columns("section");
    for expected in [
        "id",
        "item_id",
        "name",
        "bar_first",
        "bar_last",
        "kind",
        "target_bpm",
        "position",
        "updated_at",
        "deleted_at",
    ] {
        assert!(section.contains(&expected.to_string()), "{section:?}");
    }
    assert_eq!(
        store.item("p1"),
        Item {
            composer: Some("Chopin".into()),
            key: Some(key(Letter::A, Accidental::Natural, Some(Modality::Minor))),
            tempo: Some(Tempo {
                marking: Some("Lento".into()),
                bpm: Some(60),
            }),
            notes: Some("slow".into()),
            tags: vec!["rubato".into()],
            exercise_links: vec![ExerciseLink {
                id: "link|p1|e1".into(),
                exercise_id: "e1".into(),
                section_id: None,
                position: 0,
                updated_at: at("2026-01-02T00:00:00Z"),
                deleted_at: None,
            }],
            updated_at: at("2026-01-02T00:00:00Z"),
            priority: true,
            metre: Some(Metre {
                beats: 3,
                unit: 4,
                groups: None,
            }),
            ..record("p1", "Waltz")
        }
    );
    let exercise = store.item("e1");
    assert_eq!(
        (exercise.variation_ids, exercise.sections),
        (vec![], vec![])
    );
}

#[test]
fn a_v3_session_gains_an_empty_session_score() {
    let mut store = upgraded(
        "v3_session",
        &v3_session(
            "s1",
            r#"[{"id":"e1","itemId":"i1","itemTitle":"Scales","itemType":"exercise","position":0,"durationSecs":60,"status":"completed","score":3}]"#,
        ),
    );
    assert!(entries_of(&store, "s1").to_string().contains("\"score\":6"));
    assert_eq!(
        store.raw("SELECT typeof(session_score) FROM session WHERE id = 's1'"),
        Some("null".into())
    );
    assert_eq!(store.sessions()[0].session_score, None);
}

#[test]
fn a_v6_session_keeps_its_notes_and_score() {
    let mut store = upgraded(
        "v6_item_linked_exercises",
        "INSERT INTO session
           (id, started_at, completed_at, total_duration_secs, completion_status,
            session_notes, session_intention, entries, updated_at, deleted_at, session_score)
         VALUES ('s-pre', '2026-01-01T00:00:00Z', '2026-01-01T00:01:00Z', 60, 'completed',
                 'old note', NULL, '[]', '2026-01-01T00:00:00Z', NULL, 7)",
    );
    let session = store.sessions().pop().expect("a session");
    assert_eq!(
        (session.session_notes.as_deref(), session.session_score),
        (Some("old note"), Some(7))
    );
}

#[test]
fn the_retired_session_columns_and_their_text_survive_a_save() {
    let mut store = upgraded(
        migrations::latest(),
        "INSERT INTO session
           (id, started_at, completed_at, total_duration_secs, completion_status,
            session_notes, session_intention, entries, updated_at, deleted_at,
            reflection_improved, reflection_still_rough, reflection_next_target)
         VALUES ('s-old', '2026-05-01T10:00:00Z', '2026-05-01T10:30:00Z', 1800, 'completed',
                 'old note', 'even RH at 96', '[]', '2026-05-01T10:30:00Z', NULL,
                 'thumb-unders even', 'bars 12-14 rush', 'bars 12-14 at 80')",
    );
    let retired = [
        ("session_intention", "even RH at 96"),
        ("reflection_improved", "thumb-unders even"),
        ("reflection_still_rough", "bars 12-14 rush"),
        ("reflection_next_target", "bars 12-14 at 80"),
    ];
    let columns = store.columns("session");
    for (column, _) in retired {
        assert!(columns.contains(&column.to_string()), "{columns:?}");
    }

    let mut reloaded = store.sessions().pop().expect("a session");
    reloaded.session_notes = Some("edited".into());
    store.save_session(&reloaded);

    assert_eq!(
        store.raw("SELECT session_notes FROM session WHERE id = 's-old'"),
        Some("edited".into())
    );
    for (column, text) in retired {
        assert_eq!(
            store.raw(&format!("SELECT {column} FROM session WHERE id = 's-old'")),
            Some(text.into()),
            "{column}"
        );
    }
}

// ── More round trips (LibraryStoreTests.swift, LibraryStoreMigrationTests.swift) ──

#[test]
fn an_exercise_with_no_tempo_or_tags_reads_back_as_written() {
    let mut store = Store::in_memory().expect("opens");
    let exercise = Item {
        tempo: None,
        tags: vec![],
        ..etude("e1", ItemKind::Exercise)
    };
    store.save(&exercise);
    assert_eq!(store.items(), vec![exercise]);
    assert_eq!(
        store.raw("SELECT kind FROM item WHERE id = 'e1'"),
        Some("exercise".into())
    );
}

#[test]
fn a_key_with_no_mode_reads_back_with_no_modality_stored() {
    let mut store = Store::in_memory().expect("opens");
    let item = Item {
        key: Some(key(Letter::F, Accidental::Sharp, None)),
        ..etude("p1", ItemKind::Piece)
    };
    store.save(&item);
    assert_eq!(store.items(), vec![item]);
    assert_eq!(
        store.raw("SELECT typeof(modality) FROM item WHERE id = 'p1'"),
        Some("null".into())
    );
}

#[test]
fn variations_and_keys_read_back_in_the_order_written() {
    let mut store = Store::in_memory().expect("opens");
    let exercise = Item {
        variation_ids: vec!["v-b".into(), "v-a".into()],
        keys: vec![
            key(Letter::E, Accidental::Flat, Some(Modality::Major)),
            key(Letter::C, Accidental::Sharp, Some(Modality::Minor)),
            key(Letter::G, Accidental::Natural, None),
        ],
        ..etude("e1", ItemKind::Exercise)
    };
    store.save(&exercise);
    assert_eq!(store.items(), vec![exercise]);
}

#[test]
fn saving_no_tags_over_readable_ones_clears_them() {
    let mut store = Store::in_memory().expect("opens");
    let mut item = etude("i1", ItemKind::Piece);
    store.save(&item);
    item.tags = vec![];
    store.save(&item);
    assert_eq!(
        store.raw("SELECT tags FROM item WHERE id = 'i1'"),
        Some("[]".into())
    );
}

#[test]
fn removing_a_photo_clears_it() {
    let mut store = Store::in_memory().expect("opens");
    let with_photo = Item {
        composer: Some("Chopin".into()),
        photo_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".into()),
        ..record("p1", "Nocturne")
    };
    store.save(&with_photo);
    assert_eq!(store.item("p1").photo_id, with_photo.photo_id);
    store.save(&Item {
        photo_id: None,
        updated_at: at("2026-01-01T00:01:00Z"),
        ..with_photo
    });
    assert_eq!(store.item("p1").photo_id, None);
}

#[test]
fn an_ended_early_session_with_no_entries_reads_back_as_written() {
    let mut store = Store::in_memory().expect("opens");
    let ended = PracticeSession {
        entries: vec![],
        completion_status: CompletionStatus::EndedEarly,
        session_notes: None,
        ..session("s2", "2026-01-01T00:10:00Z")
    };
    store.save_session(&ended);
    assert_eq!(store.sessions(), vec![ended]);
}

#[test]
fn sessions_load_newest_first() {
    let mut store = Store::in_memory().expect("opens");
    store.save_session(&session("old", "2026-01-01T00:00:00Z"));
    store.save_session(&session("new", "2026-02-01T00:00:00Z"));
    let ids: Vec<String> = store.sessions().into_iter().map(|s| s.id).collect();
    assert_eq!(ids, vec!["new", "old"]);
}

#[test]
fn sections_and_a_tombstone_update_by_id_on_a_second_save() {
    let mut store = Store::in_memory().expect("opens");
    let section = |id: &str, name: &str, bars: Option<(u16, u16)>, position: usize| ItemSection {
        id: id.into(),
        name: name.into(),
        bars: bars.map(|(first, last)| BarRange { first, last }),
        kind: SectionKind::Form,
        target_bpm: None,
        position,
        updated_at: at("2026-10-03T09:00:00Z"),
        deleted_at: None,
    };
    let mut item = Item {
        sections: vec![
            section("s1", "A1", Some((1, 16)), 0),
            ItemSection {
                updated_at: at("2026-10-03T09:05:00Z"),
                deleted_at: Some(at("2026-10-03T09:05:00Z")),
                ..section("s2", "Gone", None, 1)
            },
            ItemSection {
                kind: SectionKind::TroubleSpot,
                target_bpm: Some(72),
                ..section("s3", "", Some((12, 14)), 2)
            },
        ],
        ..record("p1", "Rondo")
    };
    store.save(&item);
    assert_eq!(store.item("p1").sections, item.sections);

    item.sections[0].name = "A".into();
    item.sections[2].bars = Some(BarRange {
        first: 12,
        last: 13,
    });
    store.save(&item);
    assert_eq!(store.item("p1").sections, item.sections);
    assert_eq!(
        store.raw("SELECT CAST(count(*) AS TEXT) FROM section"),
        Some("3".into())
    );
}

#[test]
fn links_and_a_tombstone_update_by_id_and_load_by_position() {
    let mut store = Store::in_memory().expect("opens");
    let mut item = Item {
        exercise_links: vec![
            ExerciseLink {
                id: "l1".into(),
                exercise_id: "e1".into(),
                section_id: None,
                position: 0,
                updated_at: at("2026-10-04T09:00:00Z"),
                deleted_at: None,
            },
            ExerciseLink {
                id: "l2".into(),
                exercise_id: "e2".into(),
                section_id: Some("s2".into()),
                position: 1,
                updated_at: at("2026-10-04T09:05:00Z"),
                deleted_at: Some(at("2026-10-04T09:05:00Z")),
            },
        ],
        ..record("p1", "Nocturne")
    };
    store.save(&item);
    assert_eq!(store.item("p1").exercise_links, item.exercise_links);

    item.exercise_links[0].position = 2;
    item.exercise_links[0].section_id = Some("s1".into());
    item.exercise_links[0].updated_at = at("2026-10-04T10:00:00Z");
    item.exercise_links[1].exercise_id = "e3".into();
    item.exercise_links[1].deleted_at = None;
    item.exercise_links[1].updated_at = at("2026-10-04T10:05:00Z");
    store.save(&item);
    let reversed: Vec<ExerciseLink> = item.exercise_links.iter().rev().cloned().collect();
    assert_eq!(store.item("p1").exercise_links, reversed);
    assert_eq!(
        store.raw("SELECT linked_exercise_ids FROM item WHERE id = 'p1'"),
        Some("[]".into())
    );
}

// ── More values the core cannot read (#2005, #2097, #2106, #2234) ──

#[test]
fn unreadable_tags_and_the_old_link_list_survive_until_the_tags_change() {
    let mut store = upgraded(
        "v17_item_metre",
        "INSERT INTO item
           (id, title, kind, tags, linked_exercise_ids, created_at, updated_at, priority)
         VALUES ('i1', 'X', 'piece', 'not json', '[1, 2]',
                 '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0)",
    );
    let raw = |store: &Store, column: &str| {
        store.raw(&format!("SELECT {column} FROM item WHERE id = 'i1'"))
    };
    let mut loaded = store.item("i1");
    loaded.title = "Renamed".into();
    store.save(&loaded);
    assert_eq!(raw(&store, "tags"), Some("not json".into()));
    assert_eq!(raw(&store, "linked_exercise_ids"), Some("[1, 2]".into()));

    loaded.tags = vec!["scales".into()];
    loaded.exercise_links = vec![ExerciseLink {
        id: "l1".into(),
        exercise_id: "e1".into(),
        section_id: None,
        position: 0,
        updated_at: at("2026-10-04T09:00:00Z"),
        deleted_at: None,
    }];
    store.save(&loaded);
    assert_eq!(raw(&store, "tags"), Some(r#"["scales"]"#.into()));
    assert_eq!(raw(&store, "linked_exercise_ids"), Some("[1, 2]".into()));
}

#[test]
fn a_real_chart_and_metre_replace_unreadable_ones() {
    let mut store = upgraded(
        "v17_item_metre",
        r#"INSERT INTO item
           (id, title, kind, tags, linked_exercise_ids, created_at, updated_at, priority,
            chord_chart, metre)
         VALUES ('i1', 'X', 'piece', '[]', '[]',
                 '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0,
                 '{"key":"C","sections":"torn"}', '{"beats":900,"unit":4}')"#,
    );
    let mut loaded = store.item("i1");
    let chart = ChordChart {
        key: Some(key(Letter::G, Accidental::Natural, Some(Modality::Minor))),
        sections: vec![],
    };
    let metre = Metre {
        beats: 3,
        unit: 4,
        groups: None,
    };
    loaded.chord_chart = Some(chart.clone());
    loaded.metre = Some(metre.clone());
    store.save(&loaded);

    let stored_chart = store.raw("SELECT chord_chart FROM item WHERE id = 'i1'");
    let stored_metre = store.raw("SELECT metre FROM item WHERE id = 'i1'");
    let mut unreadable = vec![];
    assert_eq!(
        codec::decode_chord_chart(stored_chart.as_deref(), "i1", &mut unreadable),
        Some(chart)
    );
    assert_eq!(
        codec::decode_metre(stored_metre.as_deref(), "i1", &mut unreadable),
        Some(metre)
    );
    assert_eq!(unreadable, Vec::<String>::new());
}

#[test]
fn picking_keys_replaces_a_list_and_a_chart_key_the_core_cannot_read() {
    let mut store = upgraded(
        "v19_keys_and_variations",
        r#"INSERT INTO item
           (id, title, kind, tags, linked_exercise_ids, created_at, updated_at, priority,
            chord_chart, keys)
         VALUES ('i1', 'X', 'piece', '[]', '[]',
                 '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0,
                 '{"key":"H","modality":"dorian","sections":[]}',
                 '[{"key":"Eb","modality":"major"},{"key":"H","modality":"dorian"}]')"#,
    );
    let mut loaded = store.item("i1");
    let keys = vec![key(Letter::C, Accidental::Natural, Some(Modality::Major))];
    let g_minor = key(Letter::G, Accidental::Natural, Some(Modality::Minor));
    loaded.keys = keys.clone();
    loaded.chord_chart.as_mut().expect("a chart").key = Some(g_minor);
    store.save(&loaded);

    let stored_keys = store.raw("SELECT keys FROM item WHERE id = 'i1'");
    let stored_chart = store.raw("SELECT chord_chart FROM item WHERE id = 'i1'");
    let mut unreadable = vec![];
    assert_eq!(
        codec::decode_keys(stored_keys.as_deref(), "i1", &mut unreadable),
        keys
    );
    assert_eq!(
        codec::decode_chord_chart(stored_chart.as_deref(), "i1", &mut unreadable)
            .and_then(|c| c.key),
        Some(g_minor)
    );
}

#[test]
fn a_refused_session_is_skipped_and_a_replaced_status_is_named() {
    let mut store = upgraded(
        migrations::latest(),
        "INSERT INTO session (id, started_at, completed_at, total_duration_secs,
           completion_status, session_notes, session_intention, entries, updated_at, deleted_at)
         VALUES ('bad','refused-2234','2026-09-01T10:10:00Z',600,'completed',NULL,NULL,'[]',
           '2026-09-01T10:10:00Z',NULL),
           ('good','2026-09-01T10:00:00Z','2026-09-01T10:10:00Z',600,'replaced-2234',NULL,NULL,
           '[]','2026-09-01T10:10:00Z',NULL)",
    );
    let answer = store.handle(&PersistenceOperation::LoadSessions);
    let PersistenceOutput::Sessions(sessions) = answer.output else {
        panic!("sessions")
    };
    assert_eq!(
        sessions.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        vec!["good"]
    );
    assert!(
        answer.unreadable.iter().any(|u| u.contains("refused-2234")),
        "{:?}",
        answer.unreadable
    );
    assert!(
        answer
            .unreadable
            .contains(&r#"unknown CompletionStatus on decode: "replaced-2234""#.to_string()),
        "{:?}",
        answer.unreadable
    );
}

// ── Stored strings (StoredStringsTests.swift) ──

#[test]
fn item_kinds_and_modalities_keep_their_stored_text() {
    let mut unreadable = vec![];
    for (kind, text) in [(ItemKind::Piece, "piece"), (ItemKind::Exercise, "exercise")] {
        assert_eq!(codec::item_kind_text(&kind), text);
        assert_eq!(codec::item_kind(text, &mut unreadable), kind);
    }
    for (mode, text) in [(Modality::Major, "major"), (Modality::Minor, "minor")] {
        let c = key(Letter::C, Accidental::Natural, Some(mode));
        assert_eq!(codec::stored_key(&c).modality.as_deref(), Some(text));
        assert_eq!(codec::key(Some("C"), Some(text), &mut unreadable), Some(c));
    }
    assert_eq!(unreadable, Vec::<String>::new());
}

#[test]
fn chord_qualities_keep_their_stored_text() {
    let qualities = [
        (ChordQuality::Maj7, "maj7"),
        (ChordQuality::Dom7, "dom7"),
        (ChordQuality::Min7, "min7"),
        (ChordQuality::Min7b5, "min7b5"),
        (ChordQuality::Dim7, "dim7"),
        (ChordQuality::MinMaj7, "minMaj7"),
        (ChordQuality::Six, "six"),
        (ChordQuality::Min6, "min6"),
        (ChordQuality::Alt, "alt"),
        (ChordQuality::Sus4, "sus4"),
        (ChordQuality::Sus2, "sus2"),
        (ChordQuality::Aug, "aug"),
        (ChordQuality::Dom7Sharp5, "dom7Sharp5"),
        (ChordQuality::Other, "other"),
    ];
    let chart = ChordChart {
        key: None,
        sections: vec![ChartSection {
            label: None,
            bars: vec![Bar {
                chords: qualities
                    .iter()
                    .map(|(quality, text)| ChartChord {
                        symbol: ChordSymbol {
                            root: 0,
                            quality: *quality,
                            extensions: vec![],
                            bass: None,
                            raw: (*text).into(),
                        },
                    })
                    .collect(),
            }],
        }],
    };
    let json = codec::encode_chord_chart(Some(&chart), None)
        .expect("encodes")
        .expect("a chart");
    let stored: serde_json::Value = serde_json::from_str(&json).expect("json");
    let written: Vec<&str> = stored["sections"][0]["bars"][0]["chords"]
        .as_array()
        .expect("chords")
        .iter()
        .map(|c| c["symbol"]["quality"].as_str().expect("text"))
        .collect();
    assert_eq!(
        written,
        qualities.iter().map(|(_, text)| *text).collect::<Vec<_>>()
    );
    let mut unreadable = vec![];
    assert_eq!(
        codec::decode_chord_chart(Some(&json), "p1", &mut unreadable),
        Some(chart)
    );
    assert_eq!(unreadable, Vec::<String>::new());
}

#[test]
fn near_miss_text_is_refused_as_a_kind_and_as_a_modality() {
    for raw in ["", "Piece", "ended-early", "notAttempted", " major"] {
        let mut unreadable = vec![];
        assert_eq!(codec::item_kind(raw, &mut unreadable), ItemKind::Piece);
        assert_eq!(
            unreadable,
            vec![format!("unknown ItemKind on decode: \"{raw}\"")]
        );

        let mut unreadable = vec![];
        assert_eq!(
            codec::key(Some("C"), Some(raw), &mut unreadable),
            Some(key(Letter::C, Accidental::Natural, None))
        );
        assert_eq!(
            unreadable,
            vec![format!("unknown Modality on decode: \"{raw}\"")]
        );
    }
}

// ── Sync merge (#2354) ──

fn stored_records(store: &mut Store, keys: Option<Vec<RecordKey>>) -> StoredRecords {
    let operation = match keys {
        Some(keys) => PersistenceOperation::LoadRecords(keys),
        None => PersistenceOperation::LoadAllRecords,
    };
    match store.ok(&operation) {
        PersistenceOutput::Records(records) => records,
        other => panic!("expected records, got {other:?}"),
    }
}

fn item_key(id: &str) -> RecordKey {
    RecordKey {
        kind: RecordKind::Item,
        id: id.into(),
    }
}

#[test]
fn a_merge_load_sees_deleted_pieces_and_only_the_keys_asked_for() {
    let mut store = Store::in_memory().expect("opens");
    store.save(&record("kept", "Kept"));
    store.save(&record("gone", "Gone"));
    store.save(&record("other", "Other"));
    store.ok(&PersistenceOperation::DeleteItem {
        id: "gone".into(),
        deleted_at: at("2026-02-01T00:00:00Z"),
    });

    let records = stored_records(&mut store, Some(vec![item_key("kept"), item_key("gone")]));

    let mut found: Vec<(String, Option<DateTime<Utc>>)> = records
        .items
        .into_iter()
        .map(|s| (s.item.id, s.deleted_at))
        .collect();
    found.sort();
    assert_eq!(
        found,
        vec![
            ("gone".to_string(), Some(at("2026-02-01T00:00:00Z"))),
            ("kept".to_string(), None),
        ]
    );
}

#[test]
fn a_merged_tombstone_is_stored_even_for_a_piece_never_seen_here() {
    let mut store = Store::in_memory().expect("opens");
    let deleted_at = at("2026-02-01T00:00:00Z");
    let merged = StoredRecords {
        items: vec![StoredItem {
            item: record("theirs", "Theirs"),
            deleted_at: Some(deleted_at),
        }],
        ..Default::default()
    };
    assert_eq!(
        store.ok(&PersistenceOperation::ApplyMerged(merged)),
        PersistenceOutput::Ack
    );

    assert!(store.items().is_empty());
    let all = stored_records(&mut store, None);
    assert_eq!(all.items[0].deleted_at, Some(deleted_at));
}

#[test]
fn a_merged_edit_brings_back_a_piece_deleted_earlier_here() {
    let mut store = Store::in_memory().expect("opens");
    store.save(&record("piece", "Old"));
    store.ok(&PersistenceOperation::DeleteItem {
        id: "piece".into(),
        deleted_at: at("2026-02-01T00:00:00Z"),
    });
    let merged = StoredRecords {
        items: vec![StoredItem {
            item: record("piece", "Newer"),
            deleted_at: None,
        }],
        ..Default::default()
    };
    store.ok(&PersistenceOperation::ApplyMerged(merged));

    assert_eq!(store.item("piece").title, "Newer");
}

#[test]
fn a_merge_writes_variations_and_sessions_in_one_go() {
    let mut store = Store::in_memory().expect("opens");
    let variation = Variation {
        id: "v".into(),
        label: "Slow".into(),
        updated_at: at(EARLY),
        deleted_at: Some(at("2026-02-01T00:00:00Z")),
    };
    let merged = StoredRecords {
        items: vec![],
        variations: vec![variation.clone()],
        sessions: vec![session("s", "2026-02-02T00:00:00Z")],
        unreadable: vec![],
    };
    store.ok(&PersistenceOperation::ApplyMerged(merged));

    assert_eq!(store.variations(), vec![variation]);
    assert_eq!(store.sessions()[0].id, "s");
}

fn section_row(id: &str, deleted_at: Option<&str>) -> ItemSection {
    ItemSection {
        id: id.into(),
        name: id.into(),
        bars: None,
        kind: SectionKind::Form,
        target_bpm: None,
        position: 0,
        updated_at: at(EARLY),
        deleted_at: deleted_at.map(at),
    }
}

fn link_row(id: &str) -> ExerciseLink {
    ExerciseLink {
        id: id.into(),
        exercise_id: "e1".into(),
        section_id: None,
        position: 0,
        updated_at: at(EARLY),
        deleted_at: None,
    }
}

#[test]
fn a_winning_piece_deletes_the_sections_and_links_only_this_device_added() {
    let earlier = "2026-01-15T00:00:00Z";
    let won = "2026-03-01T00:00:00Z";
    let deleted = "2026-04-01T00:00:00Z";
    for (case, deleted_at, expected) in [
        ("a live winner", None, won),
        ("a deleted winner", Some(deleted), deleted),
    ] {
        let mut store = Store::in_memory().expect("opens");
        store.save(&Item {
            sections: vec![
                section_row("both", None),
                section_row("added here", None),
                section_row("gone here", Some(earlier)),
            ],
            exercise_links: vec![link_row("both"), link_row("added here")],
            ..record("piece", "Ours")
        });
        let winner = Item {
            updated_at: at(won),
            sections: vec![section_row("both", None)],
            exercise_links: vec![link_row("both")],
            ..record("piece", "Theirs")
        };
        store.ok(&PersistenceOperation::ApplyMerged(StoredRecords {
            items: vec![StoredItem {
                item: winner,
                deleted_at: deleted_at.map(at),
            }],
            ..Default::default()
        }));

        let stored = stored_records(&mut store, None).items.remove(0).item;
        let sections: Vec<(String, Option<DateTime<Utc>>)> = stored
            .sections
            .into_iter()
            .map(|s| (s.id, s.deleted_at))
            .collect();
        let links: Vec<(String, Option<DateTime<Utc>>)> = stored
            .exercise_links
            .into_iter()
            .map(|l| (l.id, l.deleted_at))
            .collect();
        assert_eq!(
            sections,
            vec![
                ("added here".to_string(), Some(at(expected))),
                ("both".to_string(), None),
                ("gone here".to_string(), Some(at(earlier))),
            ],
            "{case}"
        );
        assert_eq!(
            links,
            vec![
                ("added here".to_string(), Some(at(expected))),
                ("both".to_string(), None),
            ],
            "{case}"
        );
    }
}

#[test]
fn a_merge_load_names_the_rows_it_could_not_read() {
    let mut store = upgraded(
        migrations::latest(),
        &format!(
            "INSERT INTO item (id, title, kind, tags, created_at, updated_at)
             VALUES ('bad', 'Bad', 'piece', '[]', 'last tuesday', '{EARLY}');
             INSERT INTO item (id, title, kind, tags, created_at, updated_at)
             VALUES ('not asked', 'Bad', 'piece', '[]', 'last tuesday', '{EARLY}');
             INSERT INTO item (id, title, kind, tags, created_at, updated_at)
             VALUES ('good', 'Good', 'piece', '[]', '{EARLY}', '{EARLY}');
             INSERT INTO item (id, title, kind, tags, created_at, updated_at)
             VALUES ('partial', 'Partial', 'piece', '[]', '{EARLY}', '{EARLY}');
             INSERT INTO section (id, item_id, name, kind, position, updated_at)
             VALUES ('s', 'partial', 'A', 'form', 0, 'never');
             INSERT INTO item (id, title, kind, tags, created_at, updated_at)
             VALUES ('linked', 'Linked', 'piece', '[]', '{EARLY}', '{EARLY}');
             INSERT INTO exercise_link (id, piece_id, exercise_id, position, updated_at)
             VALUES ('l', 'linked', 'e1', 0, 'never');
             INSERT INTO variation (id, label, updated_at) VALUES ('v', 'Slow', 'never');
             INSERT INTO session (id, started_at, completed_at, total_duration_secs,
               completion_status, entries, updated_at)
             VALUES ('s1', 'yesterday', '2026-09-01T10:10:00Z', 600, 'completed', '[]',
               '2026-09-01T10:10:00Z');"
        ),
    );
    let key = |kind, id: &str| RecordKey {
        kind,
        id: id.into(),
    };
    let records = stored_records(
        &mut store,
        Some(vec![
            item_key("bad"),
            item_key("good"),
            item_key("partial"),
            item_key("linked"),
            key(RecordKind::Variation, "v"),
            key(RecordKind::Session, "s1"),
        ]),
    );
    let mut unreadable = records.unreadable;
    unreadable.sort();
    assert_eq!(
        unreadable,
        vec![
            item_key("bad"),
            item_key("linked"),
            item_key("partial"),
            key(RecordKind::Variation, "v"),
            key(RecordKind::Session, "s1"),
        ]
    );
}
