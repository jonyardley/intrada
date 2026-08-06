import Foundation
import GRDB
import SharedTypes
import Testing

@testable import Intrada

/// Coach evidence persistence (#1181 writes, #1214 reads; spec §4). The writes
/// are asserted against raw SQL rather than through the decoder, so the stored
/// shape stays pinned independently of it: `DecodedAttempt` is that contract.
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
    source: EvidenceSource = .tapVerdict, selfPredicted: Verdict? = nil,
    level: ParameterLevel = ParameterLevel(tempoBpm: 92, clickLevel: .twoAndFour)
  ) -> AttemptSummary {
    AttemptSummary(
      at: at, verdict: clean ? .clean : .missed, source: source, cold: cold,
      selfPredicted: selfPredicted, level: level)
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

  // ── Reads (#1214) ─────────────────────────────────────────────────────

  @Test("a written block reads back with every field the mastery replay needs")
  func readBackRoundTrips() throws {
    let (store, _) = try makeStore()
    let attempts = [
      attempt(clean: false, cold: true, at: "2026-08-04T10:00:09Z"),
      attempt(clean: true, at: "2026-08-04T10:00:18Z", source: .midi, selfPredicted: .clean),
    ]
    let written = block(
      attempts: attempts, escalations: [.tempoDown, .swapDrill], exit: .ceilingHit,
      attemptsToPass: 4, gateOpenedAtAttempt: 2, repsAfterGate: 1, activeMs: 42_000)
    try store.saveCoachRecords(blocks: [written], wanders: [], updatedAt: Self.updatedAt)

    let read = try #require(try store.loadCoachRecords().first)
    #expect(read == written, "the decoder must give the core back what it wrote")
  }

  @Test("a block that never passed reads its nullable counters back as nil")
  func readBackNullableCounters() throws {
    let (store, _) = try makeStore()
    try store.saveCoachRecords(
      blocks: [block(exit: .escalated, attemptsToPass: nil, gateOpenedAtAttempt: nil)],
      wanders: [], updatedAt: Self.updatedAt)

    let read = try #require(try store.loadCoachRecords().first)
    #expect(read.attemptsToPass == nil, "NULL reads back as never-passed, not 0")
    #expect(read.gateOpenedAtAttempt == nil)
  }

  @Test("a tombstoned block is not replayed into the mastery track")
  func readBackSkipsTombstones() throws {
    let (store, queue) = try makeStore()
    try store.saveCoachRecords(
      blocks: [block("b1"), block("b2")], wanders: [], updatedAt: Self.updatedAt)
    try queue.write { db in
      try db.execute(
        sql: "UPDATE block_record SET deleted_at = ? WHERE id = 'b2'",
        arguments: [Self.updatedAt])
    }

    #expect(try store.loadCoachRecords().map(\.id) == ["b1"])
  }

  @Test("an unrecognised stored enum is reported rather than silently defaulted")
  func readBackUnknownEnum() throws {
    // An older binary reading a row a newer version wrote (#949). The row must
    // still decode — losing a whole block's evidence is worse than one field
    // falling back — but it must not do so silently.
    let (store, queue) = try makeStore()
    try store.saveCoachRecords(blocks: [block()], wanders: [], updatedAt: Self.updatedAt)
    try queue.write { db in
      try db.execute(sql: "UPDATE block_record SET exit = 'teleported' WHERE id = 'b1'")
    }

    let read = try #require(try store.loadCoachRecords().first)
    #expect(read.exit == .sessionEnded, "an unknown exit reads as the neutral one")
  }

  @Test("an attempt keeps the rung it was played at, not the one the block closed on")
  func readBackPerAttemptLevel() throws {
    // The ladder's first rung drops the tempo mid-block, so a block that
    // escalated holds attempts belonging to two rungs. Losing that puts the
    // pre-drop evidence on the post-drop rung (#1214).
    let (store, _) = try makeStore()
    let before = ParameterLevel(tempoBpm: 120, clickLevel: .twoAndFour)
    let after = ParameterLevel(tempoBpm: 96, clickLevel: .twoAndFour)
    try store.saveCoachRecords(
      blocks: [
        block(
          attempts: [
            attempt(clean: false, at: "2026-08-04T10:00:09Z", level: before),
            attempt(clean: true, at: "2026-08-04T10:00:18Z", level: after),
          ], escalations: [.tempoDown], exit: .gatePassed)
      ], wanders: [], updatedAt: Self.updatedAt)

    let read = try #require(try store.loadCoachRecords().first)
    #expect(read.attempts.map(\.level) == [before, after])
  }

  @Test("every click level survives the round trip, since it is half the mastery key")
  func readBackEveryClickLevel() throws {
    let (store, _) = try makeStore()
    let levels: [ClickLevel] = [.everyBeat, .twoAndFour, .barDownbeat, .everyOtherBar]
    try store.saveCoachRecords(
      blocks: levels.enumerated().map { index, level in
        block(
          "b\(index)",
          attempts: [
            attempt(
              clean: true, at: "2026-08-04T10:0\(index):09Z",
              level: ParameterLevel(tempoBpm: 80, clickLevel: level))
          ])
      }, wanders: [], updatedAt: Self.updatedAt)

    let read = try store.loadCoachRecords()
    #expect(
      read.compactMap { $0.attempts.first?.level.clickLevel } == levels,
      "a decoder that drifts from its encoder relocates evidence silently")
  }

  @Test("a row written before the per-attempt level falls back to the block's")
  func readBackLegacyAttemptWithoutLevel() throws {
    let (store, queue) = try makeStore()
    try store.saveCoachRecords(blocks: [block()], wanders: [], updatedAt: Self.updatedAt)
    try queue.write { db in
      try db.execute(
        sql: "UPDATE block_record SET attempts = ? WHERE id = 'b1'",
        arguments: [
          """
          [{"at":"2026-08-04T10:00:09Z","verdict":"clean","source":"tap_verdict","cold":false}]
          """
        ])
    }

    let read = try #require(try store.loadCoachRecords().first)
    #expect(
      read.attempts.first?.level == read.level,
      "the block's level is all a pre-#1214 row ever knew; it must not be dropped")
  }

  @Test("an attempts blob that will not decode fails the read rather than emptying it")
  func readBackCorruptAttemptsThrows() throws {
    // Returning [] would replay as a block nobody played, which the mastery
    // track then reads as material never practised (invariant 5).
    let (store, queue) = try makeStore()
    try store.saveCoachRecords(
      blocks: [block(attempts: [attempt(clean: true)])], wanders: [], updatedAt: Self.updatedAt)
    try queue.write { db in
      try db.execute(sql: "UPDATE block_record SET attempts = 'not json' WHERE id = 'b1'")
    }

    #expect(throws: (any Error).self) { try store.loadCoachRecords() }
  }

  @Test("blocks read back oldest first, whatever order they were written")
  func readBackOrder() throws {
    let (store, _) = try makeStore()
    try store.saveCoachRecords(
      blocks: [lateBlock, block("b-early")], wanders: [], updatedAt: Self.updatedAt)

    #expect(try store.loadCoachRecords().map(\.id) == ["b-early", "b-late"])
  }

  private var lateBlock: BlockRecord {
    BlockRecord(
      id: "b-late", node: "rootless-a-b", drill: "rootless-one-key",
      gate: "rootless-one-key",
      level: ParameterLevel(tempoBpm: 60, clickLevel: .everyBeat),
      circle: .hands, mode: .keys,
      startedAt: "2026-08-05T10:00:00Z", endedAt: "2026-08-05T10:00:30Z",
      attempts: [], attemptsToPass: nil, gateOpenedAtAttempt: nil, repsAfterGate: 0,
      activeMs: 0, escalationFired: [], exit: .skipped)
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
