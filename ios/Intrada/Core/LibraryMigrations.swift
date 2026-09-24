import GRDB

extension LibraryStore {
  static let migrator: DatabaseMigrator = {
    var migrator = DatabaseMigrator()
    migrator.registerMigration("v1_item") { db in
      try db.execute(
        sql: """
          CREATE TABLE item (
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
          )
          """)
    }
    migrator.registerMigration("v2_add_modality") { db in
      try db.execute(sql: "ALTER TABLE item ADD COLUMN modality TEXT")
    }
    migrator.registerMigration("v3_session") { db in
      try db.execute(
        sql: """
          CREATE TABLE session (
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
          )
          """)
    }
    migrator.registerMigration("v4_session_score") { db in
      try db.execute(sql: "ALTER TABLE session ADD COLUMN session_score INTEGER")
    }
    // Rescales in SQL, never through StoredEntry: decoding with today's codec
    // changed this shipped migration whenever the codec did (#1947). One
    // json_set per score, since json_group_array does not promise array order.
    migrator.registerMigration("v5_rescale_entry_scores") { db in
      let rows = try Row.fetchAll(
        db,
        sql: """
          SELECT id, entries,
            CASE WHEN json_valid(entries) THEN json_type(entries) END = 'array' AS readable
          FROM session
          """)
      for row in rows {
        let id: String = row["id"]
        let entries: String = row["entries"]
        guard row["readable"] as Bool? == true else {
          report(StoredCodecError(field: "entries"), "LibraryStore v5 rescale")
          continue
        }
        let scorePaths = try String.fetchAll(
          db,
          sql: """
            SELECT fullkey || '.score' FROM json_each(?)
            WHERE json_type(?, fullkey || '.score') = 'integer'
            """, arguments: [entries, entries])
        for path in scorePaths {
          try db.execute(
            sql: """
              UPDATE session SET entries = json_set(entries, ?, min(10, json_extract(entries, ?) * 2))
              WHERE id = ?
              """, arguments: [path, path, id])
        }
      }
    }
    migrator.registerMigration("v6_item_linked_exercises") { db in
      try db.execute(
        sql: "ALTER TABLE item ADD COLUMN linked_exercise_ids TEXT NOT NULL DEFAULT '[]'")
    }
    migrator.registerMigration("v7_session_reflections") { db in
      try db.execute(sql: "ALTER TABLE session ADD COLUMN reflection_improved TEXT")
      try db.execute(sql: "ALTER TABLE session ADD COLUMN reflection_still_rough TEXT")
      try db.execute(sql: "ALTER TABLE session ADD COLUMN reflection_next_target TEXT")
    }
    migrator.registerMigration("v8_item_chord_chart") { db in
      // Nullable JSON column; NULL = no chart. Additive, non-destructive.
      try db.execute(sql: "ALTER TABLE item ADD COLUMN chord_chart TEXT")
    }
    migrator.registerMigration("v9_variant") { db in
      // Exercise variation ladders (#1083). First normalized child table: per-row
      // `updated_at`/`deleted_at` give the future sync engine per-variation LWW +
      // tombstones (invariant 2) that a JSON blob on `item` couldn't. Additive:
      // existing exercises simply have no rows here (an empty ladder).
      try db.execute(
        sql: """
          CREATE TABLE variant (
            id TEXT PRIMARY KEY NOT NULL,
            item_id TEXT NOT NULL,
            label TEXT NOT NULL,
            position INTEGER NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT
          )
          """)
      try db.execute(sql: "CREATE INDEX index_variant_on_item_id ON variant(item_id)")
    }
    migrator.registerMigration("v10_coach_records") { db in
      // Drill-loop evidence (#1181, spec §4), written as each record closes.
      // `attempts` is a JSON blob for the same reason `session.entries` is.
      try db.execute(
        sql: """
          CREATE TABLE block_record (
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
          )
          """)
      try db.execute(
        sql: """
          CREATE TABLE wander_record (
            id TEXT PRIMARY KEY NOT NULL,
            started_at TEXT NOT NULL,
            ended_at TEXT NOT NULL,
            attempts TEXT NOT NULL DEFAULT '[]',
            keep_as_drill INTEGER,
            updated_at TEXT NOT NULL,
            deleted_at TEXT
          )
          """)
    }
    migrator.registerMigration("v11_built_session") { db in
      // Self-directed practice (#1256, coach era, removed #1344; table kept
      // as a dead migration per the append-only rule). `blocks` and
      // `sections` are JSON blobs for the same reason `session.entries` is:
      // ordered child docs edited as one document, not independently synced rows.
      try db.execute(
        sql: """
          CREATE TABLE user_drill (
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
          )
          """)
      try db.execute(
        sql: """
          CREATE TABLE journal_item (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            notes TEXT,
            linked_item_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT
          )
          """)
      try db.execute(
        sql: """
          CREATE TABLE built_session (
            id TEXT PRIMARY KEY NOT NULL,
            source TEXT,
            blocks TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT
          )
          """)
      try db.execute(
        sql: """
          CREATE TABLE play_through (
            id TEXT PRIMARY KEY NOT NULL,
            item_id TEXT NOT NULL,
            started_at TEXT NOT NULL,
            ended_at TEXT NOT NULL,
            counted INTEGER NOT NULL,
            sections TEXT NOT NULL DEFAULT '[]',
            updated_at TEXT NOT NULL,
            deleted_at TEXT
          )
          """)
      try db.execute(
        sql: """
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
          )
          """)
      try db.execute(
        sql: """
          CREATE TABLE feel_entry (
            id TEXT PRIMARY KEY NOT NULL,
            block_id TEXT NOT NULL,
            feel TEXT NOT NULL,
            at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT
          )
          """)
    }
    migrator.registerMigration("v12_block_origin") { db in
      // #1256: whose block this was. A record written before built sessions
      // existed can only have been the planner's, so 'authored' is the honest
      // backfill, and it is what the mastery rebuild reads to decide whether a
      // block's taps were ever evidence (decision 17).
      try db.execute(
        sql: "ALTER TABLE block_record ADD COLUMN origin TEXT NOT NULL DEFAULT 'authored'")
    }
    migrator.registerMigration("v13_wander_item") { db in
      // #1256 Phase C: off-piste is now reachable from a piece (B0), so a
      // wander can name what it was played against. Nullable, because the
      // mid-session door has no piece behind it and every existing row came
      // through it.
      try db.execute(sql: "ALTER TABLE wander_record ADD COLUMN item_id TEXT")
    }
    migrator.registerMigration("v14_unmonitored_play") { db in
      // #1285: its own table rather than a discriminated `wander_record`,
      // because a table with no column for a piece cannot be made to name one:
      // decision 7's "minutes only", enforced by the schema rather than stated.
      try db.execute(
        sql: """
          CREATE TABLE unmonitored_play (
            id TEXT PRIMARY KEY NOT NULL,
            started_at TEXT NOT NULL,
            ended_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT
          )
          """)
    }
    migrator.registerMigration("v15_reflection_steer") { db in
      // #1256 Phase D: what became of the morning proposal a reflection earned
      // (C3). 'unoffered' is the honest backfill (a reflection written before
      // the card existed was never offered one), and it is also what stops a
      // reflection from last month arriving as this morning's steer on upgrade.
      try db.execute(
        sql: "ALTER TABLE reflection ADD COLUMN steer TEXT NOT NULL DEFAULT 'unoffered'")
      try db.execute(sql: "ALTER TABLE reflection ADD COLUMN steer_at TEXT")
    }
    migrator.registerMigration("v16_item_photo") { db in
      // Id only: a BLOB would fatten every row of a table `loadItems` reads
      // whole (specs/piece-from-photo.md, key decision 1).
      try db.execute(sql: "ALTER TABLE item ADD COLUMN photo_id TEXT")
    }
    migrator.registerMigration("v17_item_metre") { db in
      try db.execute(sql: "ALTER TABLE item ADD COLUMN metre TEXT")
      // The chart carried beats-per-bar alone; a charted piece keeps that as a
      // crotchet-unit metre. The chart's own copy stays in the JSON, ignored:
      // ignoring is cheaper and safer than a copy-table drop (CLAUDE.md,
      // local data migrations).
      try db.execute(
        sql: """
          UPDATE item
          SET metre = json_object('beats', json_extract(chord_chart, '$.metre'), 'unit', 4)
          WHERE chord_chart IS NOT NULL AND json_extract(chord_chart, '$.metre') IS NOT NULL
          """)
    }
    return migrator
  }()
}
