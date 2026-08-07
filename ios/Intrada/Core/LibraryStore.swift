import Foundation
import GRDB
import SharedTypes

/// Persistence ops the Store resolves against — a protocol so tests can inject a failing fake (#816).
protocol ItemStore {
  func loadItems() throws -> [Item]
  func save(_ item: Item) throws
  /// Upsert a batch in one transaction — all rows land or none do (#1106).
  func save(_ items: [Item]) throws
  func delete(id: String, deletedAt: String) throws
  func loadSessions() throws -> [PracticeSession]
  func saveSession(_ session: PracticeSession) throws
  /// Append coach evidence in one transaction — blocks, wanders and their
  /// attempts land together or not at all (#1181).
  func saveCoachRecords(blocks: [BlockRecord], wanders: [WanderRecord], updatedAt: String) throws
  /// The closed blocks back, so the core can rebuild the mastery track at
  /// launch (#1214). Wanders stay write-only: no `(node, level)` to score.
  func loadCoachRecords() throws -> [BlockRecord]
  // Built-session entities (#1256): upserts by id; deletes arrive as
  // tombstoned saves, so no delete methods exist.
  func saveUserDrill(_ drill: UserDrill) throws
  func saveJournalItem(_ journal: JournalItem) throws
  func saveBuiltSession(_ session: BuiltSession) throws
  func savePlayThrough(_ record: PlayThroughRecord) throws
  func saveReflection(_ reflection: Reflection) throws
  func saveFeelEntry(_ entry: FeelEntry) throws
  func loadBuiltSessionData() throws -> BuiltSessionData
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

  /// One transaction for the whole batch, so a block and the wander beside it
  /// never half-land (#1181). Upsert by id: a retry after a failed write is
  /// idempotent, and the off-piste *keep this as a drill?* answer updates the
  /// wander row already written rather than duplicating it.
  func saveCoachRecords(blocks: [BlockRecord], wanders: [WanderRecord], updatedAt: String) throws {
    try dbQueue.write { db in
      for block in blocks {
        try Self.upsert(block, updatedAt: updatedAt, in: db)
      }
      for wander in wanders {
        try Self.upsert(wander, updatedAt: updatedAt, in: db)
      }
    }
  }

  func loadCoachRecords() throws -> [BlockRecord] {
    try dbQueue.read { db in
      try Row.fetchAll(
        db, sql: "SELECT * FROM block_record WHERE deleted_at IS NULL ORDER BY ended_at, id"
      ).map(Self.blockRecord(from:))
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
    migrator.registerMigration("v9_variant") { db in
      // Exercise step ladders (#1083). First normalized child table: per-row
      // `updated_at`/`deleted_at` give the future sync engine per-step LWW +
      // tombstones (invariant 2) that a JSON blob on `item` couldn't. Additive —
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
      // Self-directed practice (#1256, specs/built-session.md). `blocks` and
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

  private static func item(from row: Row, variants: [Variant]) -> Item {
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
      chordChart: decodeChordChart(row["chord_chart"]),
      variants: variants)
  }

  // ── Row ↔ Variant codec ──────────────────────────────────────────────

  private static func variant(from row: Row) -> Variant {
    Variant(
      id: row["id"], label: row["label"], position: UInt64(row["position"] as Int),
      updatedAt: row["updated_at"], deletedAt: row["deleted_at"])
  }

  /// All variants (steps) grouped by owning item, in ladder order — tombstones
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
    case .sus4: "sus4"
    case .sus2: "sus2"
    case .aug: "aug"
    case .dom7Sharp5: "dom7Sharp5"
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
    case "sus4": return .sus4
    case "sus2": return .sus2
    case "aug": return .aug
    case "dom7Sharp5": return .dom7Sharp5
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
    var variantId: String?
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
        groupId: e.groupId, variantId: e.variantId)
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
        groupId: d.groupId, variantId: d.variantId)
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

  // ── BlockRecord / WanderRecord ↔ row codec (#1181, read side #1214) ───
  // The stored enum spellings are the format contract both directions share.
  // Wanders stay write-only: they carry no (node, level) for the rebuild.

