import Foundation
import GRDB
import SharedTypes
import Testing

@testable import Intrada

/// Coach evidence persistence (#1181, spec §4). The production codec is
/// write-only, so the stored shape is pinned here: `DecodedAttempt` is the
/// format contract a future reader has to match.
@Suite("Coach evidence store")
struct CoachRecordStoreTests {

  private struct DecodedAttempt: Codable {
    var at: String
    var verdict: String
    var source: String
    var cold: Bool
    var selfPredicted: String?
  }

  private static let updatedAt = "2026-08-04T10:00:30Z"

  private func attempt(
    clean: Bool, cold: Bool = false, at: String = "2026-08-04T10:00:09Z",
    source: EvidenceSource = .tapVerdict, selfPredicted: Verdict? = nil
  ) -> AttemptSummary {
    AttemptSummary(
      at: at, verdict: clean ? .clean : .missed, source: source, cold: cold,
      selfPredicted: selfPredicted)
  }

  private func block(
    _ id: String = "b1", attempts: [AttemptSummary] = [], escalations: [Rung] = [],
    exit: Exit = .gatePassed, attemptsToPass: UInt16? = 3, gateOpenedAtAttempt: UInt16? = 3,
    repsAfterGate: UInt16 = 0, activeMs: UInt64 = 30_000
  ) -> BlockRecord {
    BlockRecord(
      id: id, node: "rootless-a-b", drill: "shell-voicings", gate: "rootless-under-melody",
      level: ParameterLevel(tempoBpm: 92, clickLevel: .twoAndFour),
      circle: .hands, mode: .keys,
      startedAt: "2026-08-04T10:00:00Z", endedAt: Self.updatedAt,
      attempts: attempts, attemptsToPass: attemptsToPass,
      gateOpenedAtAttempt: gateOpenedAtAttempt, repsAfterGate: repsAfterGate,
      activeMs: activeMs, escalationFired: escalations, exit: exit)
  }

  private func wander(
    _ id: String = "w1", attempts: [AttemptSummary] = [], keepAsDrill: Bool? = nil
  ) -> WanderRecord {
    WanderRecord(
      id: id, startedAt: "2026-08-04T10:05:00Z", endedAt: "2026-08-04T10:09:00Z",
      attempts: attempts, keepAsDrill: keepAsDrill)
  }

  /// A store plus its queue, so a write-only codec can still be read back in
  /// tests with raw SQL rather than a production decoder nothing would call.
  private func makeStore() throws -> (LibraryStore, DatabaseQueue) {
    let queue = try DatabaseQueue()
    return (try LibraryStore(queue), queue)
  }

  private func row(_ queue: DatabaseQueue, _ sql: String) throws -> Row? {
    try queue.read { db in try Row.fetchOne(db, sql: sql) }
  }

