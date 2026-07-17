import Foundation
import GRDB
import SharedTypes

/// Persistence ops the Store resolves against — a protocol so tests can inject a failing fake (#816).
protocol ItemStore {
  func loadItems() throws -> [Item]
  func save(_ item: Item) throws
  func delete(id: String, deletedAt: String) throws
  func loadSessions() throws -> [PracticeSession]
  func saveSession(_ session: PracticeSession) throws
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
             linked_exercise_ids, created_at, updated_at, priority, chord_chart, deleted_at)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
          ON CONFLICT(id) DO UPDATE SET
            title = excluded.title, kind = excluded.kind, composer = excluded.composer,
            key = excluded.key, modality = excluded.modality,
            tempo_marking = excluded.tempo_marking,
            tempo_bpm = excluded.tempo_bpm, notes = excluded.notes, tags = excluded.tags,
            linked_exercise_ids = excluded.linked_exercise_ids,
            updated_at = excluded.updated_at, priority = excluded.priority,
            chord_chart = excluded.chord_chart, deleted_at = NULL
          """,
        arguments: [
          item.id, item.title, Self.kindString(item.kind), item.composer, item.key,
          Self.modalityString(item.modality),
          item.tempo?.marking, item.tempo?.bpm.map { Int($0) }, item.notes,
          Self.encodeTags(item.tags),
          Self.encodeLinkedExerciseIds(item.linkedExerciseIds),
          item.createdAt, item.updatedAt, item.priority,
          Self.encodeChordChart(item.chordChart),
        ])
    }
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
             session_notes, session_intention, entries, updated_at, deleted_at, session_score,
             reflection_improved, reflection_still_rough, reflection_next_target)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, ?, ?)
          ON CONFLICT(id) DO UPDATE SET
            started_at = excluded.started_at, completed_at = excluded.completed_at,
            total_duration_secs = excluded.total_duration_secs,
            completion_status = excluded.completion_status,
            session_notes = excluded.session_notes, session_intention = excluded.session_intention,
            entries = excluded.entries, updated_at = excluded.updated_at, deleted_at = NULL,
            session_score = excluded.session_score,
            reflection_improved = excluded.reflection_improved,
            reflection_still_rough = excluded.reflection_still_rough,
            reflection_next_target = excluded.reflection_next_target
          """,
        arguments: [
          session.id, session.startedAt, session.completedAt,
          Int(session.totalDurationSecs), Self.completionString(session.completionStatus),
          session.sessionNotes, session.sessionIntention,
          Self.encodeEntries(session.entries), session.completedAt,
          session.sessionScore.map { Int($0) },
          session.reflectionImproved, session.reflectionStillRough,
          session.reflectionNextTarget,
        ])
    }
  }

  /// Column names of a table (for the schema-invariant test). `[String]` not
  /// `Set` — `SharedTypes`' domain `Set` shadows `Swift.Set` here.
  func columnNames(ofTable table: String) throws -> [String] {
    try dbQueue.read { db in try db.columns(in: table).map(\.name) }
  }

  // ── Schema ───────────────────────────────────────────────────────────

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
    migrator.registerMigration("v5_rescale_entry_scores") { db in
      let rows = try Row.fetchAll(db, sql: "SELECT id, entries FROM session")
      for row in rows {
        let id: String = row["id"]
        let json: String = row["entries"]
        guard var dtos = try? JSONDecoder().decode([StoredEntry].self, from: Data(json.utf8))
        else { continue }
        for i in dtos.indices {
          if let s = dtos[i].score { dtos[i].score = UInt8(min(10, Int(s) * 2)) }
        }
        guard let data = try? JSONEncoder().encode(dtos),
          let rescaled = String(data: data, encoding: .utf8)
        else { continue }
        try db.execute(
          sql: "UPDATE session SET entries = ? WHERE id = ?", arguments: [rescaled, id])
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
    return migrator
  }()

  // ── Row ↔ Item codec ─────────────────────────────────────────────────

  // Surface (don't silently default) a stored enum string we don't recognise —
  // e.g. an older binary reading a row a newer version wrote (#949).
  private static let decodeContext = "LibraryStore decode"

  private struct UnknownStoredEnum: Error, CustomStringConvertible {
    let kind: String
    let raw: String
    var description: String { "unknown \(kind) on decode: \"\(raw)\"" }
  }

  private static func item(from row: Row) -> Item {
    let marking: String? = row["tempo_marking"]
    let bpm: UInt16? = (row["tempo_bpm"] as Int?).map { UInt16($0) }
    let tempo = (marking == nil && bpm == nil) ? nil : Tempo(marking: marking, bpm: bpm)
    return Item(
      id: row["id"], title: row["title"], kind: kind(from: row["kind"]),
      composer: row["composer"], key: row["key"], modality: modality(from: row["modality"]),
      tempo: tempo, notes: row["notes"],
      tags: decodeTags(row["tags"]),
      linkedExerciseIds: decodeLinkedExerciseIds(row["linked_exercise_ids"]),
      createdAt: row["created_at"], updatedAt: row["updated_at"],
      priority: row["priority"],
      chordChart: decodeChordChart(row["chord_chart"]))
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
    case "major": return .major
    case "minor": return .minor
    case nil: return nil
    case .some(let other):
      report(UnknownStoredEnum(kind: "Modality", raw: other), decodeContext)
      return nil
    }
  }

  private static func kind(from raw: String) -> ItemKind {
    switch raw {
    case "piece": return .piece
    case "exercise": return .exercise
    default:
      report(UnknownStoredEnum(kind: "ItemKind", raw: raw), decodeContext)
      return .piece
    }
  }

  private static func encodeTags(_ tags: [String]) -> String {
    guard let data = try? JSONEncoder().encode(tags), let json = String(data: data, encoding: .utf8)
    else { return "[]" }
    return json
  }

  private static func decodeTags(_ json: String) -> [String] {
    (try? JSONDecoder().decode([String].self, from: Data(json.utf8))) ?? []
  }

  private static func encodeLinkedExerciseIds(_ ids: [String]) -> String {
    guard let data = try? JSONEncoder().encode(ids), let json = String(data: data, encoding: .utf8)
    else { return "[]" }
    return json
  }

  private static func decodeLinkedExerciseIds(_ json: String) -> [String] {
    (try? JSONDecoder().decode([String].self, from: Data(json.utf8))) ?? []
  }

  // ── Row ↔ ChordChart codec ───────────────────────────────────────────
  // A nested aggregate, so JSON via a Codable DTO (like StoredEntry) — never
  // bincode: positional encoding would fail to decode old rows after a field
  // change, and the device is the only copy.

  private struct StoredChart: Codable {
    var key: String
    var modality: String
    var metre: UInt8
    var sections: [StoredSection]
  }
  private struct StoredSection: Codable {
    var label: String?
    var bars: [StoredBar]
  }
  private struct StoredBar: Codable {
    var chords: [StoredChartChord]
  }
  private struct StoredChartChord: Codable {
    var symbol: StoredChordSymbol
    var beats: UInt8
  }
  private struct StoredChordSymbol: Codable {
    var root: UInt8
    var quality: String
    var extensions: [String]
    var bass: UInt8?
    var raw: String
  }

  private struct ChartCodecError: Error, CustomStringConvertible {
    let phase: String
    var description: String { "chord chart failed to \(phase)" }
  }

  private static func encodeChordChart(_ chart: ChordChart?) -> String? {
    guard let chart else { return nil }
    let dto = StoredChart(
      key: chart.key, modality: modalityString(chart.modality) ?? "major", metre: chart.metre,
      sections: chart.sections.map { section in
        StoredSection(
          label: section.label,
          bars: section.bars.map { bar in
            StoredBar(
              chords: bar.chords.map { chord in
                StoredChartChord(
                  symbol: StoredChordSymbol(
                    root: chord.symbol.root, quality: chordQualityString(chord.symbol.quality),
                    extensions: chord.symbol.extensions, bass: chord.symbol.bass,
                    raw: chord.symbol.raw),
                  beats: chord.beats)
              })
          })
      })
    guard let data = try? JSONEncoder().encode(dto), let json = String(data: data, encoding: .utf8)
    else {
      // Surface, don't swallow: a chart that fails to encode would otherwise be
      // stored as NULL (silently "no chart") on the only copy of the data.
      report(ChartCodecError(phase: "encode"), decodeContext)
      return nil
    }
    return json
  }

  private static func decodeChordChart(_ json: String?) -> ChordChart? {
    guard let json else { return nil }  // no chart — legitimate
    guard let dto = try? JSONDecoder().decode(StoredChart.self, from: Data(json.utf8)) else {
      report(ChartCodecError(phase: "decode"), decodeContext)
      return nil
    }
    return ChordChart(
      key: dto.key, modality: modality(from: dto.modality) ?? .major, metre: dto.metre,
      sections: dto.sections.map { section in
        ChartSection(
          label: section.label,
          bars: section.bars.map { bar in
            Bar(
              chords: bar.chords.map { chord in
                ChartChord(
                  symbol: ChordSymbol(
                    root: chord.symbol.root,
                    quality: chordQuality(from: chord.symbol.quality),
                    extensions: chord.symbol.extensions, bass: chord.symbol.bass,
                    raw: chord.symbol.raw),
                  beats: chord.beats)
              })
          })
      })
  }

  private static func chordQualityString(_ q: ChordQuality) -> String {
    switch q {
    case .maj7: "maj7"
    case .dom7: "dom7"
    case .min7: "min7"
    case .min7b5: "min7b5"
    case .dim7: "dim7"
    case .minMaj7: "minMaj7"
    case .six: "six"
    case .min6: "min6"
    case .alt: "alt"
    case .other: "other"
    }
  }

  private static func chordQuality(from raw: String) -> ChordQuality {
    switch raw {
    case "maj7": return .maj7
    case "dom7": return .dom7
    case "min7": return .min7
    case "min7b5": return .min7b5
    case "dim7": return .dim7
    case "minMaj7": return .minMaj7
    case "six": return .six
    case "min6": return .min6
    case "alt": return .alt
    case "other": return .other
    default:
      report(UnknownStoredEnum(kind: "ChordQuality", raw: raw), decodeContext)
      return .other  // conservative: an unknown quality falls back to arpeggio
    }
  }

  // ── Row ↔ PracticeSession codec ──────────────────────────────────────

  private static func session(from row: Row) -> PracticeSession {
    let score: Int64? = row["session_score"]
    return PracticeSession(
      id: row["id"], entries: decodeEntries(row["entries"]),
      sessionNotes: row["session_notes"], sessionIntention: row["session_intention"],
      startedAt: row["started_at"], completedAt: row["completed_at"],
      totalDurationSecs: UInt64(row["total_duration_secs"] as Int64),
      completionStatus: completionStatus(from: row["completion_status"]),
      sessionScore: score.map { UInt8(clamping: $0) },
      reflectionImproved: row["reflection_improved"],
      reflectionStillRough: row["reflection_still_rough"],
      reflectionNextTarget: row["reflection_next_target"])
  }

  // Entries (a nested, optional-heavy aggregate) go to JSON via a Codable DTO,
  // not bincode: bincode is positional, so a future field change would fail to
  // decode old rows — unacceptable when the device is the only copy.
  private struct StoredEntry: Codable {
    var id: String
    var itemId: String
    var itemTitle: String
    var itemType: String
    var position: UInt64
    var durationSecs: UInt64
    var status: String
    var notes: String?
    var score: UInt8?
    var intention: String?
    var repTarget: UInt8?
    var repCount: UInt8?
    var repTargetReached: Bool?
    var repHistory: [String]?
    var plannedDurationSecs: UInt32?
    var achievedTempo: UInt16?
    var groupId: String?
  }

  private static func encodeEntries(_ entries: [SetlistEntry]) -> String {
    let dtos = entries.map { e in
      StoredEntry(
        id: e.id, itemId: e.itemId, itemTitle: e.itemTitle, itemType: kindString(e.itemType),
        position: e.position, durationSecs: e.durationSecs, status: entryStatusString(e.status),
        notes: e.notes, score: e.score, intention: e.intention, repTarget: e.repTarget,
        repCount: e.repCount, repTargetReached: e.repTargetReached,
        repHistory: e.repHistory.map { $0.map(repActionString) },
        plannedDurationSecs: e.plannedDurationSecs, achievedTempo: e.achievedTempo,
        groupId: e.groupId)
    }
    guard let data = try? JSONEncoder().encode(dtos), let json = String(data: data, encoding: .utf8)
    else { return "[]" }
    return json
  }

  private static func decodeEntries(_ json: String) -> [SetlistEntry] {
    guard let dtos = try? JSONDecoder().decode([StoredEntry].self, from: Data(json.utf8)) else {
      return []
    }
    return dtos.map { d in
      SetlistEntry(
        id: d.id, itemId: d.itemId, itemTitle: d.itemTitle, itemType: kind(from: d.itemType),
        position: d.position, durationSecs: d.durationSecs, status: entryStatus(from: d.status),
        notes: d.notes, score: d.score, intention: d.intention, repTarget: d.repTarget,
        repCount: d.repCount, repTargetReached: d.repTargetReached,
        repHistory: d.repHistory.map { $0.map(repAction(from:)) },
        plannedDurationSecs: d.plannedDurationSecs, achievedTempo: d.achievedTempo,
        groupId: d.groupId)
    }
  }

  private static func completionString(_ status: CompletionStatus) -> String {
    switch status {
    case .completed: "completed"
    case .endedEarly: "ended_early"
    }
  }

  private static func completionStatus(from raw: String) -> CompletionStatus {
    switch raw {
    case "completed": return .completed
    case "ended_early": return .endedEarly
    default:
      report(UnknownStoredEnum(kind: "CompletionStatus", raw: raw), decodeContext)
      return .completed
    }
  }

  private static func entryStatusString(_ status: EntryStatus) -> String {
    switch status {
    case .completed: "completed"
    case .skipped: "skipped"
    case .notAttempted: "not_attempted"
    }
  }

  private static func entryStatus(from raw: String) -> EntryStatus {
    switch raw {
    case "completed": return .completed
    case "skipped": return .skipped
    case "not_attempted": return .notAttempted
    default:
      report(UnknownStoredEnum(kind: "EntryStatus", raw: raw), decodeContext)
      return .notAttempted  // conservative: an unknown status must not inflate stats (#949)
    }
  }

  private static func repActionString(_ action: RepAction) -> String {
    switch action {
    case .missed: "missed"
    case .success: "success"
    }
  }

  private static func repAction(from raw: String) -> RepAction {
    switch raw {
    case "missed": return .missed
    case "success": return .success
    default:
      report(UnknownStoredEnum(kind: "RepAction", raw: raw), decodeContext)
      return .missed  // conservative: an unknown rep must not inflate achievement (#949)
    }
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
  }
#endif