  private struct StoredAttempt: Codable {
    var at: String
    var verdict: String
    var source: String
    var cold: Bool
    var selfPredicted: String?
    /// Optional because rows written before #1214 have no per-attempt level.
    /// The decoder falls back to the block's, which is all those rows ever knew.
    var levelTempoBpm: UInt16?
    var levelClickLevel: String?
  }

  private struct CoachCodecError: Error, CustomStringConvertible {
    let field: String
    let phase: String
    var description: String { "coach \(field) failed to \(phase)" }
  }

  private static func upsert(_ record: BlockRecord, updatedAt: String, in db: Database) throws {
    try db.execute(
      sql: """
        INSERT INTO block_record
          (id, node, drill, gate, level_tempo_bpm, level_click_level, circle, mode,
           started_at, ended_at, attempts, attempts_to_pass, gate_opened_at_attempt,
           reps_after_gate, active_ms, escalation_fired, exit, updated_at, deleted_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
        ON CONFLICT(id) DO UPDATE SET
          ended_at = excluded.ended_at, attempts = excluded.attempts,
          attempts_to_pass = excluded.attempts_to_pass,
          gate_opened_at_attempt = excluded.gate_opened_at_attempt,
          reps_after_gate = excluded.reps_after_gate, active_ms = excluded.active_ms,
          escalation_fired = excluded.escalation_fired, exit = excluded.exit,
          updated_at = excluded.updated_at
        """,
      arguments: [
        record.id, record.node, record.drill, record.gate,
        Int(record.level.tempoBpm), clickLevelString(record.level.clickLevel),
        circleString(record.circle), modeString(record.mode),
        record.startedAt, record.endedAt, try encodeAttempts(record.attempts),
        record.attemptsToPass.map { Int($0) }, record.gateOpenedAtAttempt.map { Int($0) },
        Int(record.repsAfterGate), Int(clamping: record.activeMs),
        try encodeJSON(record.escalationFired.map(rungString), field: "escalation_fired"),
        exitString(record.exit), updatedAt,
      ])
  }

  private static func upsert(_ record: WanderRecord, updatedAt: String, in db: Database) throws {
    try db.execute(
      sql: """
        INSERT INTO wander_record
          (id, started_at, ended_at, attempts, keep_as_drill, updated_at, deleted_at)
        VALUES (?, ?, ?, ?, ?, ?, NULL)
        ON CONFLICT(id) DO UPDATE SET
          ended_at = excluded.ended_at, attempts = excluded.attempts,
          keep_as_drill = excluded.keep_as_drill, updated_at = excluded.updated_at
        """,
      arguments: [
        record.id, record.startedAt, record.endedAt, try encodeAttempts(record.attempts),
        record.keepAsDrill, updatedAt,
      ])
  }

  /// Throws rather than substituting `[]`: losing the attempts would make a
  /// failed write look like a block nobody played (invariant 5).
  private static func encodeAttempts(_ attempts: [AttemptSummary]) throws -> String {
    try encodeJSON(
      attempts.map { attempt in
        StoredAttempt(
          at: attempt.at, verdict: verdictString(attempt.verdict),
          source: evidenceSourceString(attempt.source), cold: attempt.cold,
          selfPredicted: attempt.selfPredicted.map(verdictString),
          levelTempoBpm: attempt.level.tempoBpm,
          levelClickLevel: clickLevelString(attempt.level.clickLevel))
      }, field: "attempts")
  }

  private static func encodeJSON<T: Encodable>(_ value: T, field: String) throws -> String {
    let data = try JSONEncoder().encode(value)
    guard let json = String(data: data, encoding: .utf8) else {
      throw CoachCodecError(field: field, phase: "encode")
    }
    return json
  }

