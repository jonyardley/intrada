// ── Migrations ───────────────────────────────────────────────────────
// GRDB's own table, so a database the iPhone app wrote carries on (#2421).
// Append-only: never edit or delete a shipped one (.claude/rules/offline-first.md).

use std::collections::HashSet;

use rusqlite::{Connection, Transaction, TransactionBehavior};

use crate::codec::Unreadable;

pub(crate) enum Step {
    Sql(&'static str),
    Code(fn(&Transaction, &mut Unreadable) -> rusqlite::Result<()>),
}

pub(crate) const MIGRATIONS: &[(&str, Step)] = &[
    (
        "v1_item",
        Step::Sql(
            "CREATE TABLE item (
               id TEXT PRIMARY KEY NOT NULL,
               title TEXT NOT NULL,
               kind TEXT NOT NULL,
               composer TEXT,
               key TEXT,
               tempo_marking TEXT,
               tempo_bpm INTEGER,
               notes TEXT,
               tags TEXT NOT NULL DEFAULT '[]',
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               priority INTEGER NOT NULL DEFAULT 0,
               deleted_at TEXT
             )",
        ),
    ),
    (
        "v2_add_modality",
        Step::Sql("ALTER TABLE item ADD COLUMN modality TEXT"),
    ),
    (
        "v3_session",
        Step::Sql(
            "CREATE TABLE session (
               id TEXT PRIMARY KEY NOT NULL,
               started_at TEXT NOT NULL,
               completed_at TEXT NOT NULL,
               total_duration_secs INTEGER NOT NULL,
               completion_status TEXT NOT NULL,
               session_notes TEXT,
               session_intention TEXT,
               entries TEXT NOT NULL DEFAULT '[]',
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             )",
        ),
    ),
    (
        "v4_session_score",
        Step::Sql("ALTER TABLE session ADD COLUMN session_score INTEGER"),
    ),
    ("v5_rescale_entry_scores", Step::Code(rescale_entry_scores)),
    (
        "v6_item_linked_exercises",
        Step::Sql("ALTER TABLE item ADD COLUMN linked_exercise_ids TEXT NOT NULL DEFAULT '[]'"),
    ),
    (
        "v7_session_reflections",
        Step::Sql(
            "ALTER TABLE session ADD COLUMN reflection_improved TEXT;
             ALTER TABLE session ADD COLUMN reflection_still_rough TEXT;
             ALTER TABLE session ADD COLUMN reflection_next_target TEXT;",
        ),
    ),
    (
        "v8_item_chord_chart",
        Step::Sql("ALTER TABLE item ADD COLUMN chord_chart TEXT"),
    ),
    (
        "v9_variant",
        Step::Sql(
            "CREATE TABLE variant (
               id TEXT PRIMARY KEY NOT NULL,
               item_id TEXT NOT NULL,
               label TEXT NOT NULL,
               position INTEGER NOT NULL,
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             );
             CREATE INDEX index_variant_on_item_id ON variant(item_id);",
        ),
    ),
    (
        "v10_coach_records",
        Step::Sql(
            "CREATE TABLE block_record (
               id TEXT PRIMARY KEY NOT NULL,
               node TEXT NOT NULL,
               drill TEXT NOT NULL,
               gate TEXT NOT NULL,
               level_tempo_bpm INTEGER NOT NULL,
               level_click_level TEXT NOT NULL,
               circle TEXT NOT NULL,
               mode TEXT NOT NULL,
               started_at TEXT NOT NULL,
               ended_at TEXT NOT NULL,
               attempts TEXT NOT NULL DEFAULT '[]',
               attempts_to_pass INTEGER,
               gate_opened_at_attempt INTEGER,
               reps_after_gate INTEGER NOT NULL,
               active_ms INTEGER NOT NULL,
               escalation_fired TEXT NOT NULL DEFAULT '[]',
               exit TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             );
             CREATE TABLE wander_record (
               id TEXT PRIMARY KEY NOT NULL,
               started_at TEXT NOT NULL,
               ended_at TEXT NOT NULL,
               attempts TEXT NOT NULL DEFAULT '[]',
               keep_as_drill INTEGER,
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             );",
        ),
    ),
    (
        "v11_built_session",
        Step::Sql(
            "CREATE TABLE user_drill (
               id TEXT PRIMARY KEY NOT NULL,
               name TEXT NOT NULL,
               criterion TEXT NOT NULL,
               tempo_bpm INTEGER,
               keys TEXT NOT NULL DEFAULT '[]',
               passes_to_open INTEGER NOT NULL,
               serves_kind TEXT,
               serves_value TEXT,
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             );
             CREATE TABLE journal_item (
               id TEXT PRIMARY KEY NOT NULL,
               name TEXT NOT NULL,
               notes TEXT,
               linked_item_id TEXT,
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             );
             CREATE TABLE built_session (
               id TEXT PRIMARY KEY NOT NULL,
               source TEXT,
               blocks TEXT NOT NULL DEFAULT '[]',
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             );
             CREATE TABLE play_through (
               id TEXT PRIMARY KEY NOT NULL,
               item_id TEXT NOT NULL,
               started_at TEXT NOT NULL,
               ended_at TEXT NOT NULL,
               counted INTEGER NOT NULL,
               sections TEXT NOT NULL DEFAULT '[]',
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             );
             CREATE TABLE reflection (
               id TEXT PRIMARY KEY NOT NULL,
               kind TEXT NOT NULL,
               session_ref TEXT,
               transcript TEXT,
               audio_path TEXT,
               duration_s INTEGER,
               at TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             );
             CREATE TABLE feel_entry (
               id TEXT PRIMARY KEY NOT NULL,
               block_id TEXT NOT NULL,
               feel TEXT NOT NULL,
               at TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             );",
        ),
    ),
    (
        "v12_block_origin",
        Step::Sql("ALTER TABLE block_record ADD COLUMN origin TEXT NOT NULL DEFAULT 'authored'"),
    ),
    (
        "v13_wander_item",
        Step::Sql("ALTER TABLE wander_record ADD COLUMN item_id TEXT"),
    ),
    (
        "v14_unmonitored_play",
        Step::Sql(
            "CREATE TABLE unmonitored_play (
               id TEXT PRIMARY KEY NOT NULL,
               started_at TEXT NOT NULL,
               ended_at TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             )",
        ),
    ),
    (
        "v15_reflection_steer",
        Step::Sql(
            "ALTER TABLE reflection ADD COLUMN steer TEXT NOT NULL DEFAULT 'unoffered';
             ALTER TABLE reflection ADD COLUMN steer_at TEXT;",
        ),
    ),
    (
        "v16_item_photo",
        Step::Sql("ALTER TABLE item ADD COLUMN photo_id TEXT"),
    ),
    (
        "v17_item_metre",
        Step::Sql(
            "ALTER TABLE item ADD COLUMN metre TEXT;
             UPDATE item
             SET metre = json_object('beats', json_extract(chord_chart, '$.metre'), 'unit', 4)
             WHERE chord_chart IS NOT NULL AND json_extract(chord_chart, '$.metre') IS NOT NULL;",
        ),
    ),
    (
        "v18_section",
        Step::Sql(
            "CREATE TABLE section (
               id TEXT PRIMARY KEY NOT NULL,
               item_id TEXT NOT NULL,
               name TEXT NOT NULL,
               bar_first INTEGER,
               bar_last INTEGER,
               kind TEXT NOT NULL,
               target_bpm INTEGER,
               position INTEGER NOT NULL,
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             );
             CREATE INDEX index_section_on_item_id ON section(item_id);",
        ),
    ),
    (
        "v19_keys_and_variations",
        Step::Sql(
            "CREATE TABLE variation (
               id TEXT PRIMARY KEY NOT NULL,
               label TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             );
             ALTER TABLE item ADD COLUMN variation_ids TEXT;
             ALTER TABLE item ADD COLUMN keys TEXT;
             ALTER TABLE session ADD COLUMN capture_version INTEGER;",
        ),
    ),
    (
        "v20_exercise_link",
        Step::Sql(
            "CREATE TABLE exercise_link (
               id TEXT PRIMARY KEY NOT NULL,
               piece_id TEXT NOT NULL,
               exercise_id TEXT NOT NULL,
               section_id TEXT,
               position INTEGER NOT NULL,
               updated_at TEXT NOT NULL,
               deleted_at TEXT
             );
             CREATE INDEX index_exercise_link_on_piece_id ON exercise_link(piece_id);
             INSERT OR IGNORE INTO exercise_link
               (id, piece_id, exercise_id, section_id, position, updated_at, deleted_at)
             SELECT 'link|' || item.id || '|' || j.value, item.id, j.value, NULL, MIN(j.key),
               item.updated_at, NULL
             FROM item, json_each(
               CASE WHEN json_valid(item.linked_exercise_ids) THEN
                 CASE WHEN json_type(item.linked_exercise_ids) = 'array'
                 THEN item.linked_exercise_ids END
               END) AS j
             WHERE j.type = 'text'
             GROUP BY item.id, j.value;",
        ),
    ),
];

