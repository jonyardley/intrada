import Foundation
import GRDB
import SharedTypes

/// Persistence ops the Store resolves against — a protocol so tests can inject a failing fake (#816).
protocol ItemStore: Sendable {
  func loadItems() throws -> [Item]
  func save(_ item: Item) throws
  /// Upsert a batch in one transaction — all rows land or none do (#1106).
  func save(_ items: [Item]) throws
  func delete(id: String, deletedAt: String) throws
  func loadSessions() throws -> [PracticeSession]
  func saveSession(_ session: PracticeSession) throws
  func loadVariations() throws -> [Variation]
  /// Upsert a batch in one transaction.
  func save(_ variations: [Variation]) throws
}

/// On-device SQLite store (GRDB) — the B2 local-first persistence layer the
/// `Effect.persistence` operations resolve against. The schema is deliberately
/// **sync-agnostic**: every row carries `updated_at` + a soft-delete tombstone
/// so a later sync engine (custom LWW or Automerge) can sit on top without a
/// migration (see specs/native-ios.md "Sync engine").
final class LibraryStore: ItemStore {
  private let dbQueue: DatabaseQueue

  init(_ dbQueue: DatabaseQueue) throws {
    self.dbQueue = dbQueue
    try Self.migrator.migrate(dbQueue)
  }

