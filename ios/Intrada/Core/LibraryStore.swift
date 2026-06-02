import Foundation
import GRDB
import SharedTypes

/// Persistence ops the Store resolves against — a protocol so tests can inject a failing fake (#816).
protocol ItemStore {
  func loadItems() throws -> [Item]
  func save(_ item: Item) throws
  func delete(id: String) throws
}

/// On-device SQLite store (GRDB) — the B2 local-first persistence layer the
/// `Effect.persistence` operations resolve against. The schema is deliberately
/// **sync-agnostic**: every row carries `updated_at` + a soft-delete tombstone
/// so a later sync engine (custom LWW or Automerge) can sit on top without a
/// migration (see specs/native-ios.md "Sync engine").
///
/// Calls are synchronous; the dataset is single-user and tiny, so GRDB's own
/// serialization is enough and an off-main hop isn't worth the Sendable dance
/// against the non-Sendable generated `Item`. Revisit if data volume grows.
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
      try Row.fetchAll(
        db, sql: "SELECT * FROM item WHERE deleted_at IS NULL ORDER BY created_at DESC"
      )
      .map(Self.item(from:))
    }
  }

  /// Insert or update by id; clears any tombstone (an upsert revives a row).
  func save(_ item: Item) throws {
    try dbQueue.write { db in
      try db.execute(
        sql: """
          INSERT INTO item
            (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
             created_at, updated_at, priority, deleted_at)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
          ON CONFLICT(id) DO UPDATE SET
            title = excluded.title, kind = excluded.kind, composer = excluded.composer,
            key = excluded.key, modality = excluded.modality,
            tempo_marking = excluded.tempo_marking,
            tempo_bpm = excluded.tempo_bpm, notes = excluded.notes, tags = excluded.tags,
            updated_at = excluded.updated_at, priority = excluded.priority, deleted_at = NULL
          """,
        arguments: [
          item.id, item.title, Self.kindString(item.kind), item.composer, item.key,
          Self.modalityString(item.modality),
          item.tempo?.marking, item.tempo?.bpm.map { Int($0) }, item.notes,
          Self.encodeTags(item.tags),
          item.createdAt, item.updatedAt, item.priority,
        ])
    }
  }

  /// Soft-delete: stamp `deleted_at` (tombstone) rather than removing the row,
  /// so the deletion can win a later last-write-wins sync.
  func delete(id: String) throws {
    try dbQueue.write { db in
      try db.execute(
        sql: "UPDATE item SET deleted_at = ? WHERE id = ?",
        arguments: [Self.timestamp(), id])
    }
  }

  /// Column names of a table (for the schema-invariant test). `[String]` not
  /// `Set` — `SharedTypes`' domain `Set` shadows `Swift.Set` here.
  func columnNames(ofTable table: String) throws -> [String] {
    try dbQueue.read { db in try db.columns(in: table).map(\.name) }
  }

  // ── Schema ───────────────────────────────────────────────────────────

  private static var migrator: DatabaseMigrator {
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
    return migrator
  }

  // ── Row ↔ Item codec ─────────────────────────────────────────────────

  private static func item(from row: Row) -> Item {
    let marking: String? = row["tempo_marking"]
    let bpm: UInt16? = (row["tempo_bpm"] as Int?).map { UInt16($0) }
    let tempo = (marking == nil && bpm == nil) ? nil : Tempo(marking: marking, bpm: bpm)
    return Item(
      id: row["id"], title: row["title"], kind: kind(from: row["kind"]),
      composer: row["composer"], key: row["key"], modality: modality(from: row["modality"]),
      tempo: tempo, notes: row["notes"],
      tags: decodeTags(row["tags"]), createdAt: row["created_at"], updatedAt: row["updated_at"],
      priority: row["priority"])
  }

  private static func kindString(_ kind: ItemKind) -> String {
    switch kind {
    case .piece: "piece"
    case .exercise: "exercise"
    }
  }

  private static func modalityString(_ modality: Modality?) -> String? {
    switch modality {
    case .major: "major"
    case .minor: "minor"
    case nil: nil
    }
  }

  private static func modality(from raw: String?) -> Modality? {
    switch raw {
    case "major": .major
    case "minor": .minor
    default: nil
    }
  }

  private static func kind(from raw: String) -> ItemKind {
    raw == "exercise" ? .exercise : .piece
  }

  private static func encodeTags(_ tags: [String]) -> String {
    guard let data = try? JSONEncoder().encode(tags), let json = String(data: data, encoding: .utf8)
    else { return "[]" }
    return json
  }

  private static func decodeTags(_ json: String) -> [String] {
    (try? JSONDecoder().decode([String].self, from: Data(json.utf8))) ?? []
  }

  private static func timestamp() -> String {
    ISO8601DateFormatter().string(from: Date())
  }
}