  /// Throws rather than substituting a partial record: evidence that failed to
  /// decode would replay as a block nobody played (invariant 5). An unrecognised
  /// enum spelling is different: the row is still readable, so those report and
  /// fall back (#949).
  private static func blockRecord(from row: Row) throws -> BlockRecord {
    let level = ParameterLevel(
      tempoBpm: UInt16(clamping: row["level_tempo_bpm"] as Int),
      clickLevel: clickLevel(from: row["level_click_level"]))
    return BlockRecord(
      id: row["id"], node: row["node"], drill: row["drill"], gate: row["gate"],
      level: level,
      circle: circle(from: row["circle"]), mode: mode(from: row["mode"]),
      startedAt: row["started_at"], endedAt: row["ended_at"],
      attempts: try decodeAttempts(row["attempts"], blockLevel: level),
      attemptsToPass: (row["attempts_to_pass"] as Int?).map { UInt16(clamping: $0) },
      gateOpenedAtAttempt: (row["gate_opened_at_attempt"] as Int?).map { UInt16(clamping: $0) },
      repsAfterGate: UInt16(clamping: row["reps_after_gate"] as Int),
      activeMs: UInt64(clamping: row["active_ms"] as Int64),
      escalationFired: try decodeRungs(row["escalation_fired"]),
      exit: exit(from: row["exit"]))
  }

  private static func decodeAttempts(_ json: String, blockLevel: ParameterLevel) throws
    -> [AttemptSummary]
  {
    let stored: [StoredAttempt]
    do { stored = try JSONDecoder().decode([StoredAttempt].self, from: Data(json.utf8)) } catch {
      throw CoachCodecError(field: "attempts", phase: "decode")
    }
    return stored.map { attempt in
      AttemptSummary(
        at: attempt.at, verdict: verdict(from: attempt.verdict),
        source: evidenceSource(from: attempt.source), cold: attempt.cold,
        selfPredicted: attempt.selfPredicted.map(verdict(from:)),
        // Pre-#1214 rows knew only the block's level.
        level: ParameterLevel(
          tempoBpm: attempt.levelTempoBpm ?? blockLevel.tempoBpm,
          clickLevel: attempt.levelClickLevel.map(clickLevel(from:)) ?? blockLevel.clickLevel))
    }
  }

  private static func decodeRungs(_ json: String) throws -> [Rung] {
    let raw: [String]
    do { raw = try JSONDecoder().decode([String].self, from: Data(json.utf8)) } catch {
      throw CoachCodecError(field: "escalation_fired", phase: "decode")
    }
    return raw.map(rung(from:))
  }

  private static func verdictString(_ verdict: Verdict) -> String {
    switch verdict {
    case .clean: "clean"
    case .missed: "missed"
    }
  }

  private static func verdict(from raw: String) -> Verdict {
    switch raw {
    case "clean": return .clean
    case "missed": return .missed
    default:
      report(UnknownStoredEnum(kind: "Verdict", raw: raw), decodeContext)
      return .missed  // conservative: an unknown verdict must not inflate mastery (#949)
    }
  }

  private static func evidenceSource(from raw: String) -> EvidenceSource {
    switch raw {
    case "tap_verdict": return .tapVerdict
    case "tap_verdict_untimed": return .tapVerdictUntimed
    case "midi": return .midi
    case "audio": return .audio
    default:
      report(UnknownStoredEnum(kind: "EvidenceSource", raw: raw), decodeContext)
      // Not `.tapVerdictUntimed`: an untimed source on a clocked rung is a row
      // the engine can never write, and a fallback must not invent one.
      return .tapVerdict
    }
  }

  private static func clickLevel(from raw: String) -> ClickLevel {
    switch raw {
    case "no_click": return .noClick
    case "every_beat": return .everyBeat
    case "two_and_four": return .twoAndFour
    case "bar_downbeat": return .barDownbeat
    case "every_other_bar": return .everyOtherBar
    default:
      report(UnknownStoredEnum(kind: "ClickLevel", raw: raw), decodeContext)
      return .everyBeat
    }
  }

  private static func circle(from raw: String) -> Circle {
    switch raw {
    case "head": return .head
    case "hands": return .hands
    case "bridge": return .bridge
    default:
      report(UnknownStoredEnum(kind: "Circle", raw: raw), decodeContext)
      return .hands
    }
  }

