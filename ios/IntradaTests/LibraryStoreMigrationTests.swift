import GRDB
import SharedTypes
import XCTest

@testable import Intrada

final class LibraryStoreMigrationTests: XCTestCase {
  func testV5RescalesEntryScoresAndAddsSessionScore() throws {
    let queue = try DatabaseQueue()  // in-memory

    // Migrate only up to v3 (the pre-rescale schema), then seed a session
    // whose single entry scored 3 on the old 1–5 scale.
    try LibraryStore.migrator.migrate(queue, upTo: "v3_session")
    try queue.write { db in
      let entriesJSON =
        #"[{"id":"e1","itemId":"i1","itemTitle":"Scales","itemType":"exercise","position":0,"durationSecs":60,"status":"completed","score":3}]"#
      try db.execute(
        sql: """
          INSERT INTO session (id, started_at, completed_at, total_duration_secs,
            completion_status, session_notes, session_intention, entries, updated_at, deleted_at)
          VALUES ('s1','2026-01-01T00:00:00Z','2026-01-01T00:01:00Z',60,'completed',NULL,NULL,?, '2026-01-01T00:00:00Z',NULL)
          """, arguments: [entriesJSON])
    }

    // Run the remaining migrations (v4 add column, v5 rescale).
    try LibraryStore.migrator.migrate(queue)

    try queue.read { db in
      let row = try Row.fetchOne(
        db, sql: "SELECT entries, session_score FROM session WHERE id='s1'")!
      let entries: String = row["entries"]
      XCTAssertTrue(entries.contains("\"score\":6"), "old score 3 should rescale ×2 to 6")
      XCTAssertNil(row["session_score"] as Int64?, "session_score column exists, null for old rows")
    }
  }

  func testSessionScoreRoundTrip() throws {
    let store = try LibraryStore.inMemory()
    let entry = SetlistEntry(
      id: "e1", itemId: "i1", itemTitle: "Scales", itemType: .exercise,
      position: 0, durationSecs: 60, status: .completed,
      notes: nil, score: 8, intention: nil, repTarget: nil, repCount: nil,
      repTargetReached: nil, repHistory: nil, plannedDurationSecs: nil, achievedTempo: nil,
      groupId: nil)
    let session = PracticeSession(
      id: "sess-rt", entries: [entry],
      sessionNotes: nil, sessionIntention: nil,
      startedAt: "2026-01-01T10:00:00Z", completedAt: "2026-01-01T10:30:00Z",
      totalDurationSecs: 1800, completionStatus: .completed, sessionScore: 7,
      reflectionImproved: nil, reflectionStillRough: nil, reflectionNextTarget: nil)
    try store.saveSession(session)
    let loaded = try store.loadSessions()
    XCTAssertEqual(loaded.count, 1)
    XCTAssertEqual(
      loaded[0].sessionScore, 7,
      "sessionScore UInt8→Int64→UInt8(clamping:) round-trip must preserve 7")
  }

  func testSessionReflectionsRoundTrip() throws {
    let store = try LibraryStore.inMemory()
    let session = PracticeSession(
      id: "sess-refl", entries: [],
      sessionNotes: nil, sessionIntention: "even RH at 96",
      startedAt: "2026-07-14T10:00:00Z", completedAt: "2026-07-14T10:30:00Z",
      totalDurationSecs: 1800, completionStatus: .completed, sessionScore: nil,
      reflectionImproved: "thumb-unders even at 92",
      reflectionStillRough: "bars 12-14 rush past 88",
      reflectionNextTarget: "bars 12-14 at 80, hands together")
    try store.saveSession(session)
    let loaded = try XCTUnwrap(try store.loadSessions().first)
    XCTAssertEqual(loaded.reflectionImproved, "thumb-unders even at 92")
    XCTAssertEqual(loaded.reflectionStillRough, "bars 12-14 rush past 88")
    XCTAssertEqual(loaded.reflectionNextTarget, "bars 12-14 at 80, hands together")
  }

  func testV6SessionSurvivesReflectionMigration() throws {
    let store = try LibraryStore.upgradeTestStore(
      migratedTo: "v6_item_linked_exercises",
      seed: """
        INSERT INTO session
          (id, started_at, completed_at, total_duration_secs, completion_status,
           session_notes, session_intention, entries, updated_at, deleted_at, session_score)
        VALUES ('s-pre', '2026-01-01T00:00:00Z', '2026-01-01T00:01:00Z', 60, 'completed',
                'old note', NULL, '[]', '2026-01-01T00:00:00Z', NULL, 7)
        """)
    let loaded = try XCTUnwrap(try store.loadSessions().first)
    XCTAssertEqual(loaded.sessionNotes, "old note", "pre-migration row survives intact")
    XCTAssertEqual(loaded.sessionScore, 7)
    XCTAssertNil(loaded.reflectionImproved, "old rows read back with nil reflections")
    XCTAssertNil(loaded.reflectionStillRough)
    XCTAssertNil(loaded.reflectionNextTarget)
  }