  /// File-backed store in Application Support (the real app).
  static func onDisk() throws -> LibraryStore {
    let dir = try FileManager.default.url(
      for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
    return try LibraryStore(DatabaseQueue(path: dir.appendingPathComponent("intrada.sqlite").path))
  }

  /// In-memory store for tests/previews.
  static func inMemory() throws -> LibraryStore {
    try LibraryStore(DatabaseQueue())
  }

  // ── Operations ───────────────────────────────────────────────────────

  func loadItems() throws -> [Item] {
    try dbQueue.read { db in
      let sectionsByItem = try Self.sectionsByItem(db)
      let linksByPiece = try Self.linksByPiece(db)
      return try Row.fetchAll(
        db, sql: "SELECT * FROM item WHERE deleted_at IS NULL ORDER BY created_at DESC"
      )
      .map { row in
        Self.item(
          from: row, sections: sectionsByItem[row["id"]] ?? [],
          links: linksByPiece[row["id"]] ?? [])
      }
    }
  }

  /// Insert or update by id; clears any tombstone (an upsert revives a row).
  func save(_ item: Item) throws {
    try dbQueue.write { db in
      try Self.upsert(item, in: db)
    }
  }

  /// Batch upsert in a single transaction — the chart-to-scaffold commit writes
  /// N exercises + the piece all-or-nothing, so a mid-batch failure never
  /// orphans exercises against a half-linked piece (#1106, invariant 5).
  func save(_ items: [Item]) throws {
    try dbQueue.write { db in
      for item in items {
        try Self.upsert(item, in: db)
      }
    }
  }

  private static func upsert(_ item: Item, in db: Database) throws {
    let chordChart =
      try encodeChordChart(
        item.chordChart, keyIfUnreadable: storedChartKeyIfUnreadable(of: item.id, in: db))
      ?? storedIfUnreadable("chord_chart", of: item.id, as: StoredChart.self, in: db)
    let keys =
      try storedKeysIfUnreadable(of: item.id, unchanged: item.keys, in: db)
      ?? encodeKeys(item.keys)
    let metre =
      try encodeMetre(item.metre)
      ?? storedIfUnreadable("metre", of: item.id, as: StoredMetre.self, in: db)
    let key = try item.key.map(stored) ?? storedKeyIfUnreadable(of: item.id, in: db)
    try db.execute(
      sql: """
        INSERT INTO item
          (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
           created_at, updated_at, priority, chord_chart, photo_id,
           metre, variation_ids, keys, deleted_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
        ON CONFLICT(id) DO UPDATE SET
          title = excluded.title, kind = excluded.kind, composer = excluded.composer,
          key = excluded.key, modality = excluded.modality,
          tempo_marking = excluded.tempo_marking,
          tempo_bpm = excluded.tempo_bpm, notes = excluded.notes,
          tags = \(keepingUnreadable("tags")),
          updated_at = excluded.updated_at, priority = excluded.priority,
          chord_chart = excluded.chord_chart, photo_id = excluded.photo_id,
          metre = excluded.metre,
          variation_ids = \(keepingUnreadable("variation_ids")),
          keys = excluded.keys, deleted_at = NULL
        """,
      arguments: [
        item.id, item.title, Self.itemKinds.encode(item.kind), item.composer, key?.key,
        key?.modality,
        item.tempo?.marking, item.tempo?.bpm.map { Int($0) }, item.notes,
        try Self.encodeJSON(item.tags),
        item.createdAt, item.updatedAt, item.priority,
        chordChart, item.photoId, metre,
        try Self.encodeJSON(item.variationIds), keys,
      ])
    for s in item.sections {
      try db.execute(
        sql: """
          INSERT INTO section
            (id, item_id, name, bar_first, bar_last, kind, target_bpm, position, updated_at,
             deleted_at)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
          ON CONFLICT(id) DO UPDATE SET
            item_id = excluded.item_id, name = excluded.name,
            bar_first = excluded.bar_first, bar_last = excluded.bar_last,
            kind = excluded.kind, target_bpm = excluded.target_bpm,
            position = excluded.position, updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at
          """,
        arguments: [
          s.id, item.id, s.name, s.bars.map { Int($0.first) }, s.bars.map { Int($0.last) },
          Self.sectionKinds.encode(s.kind), s.targetBpm.map { Int($0) }, Int(s.position),
          s.updatedAt, s.deletedAt,
        ])
    }
    for l in item.exerciseLinks {
      try db.execute(
        sql: """
          INSERT INTO exercise_link
            (id, piece_id, exercise_id, section_id, position, updated_at, deleted_at)
          VALUES (?, ?, ?, ?, ?, ?, ?)
          ON CONFLICT(id) DO UPDATE SET
            piece_id = excluded.piece_id, exercise_id = excluded.exercise_id,
            section_id = excluded.section_id, position = excluded.position,
            updated_at = excluded.updated_at, deleted_at = excluded.deleted_at
          """,
        arguments: [
          l.id, item.id, l.exerciseId, l.sectionId, Int(l.position), l.updatedAt, l.deletedAt,
        ])
    }
  }

  /// A list column that will not decode reads back empty (#1117), so writing
  /// that empty list back would destroy the only copy; keep it until the list
  /// really changes.
  private static func keepingUnreadable(_ column: String) -> String {
    """
    CASE WHEN excluded.\(column) = '[]' AND NOT (
      json_valid(item.\(column)) AND json_type(item.\(column)) = 'array'
      AND NOT EXISTS (SELECT 1 FROM json_each(item.\(column)) WHERE type <> 'text')
    ) THEN item.\(column) ELSE excluded.\(column) END
    """
  }

  /// A value that will not decode reads back as none, so writing none back
  /// would destroy the only copy (#2097); keep it until a real value replaces it.
  private static func storedIfUnreadable<T: Decodable>(
    _ column: String, of id: String, as type: T.Type, in db: Database
  ) throws -> String? {
    guard
      let stored = try String.fetchOne(
        db, sql: "SELECT \(column) FROM item WHERE id = ?", arguments: [id]),
      tryDecodeJSON(type, from: stored) == nil
    else { return nil }
    return stored
  }

  /// A key list holding one the core cannot read loads without it; while the
  /// item's list is what loaded, the stored one stays so that key is not lost
  /// (#2097, #2106).
  private static func storedKeysIfUnreadable(
    of id: String, unchanged keys: [Key], in db: Database
  ) throws -> String? {
    guard
      let stored = try String.fetchOne(
        db, sql: "SELECT keys FROM item WHERE id = ?", arguments: [id])
    else { return nil }
    guard let dtos = tryDecodeJSON([StoredKeyJSON].self, from: stored) else {
      return keys.isEmpty ? stored : nil
    }
    let readable = decodeKeys(stored)
    return readable.count < dtos.count && readable == keys ? stored : nil
  }

  private static func storedChartKeyIfUnreadable(of id: String, in db: Database) throws
    -> StoredKeyJSON?
  {
    guard
      let stored = try String.fetchOne(
        db, sql: "SELECT chord_chart FROM item WHERE id = ?", arguments: [id]),
      let chart = tryDecodeJSON(StoredChart.self, from: stored),
      !chart.key.isEmpty, key(text: chart.key, modality: chart.modality) == nil
    else { return nil }
    return StoredKeyJSON(key: chart.key, modality: chart.modality)
  }

  /// A key the core could not read loads as none, so writing none back would
  /// lose the musician's text; it stays until they pick a key (#2106).
  private static func storedKeyIfUnreadable(of id: String, in db: Database) throws
    -> StoredKeyJSON?
  {
    guard
      let row = try Row.fetchOne(
        db, sql: "SELECT key, modality FROM item WHERE id = ?", arguments: [id]),
      let text = row["key"] as String?,
      key(text: text, modality: row["modality"]) == nil
    else { return nil }
    return StoredKeyJSON(key: text, modality: row["modality"])
  }

  /// Soft-delete: write the core-stamped `deletedAt` tombstone (RFC3339, same
  /// format as `updated_at`) rather than removing the row, so the deletion can
  /// win a later last-write-wins sync.
  func delete(id: String, deletedAt: String) throws {
    try dbQueue.write { db in
      try db.execute(
        sql: "UPDATE item SET deleted_at = ? WHERE id = ?",
        arguments: [deletedAt, id])
    }
  }

  func loadSessions() throws -> [PracticeSession] {
    try dbQueue.read { db in
      try Row.fetchAll(
        db, sql: "SELECT * FROM session WHERE deleted_at IS NULL ORDER BY completed_at DESC"
      )
      .map(Self.session(from:))
    }
  }

  /// Insert or update by id. A session is immutable once completed, so
  /// `updated_at` simply tracks `completed_at` — the column exists for sync LWW.
  func saveSession(_ session: PracticeSession) throws {
    try dbQueue.write { db in
      try db.execute(
        sql: """
          INSERT INTO session
            (id, started_at, completed_at, total_duration_secs, completion_status,
             session_notes, entries, updated_at, deleted_at, session_score, capture_version)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?)
          ON CONFLICT(id) DO UPDATE SET
            started_at = excluded.started_at, completed_at = excluded.completed_at,
            total_duration_secs = excluded.total_duration_secs,
            completion_status = excluded.completion_status,
            session_notes = excluded.session_notes,
            entries = excluded.entries, updated_at = excluded.updated_at, deleted_at = NULL,
            session_score = excluded.session_score, capture_version = excluded.capture_version
          """,
        arguments: [
          session.id, session.startedAt, session.completedAt,
          Int(session.totalDurationSecs), Self.completionStatuses.encode(session.completionStatus),
          session.sessionNotes,
          try Self.encodeEntries(session.entries), session.completedAt,
          session.sessionScore.map { Int($0) }, session.captureVersion.map { Int($0) },
        ])
    }
  }

  /// Tombstones included: plays name deleted variations too (#2246).
  func loadVariations() throws -> [Variation] {
    try dbQueue.read { db in
      try Row.fetchAll(db, sql: "SELECT * FROM variation ORDER BY rowid").map(Self.variation)
    }
  }

  /// Keyed by id, tombstones written as sent: the core owns every row.
  func save(_ variations: [Variation]) throws {
    try dbQueue.write { db in
      for v in variations {
        try db.execute(
          sql: """
            INSERT INTO variation (id, label, updated_at, deleted_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
              label = excluded.label, updated_at = excluded.updated_at,
              deleted_at = excluded.deleted_at
            """,
          arguments: [v.id, v.label, v.updatedAt, v.deletedAt])
      }
    }
  }

  /// Column names of a table (for the schema-invariant test). `[String]` not
  /// `Set` — `SharedTypes`' domain `Set` shadows `Swift.Set` here.
  func columnNames(ofTable table: String) throws -> [String] {
    try dbQueue.read { db in try db.columns(in: table).map(\.name) }
  }

  /// Tombstones included: the core reconciles (#2248).
  private static func linksByPiece(_ db: Database) throws -> [String: [ExerciseLink]] {
    let rows = try Row.fetchAll(
      db, sql: "SELECT * FROM exercise_link ORDER BY piece_id, position, id")
    var byPiece: [String: [ExerciseLink]] = [:]
    for row in rows {
      byPiece[row["piece_id"], default: []].append(exerciseLink(from: row))
    }
    return byPiece
  }

  /// Tombstones included: the core reconciles (#2245).
  private static func sectionsByItem(_ db: Database) throws -> [String: [ItemSection]] {
    let rows = try Row.fetchAll(
      db,
      sql: "SELECT * FROM section ORDER BY item_id, position, id")
    var byItem: [String: [ItemSection]] = [:]
    for row in rows {
      byItem[row["item_id"], default: []].append(section(from: row))
    }
    return byItem
  }
}

#if DEBUG
  extension LibraryStore {
    /// Test seam for upgrade-path tests (CLAUDE.md "Local data migrations"):
    /// migrate to `version`, seed raw rows at that schema, then finish to HEAD.
    static func upgradeTestStore(migratedTo version: String, seed: String) throws -> LibraryStore {
      let queue = try DatabaseQueue()
      try migrator.migrate(queue, upTo: version)
      try queue.write { db in try db.execute(sql: seed) }
      return try LibraryStore(queue)
    }

    func rawText(_ column: String, ofItem id: String) throws -> String? {
      try dbQueue.read { db in
        try String.fetchOne(db, sql: "SELECT \(column) FROM item WHERE id = ?", arguments: [id])
      }
    }
  }
#endif