  private static func mode(from raw: String) -> Mode {
    switch raw {
    case "keys": return .keys
    case "away": return .away
    case "keys_to_away": return .keysToAway
    default:
      report(UnknownStoredEnum(kind: "Mode", raw: raw), decodeContext)
      return .keys
    }
  }

  private static func rung(from raw: String) -> Rung {
    switch raw {
    case "tempo_down": return .tempoDown
    case "shrink_scope": return .shrinkScope
    case "change_mode": return .changeMode
    case "swap_drill": return .swapDrill
    default:
      report(UnknownStoredEnum(kind: "Rung", raw: raw), decodeContext)
      return .tempoDown  // conservative: the mildest rung
    }
  }

  private static func exit(from raw: String) -> Exit {
    switch raw {
    case "gate_passed": return .gatePassed
    case "ceiling_hit": return .ceilingHit
    case "skipped": return .skipped
    case "escalated": return .escalated
    case "session_ended": return .sessionEnded
    default:
      report(UnknownStoredEnum(kind: "Exit", raw: raw), decodeContext)
      return .sessionEnded  // conservative: an unknown exit must not replay a level-up (#949)
    }
  }

  private static func evidenceSourceString(_ source: EvidenceSource) -> String {
    switch source {
    case .tapVerdict: "tap_verdict"
    case .tapVerdictUntimed: "tap_verdict_untimed"
    case .midi: "midi"
    case .audio: "audio"
    }
  }

  private static func clickLevelString(_ level: ClickLevel) -> String {
    switch level {
    case .noClick: "no_click"
    case .everyBeat: "every_beat"
    case .twoAndFour: "two_and_four"
    case .barDownbeat: "bar_downbeat"
    case .everyOtherBar: "every_other_bar"
    }
  }

  private static func circleString(_ circle: Circle) -> String {
    switch circle {
    case .head: "head"
    case .hands: "hands"
    case .bridge: "bridge"
    }
  }

  private static func modeString(_ mode: Mode) -> String {
    switch mode {
    case .keys: "keys"
    case .away: "away"
    case .keysToAway: "keys_to_away"
    }
  }

  private static func exitString(_ exit: Exit) -> String {
    switch exit {
    case .gatePassed: "gate_passed"
    case .ceilingHit: "ceiling_hit"
    case .skipped: "skipped"
    case .escalated: "escalated"
    case .sessionEnded: "session_ended"
    }
  }

  private static func rungString(_ rung: Rung) -> String {
    switch rung {
    case .tempoDown: "tempo_down"
    case .shrinkScope: "shrink_scope"
    case .changeMode: "change_mode"
    case .swapDrill: "swap_drill"
    }
  }

  // ── Built-session entities (#1256) ───────────────────────────────────

  func saveUserDrill(_ drill: UserDrill) throws {
    try dbQueue.write { db in
      try db.execute(
        sql: """
          INSERT INTO user_drill
            (id, name, criterion, tempo_bpm, keys, passes_to_open, serves_kind, serves_value,
             created_at, updated_at, deleted_at)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
          ON CONFLICT(id) DO UPDATE SET
            name = excluded.name, criterion = excluded.criterion,
            tempo_bpm = excluded.tempo_bpm, keys = excluded.keys,
            passes_to_open = excluded.passes_to_open,
            serves_kind = excluded.serves_kind, serves_value = excluded.serves_value,
            created_at = excluded.created_at, updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at
          """,
        arguments: [
          drill.id, drill.name, drill.criterion, drill.tempoBpm.map { Int($0) },
          try Self.encodeJSON(drill.keys, field: "keys"), Int(drill.passesToOpen),
          Self.servesKind(drill.serves), Self.servesValue(drill.serves),
          drill.createdAt, drill.updatedAt, drill.deletedAt,
        ])
    }
  }