  func testGroupIdRoundTripsThroughTheJsonCodec() throws {
    let store = try LibraryStore.inMemory()
    let entry = SetlistEntry(
      id: "e1", itemId: "i1", itemTitle: "Scales", itemType: .exercise,
      position: 0, durationSecs: 60, status: .completed,
      notes: nil, score: nil, intention: nil, repTarget: nil, repCount: nil,
      repTargetReached: nil, repHistory: nil, plannedDurationSecs: nil, achievedTempo: nil,
      groupId: "block-1")
    let session = PracticeSession(
      id: "sess-g", entries: [entry],
      sessionNotes: nil, sessionIntention: nil,
      startedAt: "2026-01-01T10:00:00Z", completedAt: "2026-01-01T10:30:00Z",
      totalDurationSecs: 60, completionStatus: .completed, sessionScore: nil,
      reflectionImproved: nil, reflectionStillRough: nil, reflectionNextTarget: nil)
    try store.saveSession(session)
    let loaded = try store.loadSessions()
    XCTAssertEqual(
      loaded.first?.entries.first?.groupId, "block-1",
      "group_id round-trips through the JSON-blob codec")
  }

  func testV5RescaleClampNilAndBlobPreservation() throws {
    let queue = try DatabaseQueue()

    // Seed at v3: one entry with score 5 (boundary — should clamp to 10) and notes,
    // one entry with no score (null — must remain null after rescale).
    try LibraryStore.migrator.migrate(queue, upTo: "v3_session")
    try queue.write { db in
      let entriesJSON =
        #"[{"id":"e1","itemId":"i1","itemTitle":"Bach","itemType":"piece","position":0,"durationSecs":120,"status":"completed","score":5,"notes":"keep"},{"id":"e2","itemId":"i2","itemTitle":"Scales","itemType":"exercise","position":1,"durationSecs":60,"status":"completed"}]"#
      try db.execute(
        sql: """
          INSERT INTO session (id, started_at, completed_at, total_duration_secs,
            completion_status, session_notes, session_intention, entries, updated_at, deleted_at)
          VALUES ('s2','2026-01-02T00:00:00Z','2026-01-02T00:02:00Z',120,'completed',NULL,NULL,?, '2026-01-02T00:00:00Z',NULL)
          """, arguments: [entriesJSON])
    }

    try LibraryStore.migrator.migrate(queue)

    try queue.read { db in
      let row = try Row.fetchOne(db, sql: "SELECT entries FROM session WHERE id='s2'")!
      let entries: String = row["entries"]
      // Score 5 × 2 = 10 — at the clamp boundary.
      XCTAssertTrue(entries.contains("\"score\":10"), "score 5 must clamp to 10 after ×2 rescale")
      // The notes field on entry e1 must survive the decode→re-encode round-trip.
      XCTAssertTrue(
        entries.contains("\"notes\":\"keep\""), "notes field must survive blob re-encode")
      // Entry e2 had no score key — must still have no score after rescale.
      XCTAssertFalse(entries.contains("\"score\":0"), "null-score entry must not gain a zero score")
    }
  }

  // ── v6: linked_exercise_ids ───────────────────────────────────────────

  func testV6AddsLinkedExerciseIdsColumnDefaultingToEmptyArray() throws {
    // Populate at v5 (no linked_exercise_ids column), insert an item row, then finish.
    let store = try LibraryStore.upgradeTestStore(
      migratedTo: "v5_rescale_entry_scores",
      seed: """
        INSERT INTO item
          (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
           created_at, updated_at, priority, deleted_at)
        VALUES ('p1', 'Legacy Piece', 'piece', NULL, NULL, NULL, NULL, NULL, NULL, '[]',
                '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, NULL)
        """)
    let columns = try store.columnNames(ofTable: "item")
    XCTAssertTrue(
      columns.contains("linked_exercise_ids"),
      "v6 must add linked_exercise_ids column; got \(columns)")

    let loaded = try store.loadItems()
    XCTAssertEqual(loaded.count, 1, "pre-existing row must survive v6 migration")
    XCTAssertEqual(
      loaded[0].linkedExerciseIds, [],
      "pre-existing row gets empty-array default for linked_exercise_ids")
  }

  func testV6LinkedExerciseIdsRoundTrip() throws {
    let store = try LibraryStore.inMemory()
    let item = Item(
      id: "p2", title: "Étude", kind: .piece, composer: nil, key: nil, modality: nil,
      tempo: nil, notes: nil, tags: [], linkedExerciseIds: ["e1", "e2"],
      createdAt: "2026-01-01T00:00:00Z", updatedAt: "2026-01-01T00:00:00Z", priority: false)
    try store.save(item)
    let loaded = try store.loadItems()
    XCTAssertEqual(loaded.count, 1)
    XCTAssertEqual(
      loaded[0].linkedExerciseIds, ["e1", "e2"],
      "linked_exercise_ids must round-trip through JSON storage intact")
  }
}