  private func count(_ queue: DatabaseQueue, _ table: String) throws -> Int {
    try queue.read { db in try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM \(table)") ?? 0 }
  }

  // ── Schema (offline-first invariant 2) ────────────────────────────────

  @Test("both evidence tables carry the sync columns")
  func syncColumns() throws {
    let (store, _) = try makeStore()
    for table in ["block_record", "wander_record"] {
      let columns = try store.columnNames(ofTable: table)
      #expect(columns.contains("updated_at"), "\(table) must carry updated_at; has \(columns)")
      #expect(columns.contains("deleted_at"), "\(table) must carry deleted_at; has \(columns)")
    }
  }

  // ── Writes ────────────────────────────────────────────────────────────

  @Test("a closed block writes every column it was given")
  func blockColumns() throws {
    let (store, queue) = try makeStore()
    try store.saveCoachRecords(
      blocks: [block(escalations: [.tempoDown, .shrinkScope], exit: .ceilingHit)],
      wanders: [], updatedAt: Self.updatedAt)

    let row = try #require(try self.row(queue, "SELECT * FROM block_record WHERE id = 'b1'"))
    #expect(row["node"] as String? == "rootless-a-b")
    #expect(row["drill"] as String? == "shell-voicings")
    #expect(row["gate"] as String? == "rootless-under-melody")
    #expect(row["level_tempo_bpm"] as Int? == 92)
    #expect(row["level_click_level"] as String? == "two_and_four")
    #expect(row["circle"] as String? == "hands")
    #expect(row["mode"] as String? == "keys")
    #expect(row["started_at"] as String? == "2026-08-04T10:00:00Z")
    #expect(row["ended_at"] as String? == Self.updatedAt)
    #expect(row["attempts_to_pass"] as Int? == 3)
    #expect(row["gate_opened_at_attempt"] as Int? == 3)
    #expect(row["reps_after_gate"] as Int? == 0)
    #expect(row["active_ms"] as Int? == 30_000)
    #expect(row["exit"] as String? == "ceiling_hit")
    #expect(row["updated_at"] as String? == Self.updatedAt, "the core stamps updated_at")
    #expect(row["deleted_at"] as String? == nil, "a fresh record carries no tombstone")
  }

  @Test("a block that never passed stores its nullable counters as NULL")
  func blockNullableCounters() throws {
    let (store, queue) = try makeStore()
    try store.saveCoachRecords(
      blocks: [block(exit: .escalated, attemptsToPass: nil, gateOpenedAtAttempt: nil)],
      wanders: [], updatedAt: Self.updatedAt)

    let row = try #require(try self.row(queue, "SELECT * FROM block_record WHERE id = 'b1'"))
    #expect(row["attempts_to_pass"] as Int? == nil, "never passed reads back as NULL, not 0")
    #expect(row["gate_opened_at_attempt"] as Int? == nil)
  }

  @Test("attempts store as JSON keeping verdict, source and the cold flag")
  func attemptsBlob() throws {
    let (store, queue) = try makeStore()
    let attempts = [
      attempt(clean: false, cold: true, at: "2026-08-04T10:00:09Z"),
      attempt(clean: true, at: "2026-08-04T10:00:18Z", source: .midi, selfPredicted: .clean),
    ]
    try store.saveCoachRecords(
      blocks: [block(attempts: attempts)], wanders: [], updatedAt: Self.updatedAt)

    let row = try #require(try self.row(queue, "SELECT attempts FROM block_record WHERE id = 'b1'"))
    let json = try #require(row["attempts"] as String?)
    let decoded = try JSONDecoder().decode([DecodedAttempt].self, from: Data(json.utf8))

    #expect(decoded.count == 2, "attempts keep their order and none is dropped")
    #expect(decoded[0].verdict == "missed")
    #expect(decoded[0].source == "tap_verdict")
    #expect(decoded[0].cold, "the cold flag is the highest-information bit; it must survive")
    #expect(decoded[0].at == "2026-08-04T10:00:09Z")
    #expect(decoded[0].selfPredicted == nil)
    #expect(decoded[1].verdict == "clean")
    #expect(decoded[1].source == "midi", "a machine-scored attempt is expressible today")
    #expect(!decoded[1].cold)
    #expect(decoded[1].selfPredicted == "clean")
  }

  @Test("the escalation ladder stores as JSON in the order it fired")
  func escalationBlob() throws {
    let (store, queue) = try makeStore()
    try store.saveCoachRecords(
      blocks: [block(escalations: [.tempoDown, .changeMode, .swapDrill])],
      wanders: [], updatedAt: Self.updatedAt)

    let row = try #require(
      try self.row(queue, "SELECT escalation_fired FROM block_record WHERE id = 'b1'"))
    let json = try #require(row["escalation_fired"] as String?)
    let rungs = try JSONDecoder().decode([String].self, from: Data(json.utf8))
    #expect(rungs == ["tempo_down", "change_mode", "swap_drill"])
  }

  @Test("a block with no attempts and no escalations stores empty arrays")
  func emptyBlobs() throws {
    let (store, queue) = try makeStore()
    try store.saveCoachRecords(
      blocks: [block(exit: .skipped)], wanders: [], updatedAt: Self.updatedAt)

    let row = try #require(try self.row(queue, "SELECT * FROM block_record WHERE id = 'b1'"))
    #expect(row["attempts"] as String? == "[]")
    #expect(row["escalation_fired"] as String? == "[]")
  }

  @Test("a wander writes its own row with the keep prompt unanswered")
  func wanderColumns() throws {
    let (store, queue) = try makeStore()
    try store.saveCoachRecords(
      blocks: [], wanders: [wander(attempts: [attempt(clean: true)])],
      updatedAt: Self.updatedAt)

    let row = try #require(try self.row(queue, "SELECT * FROM wander_record WHERE id = 'w1'"))
    #expect(row["started_at"] as String? == "2026-08-04T10:05:00Z")
    #expect(row["ended_at"] as String? == "2026-08-04T10:09:00Z")
    #expect(row["keep_as_drill"] as Bool? == nil, "NULL is not yet asked, not a no")
    #expect(row["updated_at"] as String? == Self.updatedAt)
    #expect(row["deleted_at"] as String? == nil)

    let json = try #require(row["attempts"] as String?)
    let decoded = try JSONDecoder().decode([DecodedAttempt].self, from: Data(json.utf8))
    #expect(decoded.count == 1, "a wander still captures its attempts")
  }