pub(crate) fn latest() -> &'static str {
    MIGRATIONS[MIGRATIONS.len() - 1].0
}

/// Up to and including `target`.
pub(crate) fn migrate(
    conn: &mut Connection,
    target: &str,
    unreadable: &mut Unreadable,
) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS grdb_migrations (identifier TEXT NOT NULL PRIMARY KEY)",
    )?;
    let applied: HashSet<String> = conn
        .prepare("SELECT identifier FROM grdb_migrations")?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for (id, step) in MIGRATIONS {
        if !applied.contains(*id) {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            match step {
                Step::Sql(sql) => tx.execute_batch(sql)?,
                Step::Code(run) => run(&tx, unreadable)?,
            }
            tx.execute("INSERT INTO grdb_migrations (identifier) VALUES (?1)", [id])?;
            tx.commit()?;
        }
        if *id == target {
            break;
        }
    }
    Ok(())
}

/// Rescales in SQL, never through the session codec: decoding with today's
/// codec changed this shipped migration whenever the codec did (#1947). One
/// json_set per score, since json_group_array does not promise array order.
/// A row that is not a JSON array is skipped and named (#2019).
fn rescale_entry_scores(tx: &Transaction, unreadable: &mut Unreadable) -> rusqlite::Result<()> {
    let rows: Vec<(String, String, Option<bool>)> = tx
        .prepare(
            "SELECT id, entries,
               CASE WHEN json_valid(entries) THEN json_type(entries) END = 'array' AS readable
             FROM session",
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    for (id, entries, readable) in rows {
        if readable != Some(true) {
            unreadable.push(format!("v5_rescale_entry_scores skipped row {id}"));
            continue;
        }
        let paths: Vec<String> = tx
            .prepare(
                "SELECT fullkey || '.score' FROM json_each(?1)
                 WHERE json_type(?1, fullkey || '.score') = 'integer'",
            )?
            .query_map([&entries], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        for path in paths {
            tx.execute(
                "UPDATE session
                 SET entries = json_set(entries, ?1, min(10, json_extract(entries, ?1) * 2))
                 WHERE id = ?2",
                [&path, &id],
            )?;
        }
    }
    Ok(())
}