  func saveJournalItem(_ journal: JournalItem) throws {
    try dbQueue.write { db in
      try db.execute(
        sql: """
          INSERT INTO journal_item
            (id, name, notes, linked_item_id, created_at, updated_at, deleted_at)
          VALUES (?, ?, ?, ?, ?, ?, ?)
          ON CONFLICT(id) DO UPDATE SET
            name = excluded.name, notes = excluded.notes,
            linked_item_id = excluded.linked_item_id,
            created_at = excluded.created_at, updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at
          """,
        arguments: [
          journal.id, journal.name, journal.notes, journal.linkedItemId,
          journal.createdAt, journal.updatedAt, journal.deletedAt,
        ])
    }
  }

  func saveBuiltSession(_ session: BuiltSession) throws {
    try dbQueue.write { db in
      try db.execute(
        sql: """
          INSERT INTO built_session
            (id, source, blocks, created_at, updated_at, deleted_at)
          VALUES (?, ?, ?, ?, ?, ?)
          ON CONFLICT(id) DO UPDATE SET
            source = excluded.source, blocks = excluded.blocks,
            created_at = excluded.created_at, updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at
          """,
        arguments: [
          session.id, session.source, try Self.encodeBlocks(session.blocks),
          session.createdAt, session.updatedAt, session.deletedAt,
        ])
    }
  }

  func savePlayThrough(_ record: PlayThroughRecord) throws {
    try dbQueue.write { db in
      try db.execute(
        sql: """
          INSERT INTO play_through
            (id, item_id, started_at, ended_at, counted, sections, updated_at, deleted_at)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?)
          ON CONFLICT(id) DO UPDATE SET
            item_id = excluded.item_id, started_at = excluded.started_at,
            ended_at = excluded.ended_at, counted = excluded.counted,
            sections = excluded.sections, updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at
          """,
        arguments: [
          record.id, record.itemId, record.startedAt, record.endedAt, record.counted,
          try Self.encodeSections(record.sections), record.updatedAt, record.deletedAt,
        ])
    }
  }

  func saveReflection(_ reflection: Reflection) throws {
    try dbQueue.write { db in
      try db.execute(
        sql: """
          INSERT INTO reflection
            (id, kind, session_ref, transcript, audio_path, duration_s, at, updated_at, deleted_at)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
          ON CONFLICT(id) DO UPDATE SET
            kind = excluded.kind, session_ref = excluded.session_ref,
            transcript = excluded.transcript, audio_path = excluded.audio_path,
            duration_s = excluded.duration_s, at = excluded.at,
            updated_at = excluded.updated_at, deleted_at = excluded.deleted_at
          """,
        arguments: [
          reflection.id, Self.reflectionKindString(reflection.kind), reflection.sessionRef,
          reflection.transcript, reflection.audioPath, reflection.durationS.map { Int($0) },
          reflection.at, reflection.updatedAt, reflection.deletedAt,
        ])
    }
  }

  func saveFeelEntry(_ entry: FeelEntry) throws {
    try dbQueue.write { db in
      try db.execute(
        sql: """
          INSERT INTO feel_entry
            (id, block_id, feel, at, updated_at, deleted_at)
          VALUES (?, ?, ?, ?, ?, ?)
          ON CONFLICT(id) DO UPDATE SET
            block_id = excluded.block_id, feel = excluded.feel, at = excluded.at,
            updated_at = excluded.updated_at, deleted_at = excluded.deleted_at
          """,
        arguments: [
          entry.id, entry.blockId, Self.feelString(entry.feel), entry.at,
          entry.updatedAt, entry.deletedAt,
        ])
    }
  }

