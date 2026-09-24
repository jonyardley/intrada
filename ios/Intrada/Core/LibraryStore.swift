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
      // Variants load tombstones included: the core owns reconciliation
      // (resurrect-by-label) and history labels resolve through them (#1083).
      let variantsByItem = try Self.variantsByItem(db)
      return try Row.fetchAll(
        db, sql: "SELECT * FROM item WHERE deleted_at IS NULL ORDER BY created_at DESC"
      )
      .map { row in Self.item(from: row, variants: variantsByItem[row["id"]] ?? []) }
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
      try encodeChordChart(item.chordChart)
      ?? storedIfUnreadable("chord_chart", of: item.id, as: StoredChart.self, in: db)
    let metre =
      try encodeMetre(item.metre)
      ?? storedIfUnreadable("metre", of: item.id, as: StoredMetre.self, in: db)
    try db.execute(
      sql: """
        INSERT INTO item
          (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
           linked_exercise_ids, created_at, updated_at, priority, chord_chart, photo_id,
           metre, deleted_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
        ON CONFLICT(id) DO UPDATE SET
          title = excluded.title, kind = excluded.kind, composer = excluded.composer,
          key = excluded.key, modality = excluded.modality,
          tempo_marking = excluded.tempo_marking,
          tempo_bpm = excluded.tempo_bpm, notes = excluded.notes,
          tags = \(keepingUnreadable("tags")),
          linked_exercise_ids = \(keepingUnreadable("linked_exercise_ids")),
          updated_at = excluded.updated_at, priority = excluded.priority,
          chord_chart = excluded.chord_chart, photo_id = excluded.photo_id,
          metre = excluded.metre, deleted_at = NULL
        """,
      arguments: [
        item.id, item.title, Self.itemKinds.encode(item.kind), item.composer, item.key,
        item.modality.map(Self.modalities.encode),
        item.tempo?.marking, item.tempo?.bpm.map { Int($0) }, item.notes,
        try Self.encodeJSON(item.tags),
        try Self.encodeJSON(item.linkedExerciseIds),
        item.createdAt, item.updatedAt, item.priority,
        chordChart, item.photoId, metre,
      ])
    // Same transaction as the item row, keyed by id; no delete-missing: the
    // core always carries the tombstones it loaded and writes them back
    // verbatim, so Swift never diffs and a tombstone round-trips (fixes the
    // forced-NULL resurrect hazard, #1113).
    for v in item.variants {
      try db.execute(
        sql: """
          INSERT INTO variant (id, item_id, label, position, updated_at, deleted_at)
          VALUES (?, ?, ?, ?, ?, ?)
          ON CONFLICT(id) DO UPDATE SET
            item_id = excluded.item_id, label = excluded.label,
            position = excluded.position, updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at
          """,
        arguments: [v.id, item.id, v.label, Int(v.position), v.updatedAt, v.deletedAt])
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
             session_notes, entries, updated_at, deleted_at, session_score)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, ?)
          ON CONFLICT(id) DO UPDATE SET
            started_at = excluded.started_at, completed_at = excluded.completed_at,
            total_duration_secs = excluded.total_duration_secs,
            completion_status = excluded.completion_status,
            session_notes = excluded.session_notes,
            entries = excluded.entries, updated_at = excluded.updated_at, deleted_at = NULL,
            session_score = excluded.session_score
          """,
        arguments: [
          session.id, session.startedAt, session.completedAt,
          Int(session.totalDurationSecs), Self.completionStatuses.encode(session.completionStatus),
          session.sessionNotes,
          try Self.encodeEntries(session.entries), session.completedAt,
          session.sessionScore.map { Int($0) },
        ])
    }
  }

  /// Column names of a table (for the schema-invariant test). `[String]` not
  /// `Set` — `SharedTypes`' domain `Set` shadows `Swift.Set` here.
  func columnNames(ofTable table: String) throws -> [String] {
    try dbQueue.read { db in try db.columns(in: table).map(\.name) }
  }

  /// All variants (variations) grouped by owning item, in ladder order, tombstones
  /// included, per the core's reconciliation contract (#1083). One query for
  /// the whole library, keyed by `item_id`, so `loadItems` stays O(1) reads.
  private static func variantsByItem(_ db: Database) throws -> [String: [Variant]] {
    let rows = try Row.fetchAll(
      db,
      sql: "SELECT * FROM variant ORDER BY item_id, position, id")
    var byItem: [String: [Variant]] = [:]
    for row in rows {
      byItem[row["item_id"], default: []].append(variant(from: row))
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