  @Test("a block and a wander in one batch both land")
  func batchWritesBoth() throws {
    let (store, queue) = try makeStore()
    try store.saveCoachRecords(
      blocks: [block("b1"), block("b2")], wanders: [wander("w1")], updatedAt: Self.updatedAt)

    #expect(try count(queue, "block_record") == 2)
    #expect(try count(queue, "wander_record") == 1)
  }

  @Test("answering the keep prompt updates the wander row rather than duplicating it")
  func wanderUpsert() throws {
    let (store, queue) = try makeStore()
    try store.saveCoachRecords(blocks: [], wanders: [wander()], updatedAt: Self.updatedAt)
    try store.saveCoachRecords(
      blocks: [], wanders: [wander(keepAsDrill: true)], updatedAt: "2026-08-04T10:10:00Z")

    #expect(try count(queue, "wander_record") == 1, "the same id upserts, never duplicates")
    let row = try #require(try self.row(queue, "SELECT * FROM wander_record WHERE id = 'w1'"))
    #expect(row["keep_as_drill"] as Bool? == true)
    #expect(row["updated_at"] as String? == "2026-08-04T10:10:00Z")
  }

  @Test("rewriting a record after a failed write is idempotent")
  func blockUpsertIsIdempotent() throws {
    let (store, queue) = try makeStore()
    let record = block(attempts: [attempt(clean: true)])
    try store.saveCoachRecords(blocks: [record], wanders: [], updatedAt: Self.updatedAt)
    try store.saveCoachRecords(blocks: [record], wanders: [], updatedAt: Self.updatedAt)

    #expect(try count(queue, "block_record") == 1, "a retried write must not double-count evidence")
  }

  // ── Upgrade path (CLAUDE.md "Local data migrations") ───────────────────

  @Test("a database populated at v9 migrates to v10 with its data intact")
  func upgradeFromV9() throws {
    // Populate at v9 (no evidence tables), then finish the chain. The device is
    // the only copy of this data, so the assertion is that it survives.
    let queue = try DatabaseQueue()
    try LibraryStore.migrator.migrate(queue, upTo: "v9_variant")
    try queue.write { db in
      try db.execute(
        sql: """
          INSERT INTO item
            (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
             linked_exercise_ids, created_at, updated_at, priority, deleted_at, chord_chart)
          VALUES ('e-legacy', 'Legacy Exercise', 'exercise', NULL, NULL, NULL, NULL, NULL,
                  'keep me', '["scales"]', '[]', '2026-01-01T00:00:00Z',
                  '2026-01-01T00:00:00Z', 1, NULL, NULL)
          """)
      try db.execute(
        sql: """
          INSERT INTO variant (id, item_id, label, position, updated_at, deleted_at)
          VALUES ('v-c', 'e-legacy', 'C', 0, '2026-01-01T00:00:00Z', NULL)
          """)
      try db.execute(
        sql: """
          INSERT INTO session
            (id, started_at, completed_at, total_duration_secs, completion_status,
             session_notes, session_intention, entries, updated_at, deleted_at, session_score,
             reflection_improved, reflection_still_rough, reflection_next_target)
          VALUES ('s-legacy', '2026-01-01T00:00:00Z', '2026-01-01T00:30:00Z', 1800, 'completed',
                  'old note', NULL, '[]', '2026-01-01T00:00:00Z', NULL, 7, NULL, NULL, NULL)
          """)
    }

    let store = try LibraryStore(queue)

    let item = try #require(try store.loadItems().first, "the pre-v10 item survives")
    #expect(item.id == "e-legacy")
    #expect(item.notes == "keep me")
    #expect(item.tags == ["scales"])
    #expect(item.priority)
    #expect(item.variants.map(\.label) == ["C"], "its step ladder survives too")

    let session = try #require(try store.loadSessions().first, "the pre-v10 session survives")
    #expect(session.sessionNotes == "old note")
    #expect(session.sessionScore == 7)

    // The new tables arrive empty and usable, not merely present.
    #expect(try count(queue, "block_record") == 0)
    #expect(try count(queue, "wander_record") == 0)
    try store.saveCoachRecords(
      blocks: [block(attempts: [attempt(clean: true, cold: true)])], wanders: [wander()],
      updatedAt: Self.updatedAt)
    #expect(try count(queue, "block_record") == 1, "a migrated database accepts evidence")
    #expect(try count(queue, "wander_record") == 1)
  }

  @Test("the migration chain runs cleanly on a fresh database")
  func freshDatabaseReachesHead() throws {
    let queue = try DatabaseQueue()
    _ = try LibraryStore(queue)
    let applied = try queue.read { db in try LibraryStore.migrator.appliedMigrations(db) }
    #expect(applied.contains("v10_coach_records"), "applied \(applied)")
  }
}