  func loadBuiltSessionData() throws -> BuiltSessionData {
    try dbQueue.read { db in
      BuiltSessionData(
        userDrills: try Row.fetchAll(
          db, sql: "SELECT * FROM user_drill WHERE deleted_at IS NULL ORDER BY created_at, id"
        ).map(Self.userDrill(from:)),
        journalItems: try Row.fetchAll(
          db, sql: "SELECT * FROM journal_item WHERE deleted_at IS NULL ORDER BY created_at, id"
        ).map(Self.journalItem(from:)),
        builtSessions: try Row.fetchAll(
          db, sql: "SELECT * FROM built_session WHERE deleted_at IS NULL ORDER BY created_at, id"
        ).map(Self.builtSession(from:)),
        playThroughs: try Row.fetchAll(
          db, sql: "SELECT * FROM play_through WHERE deleted_at IS NULL ORDER BY started_at, id"
        ).map(Self.playThrough(from:)),
        reflections: try Row.fetchAll(
          db, sql: "SELECT * FROM reflection WHERE deleted_at IS NULL ORDER BY at, id"
        ).map(Self.reflection(from:)),
        feelEntries: try Row.fetchAll(
          db, sql: "SELECT * FROM feel_entry WHERE deleted_at IS NULL ORDER BY at, id"
        ).map(Self.feelEntry(from:)))
    }
  }

  // ── Row ↔ built-session codecs ───────────────────────────────────────

  private struct StoredBuiltBlock: Codable {
    var id: String
    var targetKind: String
    var targetValue: String
    var minutes: UInt16?
  }

  private struct StoredSectionVerdict: Codable {
    var section: String
    var held: Bool
    var at: String
  }

  private static func userDrill(from row: Row) throws -> UserDrill {
    UserDrill(
      id: row["id"], name: row["name"], criterion: row["criterion"],
      tempoBpm: (row["tempo_bpm"] as Int?).map { UInt16(clamping: $0) },
      keys: try decodeKeys(row["keys"]),
      passesToOpen: UInt8(clamping: row["passes_to_open"] as Int),
      serves: serves(kind: row["serves_kind"], value: row["serves_value"]),
      createdAt: row["created_at"], updatedAt: row["updated_at"], deletedAt: row["deleted_at"])
  }

  private static func decodeKeys(_ json: String) throws -> [String] {
    do { return try JSONDecoder().decode([String].self, from: Data(json.utf8)) } catch {
      throw CoachCodecError(field: "keys", phase: "decode")
    }
  }

  private static func journalItem(from row: Row) -> JournalItem {
    JournalItem(
      id: row["id"], name: row["name"], notes: row["notes"],
      linkedItemId: row["linked_item_id"],
      createdAt: row["created_at"], updatedAt: row["updated_at"], deletedAt: row["deleted_at"])
  }

  private static func builtSession(from row: Row) throws -> BuiltSession {
    BuiltSession(
      id: row["id"], source: row["source"], blocks: try decodeBlocks(row["blocks"]),
      createdAt: row["created_at"], updatedAt: row["updated_at"], deletedAt: row["deleted_at"])
  }

  private static func playThrough(from row: Row) throws -> PlayThroughRecord {
    PlayThroughRecord(
      id: row["id"], itemId: row["item_id"],
      startedAt: row["started_at"], endedAt: row["ended_at"], counted: row["counted"],
      sections: try decodeSections(row["sections"]),
      updatedAt: row["updated_at"], deletedAt: row["deleted_at"])
  }

  private static func reflection(from row: Row) -> Reflection {
    Reflection(
      id: row["id"], kind: reflectionKind(from: row["kind"]), sessionRef: row["session_ref"],
      transcript: row["transcript"], audioPath: row["audio_path"],
      durationS: (row["duration_s"] as Int?).map { UInt32(clamping: $0) },
      at: row["at"], updatedAt: row["updated_at"], deletedAt: row["deleted_at"])
  }

  private static func feelEntry(from row: Row) -> FeelEntry {
    FeelEntry(
      id: row["id"], blockId: row["block_id"], feel: feel(from: row["feel"]),
      at: row["at"], updatedAt: row["updated_at"], deletedAt: row["deleted_at"])
  }

  private static func encodeBlocks(_ blocks: [BuiltBlock]) throws -> String {
    try encodeJSON(
      blocks.map { block in
        StoredBuiltBlock(
          id: block.id, targetKind: targetKind(block.target),
          targetValue: targetValue(block.target), minutes: block.minutes)
      }, field: "blocks")
  }

