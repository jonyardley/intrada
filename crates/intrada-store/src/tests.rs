//! Seeds are the shapes the iPhone's GRDB store wrote, copied from
//! `LibraryStoreMigrationTests` and `LibraryStoreTests`, never written to match
//! this port (#1256).

use chrono::{DateTime, Utc};
use intrada_core::domain::chart::{
    Bar, ChartChord, ChartSection, ChordChart, ChordQuality, ChordSymbol,
};
use intrada_core::domain::link::ExerciseLink;
use intrada_core::domain::section::{BarRange, ItemSection, SectionKind};
use intrada_core::domain::{Metre, Variation};
use intrada_core::{
    Accidental, Item, ItemKind, Key, Letter, Modality, PersistenceOperation, PersistenceOutput,
    PracticeSession, Tempo,
};
use rusqlite::Connection;

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

// The iPhone keeps adding migrations until it switches over (#2432); one added
// there and not here would leave Android's schema behind with every gate green.
#[test]
fn the_migrations_match_the_iphones_in_order() {
    let swift = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../ios/Intrada/Core/LibraryMigrations.swift"
    ))
    .expect("reads the iPhone's migrations");
    let iphone: Vec<&str> = swift
        .split("registerMigration(\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .collect();
    let here: Vec<&str> = MIGRATIONS.iter().map(|(id, _)| *id).collect();
    assert_eq!(here, iphone);
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

    let session = store.sessions().pop().expect("a session");
    assert_eq!(session.capture_version, None);
    assert_eq!(session.entries[0].plays[0].score, Some(7));
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

// ── A database the iPhone app wrote (SharedStoreFixtureTests.swift) ──

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