  /// Throws on a broken blob (a lost block is invariant-5 territory); an
  /// unrecognised target kind reports and drops that block only (#949) —
  /// the rest of the composition stays readable.
  private static func decodeBlocks(_ json: String) throws -> [BuiltBlock] {
    let stored: [StoredBuiltBlock]
    do { stored = try JSONDecoder().decode([StoredBuiltBlock].self, from: Data(json.utf8)) } catch {
      throw CoachCodecError(field: "blocks", phase: "decode")
    }
    return stored.compactMap { block in
      guard let target = target(kind: block.targetKind, value: block.targetValue) else {
        return nil
      }
      return BuiltBlock(id: block.id, target: target, minutes: block.minutes)
    }
  }

  private static func encodeSections(_ sections: [SectionVerdict]) throws -> String {
    try encodeJSON(
      sections.map { StoredSectionVerdict(section: $0.section, held: $0.held, at: $0.at) },
      field: "sections")
  }

  private static func decodeSections(_ json: String) throws -> [SectionVerdict] {
    let stored: [StoredSectionVerdict]
    do {
      stored = try JSONDecoder().decode([StoredSectionVerdict].self, from: Data(json.utf8))
    } catch {
      throw CoachCodecError(field: "sections", phase: "decode")
    }
    return stored.map { SectionVerdict(section: $0.section, held: $0.held, at: $0.at) }
  }

  private static func targetKind(_ target: BuiltTarget) -> String {
    switch target {
    case .node: "node"
    case .userDrill: "user_drill"
    case .journal: "journal"
    case .piece: "piece"
    }
  }

  private static func targetValue(_ target: BuiltTarget) -> String {
    switch target {
    case .node(let node): node
    case .userDrill(let drillId): drillId
    case .journal(let journalId): journalId
    case .piece(let itemId): itemId
    }
  }

  private static func target(kind: String, value: String) -> BuiltTarget? {
    switch kind {
    case "node": return .node(node: value)
    case "user_drill": return .userDrill(drillId: value)
    case "journal": return .journal(journalId: value)
    case "piece": return .piece(itemId: value)
    default:
      report(UnknownStoredEnum(kind: "BuiltTarget", raw: kind), decodeContext)
      return nil
    }
  }

  private static func servesKind(_ serves: Serves?) -> String? {
    switch serves {
    case .circle: "circle"
    case .node: "node"
    case nil: nil
    }
  }

  private static func servesValue(_ serves: Serves?) -> String? {
    switch serves {
    case .circle(let circle): circleString(circle)
    case .node(let node): node
    case nil: nil
    }
  }

  private static func serves(kind: String?, value: String?) -> Serves? {
    switch (kind, value) {
    case (nil, _), (_, nil): return nil
    case ("circle", .some(let value)): return .circle(circle(from: value))
    case ("node", .some(let value)): return .node(value)
    case (.some(let other), _):
      report(UnknownStoredEnum(kind: "Serves", raw: other), decodeContext)
      return nil
    }
  }

  private static func reflectionKindString(_ kind: ReflectionKind) -> String {
    switch kind {
    case .voiceNote: "voice_note"
    case .sessionClose: "session_close"
    }
  }

  private static func reflectionKind(from raw: String) -> ReflectionKind {
    switch raw {
    case "voice_note": return .voiceNote
    case "session_close": return .sessionClose
    default:
      report(UnknownStoredEnum(kind: "ReflectionKind", raw: raw), decodeContext)
      return .voiceNote
    }
  }

  private static func feelString(_ feel: Feel) -> String {
    switch feel {
    case .foughtIt: "fought_it"
    case .gettingThere: "getting_there"
    case .itSang: "it_sang"
    }
  }

  private static func feel(from raw: String) -> Feel {
    switch raw {
    case "fought_it": return .foughtIt
    case "getting_there": return .gettingThere
    case "it_sang": return .itSang
    default:
      report(UnknownStoredEnum(kind: "Feel", raw: raw), decodeContext)
      return .gettingThere  // conservative: the neutral middle, never the success tint
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
