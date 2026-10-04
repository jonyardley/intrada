import SharedTypes
import XCTest

@testable import Intrada

final class LibraryStoreTests: XCTestCase {
  private func makeStore() throws -> LibraryStore { try LibraryStore.inMemory() }

  private func item(
    _ id: String, title: String = "Etude", kind: ItemKind = .piece,
    createdAt: String = "2026-01-01T00:00:00Z"
  ) -> Item {
    LibraryItemFixture.record(
      id: id, title: title, kind: kind, composer: "Chopin",
      key: Key(letter: .c, accidental: .natural, mode: .major),
      tempo: Tempo(marking: "Allegro", bpm: 132), notes: "evenness",
      tags: ["scale", "warmup"], createdAt: createdAt, priority: true)
  }

  func testSaveThenLoadRoundTrips() throws {
    let store = try makeStore()
    try store.save(item("p1"))
    let loaded = try store.loadItems()
    XCTAssertEqual(loaded.count, 1)
    let got = try XCTUnwrap(loaded.first)
    XCTAssertEqual(got.id, "p1")
    XCTAssertEqual(got.kind, .piece)
    XCTAssertEqual(got.key, Key(letter: .c, accidental: .natural, mode: .major))
    XCTAssertEqual(got.tempo, Tempo(marking: "Allegro", bpm: 132))
    XCTAssertEqual(got.tags, ["scale", "warmup"])
    XCTAssertTrue(got.priority)
  }

  func testBatchSaveUpsertsEveryRow() throws {
    let store = try makeStore()
    try store.save([item("a", title: "One"), item("b", title: "Two"), item("c", title: "Three")])
    let loaded = try store.loadItems()
    XCTAssertEqual(Swift.Set(loaded.map(\.id)), ["a", "b", "c"], "all batch rows land")
  }

  func testExerciseKindAndNilTempoRoundTrip() throws {
    let store = try makeStore()
    var ex = item("e1", kind: .exercise)
    ex.tempo = nil
    ex.tags = []
    try store.save(ex)
    let got = try XCTUnwrap(try store.loadItems().first)
    XCTAssertEqual(got.kind, .exercise)
    XCTAssertNil(got.tempo)
    XCTAssertEqual(got.tags, [])
  }

  func testAKeyWithNoModeRoundTrips() throws {
    let store = try makeStore()
    var it = item("p1")
    it.key = Key(letter: .f, accidental: .sharp, mode: nil)
    try store.save(it)
    XCTAssertEqual(try XCTUnwrap(try store.loadItems().first).key, it.key)
  }

  /// Upgrade path: a v1 row survives the v2 `modality` migration intact.
  func testV1RowSurvivesModalityMigration() throws {
    let store = try LibraryStore.upgradeTestStore(
      migratedTo: "v1_item",
      seed: """
        INSERT INTO item
          (id, title, kind, composer, key, tempo_marking, tempo_bpm, notes, tags,
           created_at, updated_at, priority, deleted_at)
        VALUES ('p1', 'Legacy Etude', 'piece', 'Bach', 'C', 'Allegro', 120, 'phrasing',
                '["scale"]', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, NULL)
        """)
    let loaded = try store.loadItems()
    XCTAssertEqual(loaded.count, 1, "the pre-existing v1 row must survive the v2 migration")
    let got = try XCTUnwrap(loaded.first)
    XCTAssertEqual(got.id, "p1")
    XCTAssertEqual(got.title, "Legacy Etude")
    XCTAssertEqual(got.composer, "Bach")
    XCTAssertEqual(
      got.key, Key(letter: .c, accidental: .natural, mode: nil),
      "v2 adds modality as NULL for pre-existing rows")
    XCTAssertEqual(got.tempo, Tempo(marking: "Allegro", bpm: 120))
    XCTAssertEqual(got.tags, ["scale"])
    XCTAssertFalse(got.priority)
  }

  func testUpsertUpdatesInPlace() throws {
    let store = try makeStore()
    try store.save(item("p1", title: "Before"))
    try store.save(item("p1", title: "After"))
    let loaded = try store.loadItems()
    XCTAssertEqual(loaded.count, 1)
    XCTAssertEqual(loaded.first?.title, "After")
  }

  func testDeleteTombstonesAndHidesFromLoad() throws {
    let store = try makeStore()
    try store.save(item("p1"))
    try store.delete(id: "p1", deletedAt: "2026-01-02T00:00:00+00:00")
    XCTAssertTrue(try store.loadItems().isEmpty)
  }

  func testSaveRevivesADeletedRow() throws {
    let store = try makeStore()
    try store.save(item("p1"))
    try store.delete(id: "p1", deletedAt: "2026-01-02T00:00:00+00:00")
    try store.save(item("p1", title: "Revived"))
    let loaded = try store.loadItems()
    XCTAssertEqual(loaded.count, 1)
    XCTAssertEqual(loaded.first?.title, "Revived")
  }

  func testLoadOrdersNewestFirst() throws {
    let store = try makeStore()
    try store.save(item("old", title: "Old", createdAt: "2026-01-01T00:00:00Z"))
    try store.save(item("new", title: "New", createdAt: "2026-02-01T00:00:00Z"))
    XCTAssertEqual(try store.loadItems().map(\.id), ["new", "old"])
  }

  /// Offline-first invariant #2 (CLAUDE.md): persisted tables carry the sync columns.
  func testSchemaHasSyncColumns() throws {
    let columns = try makeStore().columnNames(ofTable: "item")
    XCTAssertTrue(
      columns.contains("updated_at"), "item table must carry updated_at; has \(columns)")
    XCTAssertTrue(
      columns.contains("deleted_at"), "item table must carry deleted_at; has \(columns)")
  }

  // ── Variations and keys (#2246) ──────────────────────────────────────

  private func variation(_ id: String, label: String, deletedAt: String? = nil) -> Variation {
    Variation(id: id, label: label, updatedAt: "2026-07-01T00:00:00+00:00", deletedAt: deletedAt)
  }

  func testVariationRowsRoundTripWithTombstonesIntact() throws {
    let store = try makeStore()
    try store.save([
      variation("v-dot", label: "Dotted rhythms"),
      variation("v-hs", label: "Hands separately", deletedAt: "2026-07-02T00:00:00+00:00"),
    ])

    let got = try store.loadVariations().sorted { $0.id < $1.id }
    XCTAssertEqual(got.map(\.label), ["Dotted rhythms", "Hands separately"])
    XCTAssertEqual(got[0].updatedAt, "2026-07-01T00:00:00+00:00")
    XCTAssertEqual(
      got[1].deletedAt, "2026-07-02T00:00:00+00:00",
      "the tombstone survives the round trip; no hard deletes")
  }

  func testVariationUpsertUpdatesRowsInPlace() throws {
    let store = try makeStore()
    try store.save([variation("v-dot", label: "Dotted")])
    try store.save([variation("v-dot", label: "Dotted rhythms")])

    XCTAssertEqual(try store.loadVariations().map(\.label), ["Dotted rhythms"])
  }

  func testAnItemsVariationsAndKeysRoundTripInOrder() throws {
    let store = try makeStore()
    var ex = item("e1", kind: .exercise)
    ex.variationIds = ["v-b", "v-a"]
    ex.keys = [
      Key(letter: .e, accidental: .flat, mode: .major),
      Key(letter: .c, accidental: .sharp, mode: .minor),
      Key(letter: .g, accidental: .natural, mode: nil),
    ]
    try store.save(ex)

    let got = try XCTUnwrap(try store.loadItems().first)
    XCTAssertEqual(got.variationIds, ["v-b", "v-a"])
    XCTAssertEqual(got.keys, ex.keys)
  }

  func testVariationSchemaHasSyncColumns() throws {
    let columns = try makeStore().columnNames(ofTable: "variation")
    XCTAssertTrue(
      columns.contains("updated_at"), "variation table must carry updated_at; has \(columns)")
    XCTAssertTrue(
      columns.contains("deleted_at"), "variation table must carry deleted_at; has \(columns)")
  }

  // ── Sessions ──────────────────────────────────────────────────────────

  private func entry(_ id: String) -> SetlistEntry {
    SetlistEntry(
      id: id, itemId: "item-\(id)", itemTitle: "Etude", itemType: .exercise, position: 0,
      durationSecs: 300, status: .completed, notes: "good", intention: "evenness",
      plannedDurationSecs: 300, groupId: nil, plannedVariationIds: [],
      plannedRepTarget: 5,
      plays: [
        Play(
          id: "\(id)-p1", sectionId: nil, key: nil, variationIds: [],
          startedAt: "2026-01-01T00:00:00Z", seconds: 300,
          repTarget: 5, repCount: 5,
          repHistory: [
            RepEvent(action: .success, at: "2026-01-01T00:01:00Z", tempo: nil, clickSounding: nil),
            RepEvent(action: .missed, at: "2026-01-01T00:02:00Z", tempo: nil, clickSounding: nil),
            RepEvent(action: .success, at: "2026-01-01T00:03:30Z", tempo: nil, clickSounding: nil),
          ], tempoChanges: [], achievedTempo: 120, clickPattern: nil, score: 4, away: [])
      ], segments: [], focus: nil, intentionMet: nil, felt: nil, gotInTheWay: [], notePoints: [])
  }

  private func session(_ id: String, completedAt: String = "2026-01-01T00:10:00Z")
    -> PracticeSession
  {
    PracticeSession(
      id: id, entries: [entry("a"), entry("b")], sessionNotes: "solid",
      startedAt: "2026-01-01T00:00:00Z", completedAt: completedAt,
      totalDurationSecs: 600, completionStatus: .completed, sessionScore: nil, captureVersion: nil)
  }

  func testSaveThenLoadSessionRoundTrips() throws {
    let store = try makeStore()
    try store.saveSession(session("s1"))
    let got = try XCTUnwrap(try store.loadSessions().first)
    XCTAssertEqual(got.id, "s1")
    XCTAssertEqual(got.startedAt, "2026-01-01T00:00:00Z")
    XCTAssertEqual(got.completedAt, "2026-01-01T00:10:00Z")
    XCTAssertEqual(got.totalDurationSecs, 600)
    XCTAssertEqual(got.completionStatus, .completed)
    XCTAssertEqual(got.sessionNotes, "solid")
    XCTAssertEqual(got.entries.count, 2)
    let e = try XCTUnwrap(got.entries.first)
    XCTAssertEqual(e.itemType, .exercise)
    XCTAssertEqual(e.status, .completed)
    XCTAssertEqual(e.plannedRepTarget, 5)
    let play = try XCTUnwrap(e.plays.first)
    XCTAssertEqual(play.score, 4)
    XCTAssertEqual(play.repTarget, 5)
    XCTAssertEqual(
      play.repHistory,
      [
        RepEvent(action: .success, at: "2026-01-01T00:01:00Z", tempo: nil, clickSounding: nil),
        RepEvent(action: .missed, at: "2026-01-01T00:02:00Z", tempo: nil, clickSounding: nil),
        RepEvent(action: .success, at: "2026-01-01T00:03:30Z", tempo: nil, clickSounding: nil),
      ], "each tap keeps its own time through the blob")
    XCTAssertEqual(play.achievedTempo, 120)
  }

  /// Rows written before #1367 hold bare action strings with no time. They
  /// must still decode, and the session start stands in for the missing `at`.
  func testLegacyUntimedRepHistoryDecodesAgainstTheSessionStart() throws {
    let entries =
      #"[{"id":"e1","itemId":"i1","itemTitle":"X","itemType":"piece","position":0,"durationSecs":0,"status":"completed","repTarget":5,"repCount":1,"repHistory":["success","missed"]}]"#
    let store = try LibraryStore.upgradeTestStore(
      migratedTo: "v3_session",
      seed: """
        INSERT INTO item
          (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
           created_at, updated_at, priority, deleted_at)
        VALUES ('i1', 'X', 'piece', NULL, NULL, NULL, NULL, NULL, NULL, '[]',
                '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, NULL);
        INSERT INTO session
          (id, started_at, completed_at, total_duration_secs, completion_status,
           session_notes, session_intention, entries, updated_at, deleted_at)
        VALUES ('s1', '2026-01-01T09:00:00Z', '2026-01-01T09:30:00Z', 0, 'completed',
                NULL, NULL, '\(entries)', '2026-01-01T09:30:00Z', NULL)
        """)
    let e = try XCTUnwrap(try store.loadSessions().first?.entries.first)
    let play = try XCTUnwrap(e.plays.first, "a legacy completed entry folds into one play")
    XCTAssertEqual(
      play.repHistory,
      [
        RepEvent(action: .success, at: "2026-01-01T09:00:00Z", tempo: nil, clickSounding: nil),
        RepEvent(action: .missed, at: "2026-01-01T09:00:00Z", tempo: nil, clickSounding: nil),
      ], "an untimed legacy tap is dated to the session start, never dropped")
  }

  func testEndedEarlyAndEmptyEntriesRoundTrip() throws {
    let store = try makeStore()
    var s = session("s2")
    s.entries = []
    s.completionStatus = .endedEarly
    s.sessionNotes = nil
    try store.saveSession(s)
    let got = try XCTUnwrap(try store.loadSessions().first)
    XCTAssertEqual(got.completionStatus, .endedEarly)
    XCTAssertTrue(got.entries.isEmpty)
    XCTAssertNil(got.sessionNotes)
  }

  func testSkippedAndNotAttemptedEntryStatusRoundTrip() throws {
    let store = try makeStore()
    var s = session("s3")
    s.entries[0].status = .skipped
    s.entries[1].status = .notAttempted
    try store.saveSession(s)
    let got = try XCTUnwrap(try store.loadSessions().first)
    XCTAssertEqual(got.entries.map(\.status), [.skipped, .notAttempted])
  }

  func testEntriesBlobRoundTripsThePlanAndTheWayPlayed() throws {
    let store = try makeStore()
    var s = session("s1")
    s.entries[0].segments = [Segment(sectionId: "sec-a", plannedSecs: 300)]
    s.entries[0].plannedVariationIds = ["v-c", "v-d"]
    s.entries[0].plays[0].sectionId = "sec-a"
    s.entries[0].plays[0].key = Key(letter: .e, accidental: .flat, mode: .major)
    s.entries[0].plays[0].variationIds = ["v-c", "v-d"]
    s.entries[0].plays[0].tempoChanges = [
      TempoChange(at: "2026-01-01T00:01:00Z", tempo: 84, clickSounding: true)
    ]
    s.entries[0].plays[0].repHistory = [
      RepEvent(action: .success, at: "2026-01-01T00:01:00Z", tempo: 96, clickSounding: true)
    ]
    try store.saveSession(s)

    let got = try XCTUnwrap(try store.loadSessions().first)
    XCTAssertEqual(
      got.entries[0].segments, [Segment(sectionId: "sec-a", plannedSecs: 300)],
      "the plan rides the blob")
    XCTAssertEqual(got.entries[0].plannedVariationIds, ["v-c", "v-d"])
    XCTAssertEqual(got.entries[0].plays.first, s.entries[0].plays[0], "and so does the play")
    XCTAssertEqual(got.entries[1].plannedVariationIds, [])
    XCTAssertNil(got.entries[1].plays.first?.key)
  }

  func testTheCaptureVersionRoundTrips() throws {
    let store = try makeStore()
    var s = session("s1")
    s.captureVersion = 1
    try store.saveSession(s)
    XCTAssertEqual(try XCTUnwrap(try store.loadSessions().first).captureVersion, 1)
  }

  func testEntriesBlobRoundTripsTheClickPattern() throws {
    let store = try makeStore()
    var s = session("s1")
    let pattern = ClickState(
      metre: Metre(beats: 7, unit: 8, groups: [3, 2, 2]), sounding: 0b0101001)
    s.entries[0].plays[0].clickPattern = pattern
    try store.saveSession(s)

    let got = try XCTUnwrap(try store.loadSessions().first)
    XCTAssertEqual(
      got.entries[0].plays.first?.clickPattern, pattern, "the pattern rides the blob")
    XCTAssertNil(got.entries[1].plays.first?.clickPattern)
  }

  func testSessionsLoadNewestFirst() throws {
    let store = try makeStore()
    try store.saveSession(session("old", completedAt: "2026-01-01T00:00:00Z"))
    try store.saveSession(session("new", completedAt: "2026-02-01T00:00:00Z"))
    XCTAssertEqual(try store.loadSessions().map(\.id), ["new", "old"])
  }

  func testSessionSchemaHasSyncColumns() throws {
    let columns = try makeStore().columnNames(ofTable: "session")
    XCTAssertTrue(columns.contains("updated_at"), "session needs updated_at; has \(columns)")
    XCTAssertTrue(columns.contains("deleted_at"), "session needs deleted_at; has \(columns)")
  }

  /// Unknown stored enum strings (an older binary reading a newer row) fall back
  /// to conservative defaults rather than silently miscategorising (#949).
  func testUnknownStoredEnumStringsFallBackToConservativeDefaults() throws {
    // The rep action rides a second, completed entry: an unknown status folds to
    // notAttempted, which keeps no play, so e1 has nowhere to carry a tap.
    let entries =
      #"[{"id":"e1","itemId":"i1","itemTitle":"X","itemType":"klingon","position":0,"durationSecs":0,"status":"quantum"},{"id":"e2","itemId":"i1","itemTitle":"X","itemType":"piece","position":1,"durationSecs":0,"status":"completed","repHistory":["warp"]}]"#
    let store = try LibraryStore.upgradeTestStore(
      migratedTo: "v3_session",
      seed: """
        INSERT INTO item
          (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
           created_at, updated_at, priority, deleted_at)
        VALUES ('i1', 'X', 'piece', NULL, NULL, 'lydian', NULL, NULL, NULL, '[]',
                '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, NULL);
        INSERT INTO session
          (id, started_at, completed_at, total_duration_secs, completion_status,
           session_notes, session_intention, entries, updated_at, deleted_at)
        VALUES ('s1', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, 'enlightenment',
                NULL, NULL, '\(entries)', '2026-01-01T00:00:00Z', NULL)
        """)
    XCTAssertNil(try XCTUnwrap(try store.loadItems().first).key, "unknown modality, no key")

    let got = try XCTUnwrap(try store.loadSessions().first)
    XCTAssertEqual(got.completionStatus, .completed, "unknown completion status → completed")
    let e = try XCTUnwrap(got.entries.first)
    XCTAssertEqual(
      e.status, .notAttempted, "unknown entry status → conservative notAttempted, not completed")
    XCTAssertEqual(e.itemType, .piece, "unknown kind → piece")
    let played = try XCTUnwrap(got.entries.last?.plays.first)
    XCTAssertEqual(
      played.repHistory?.map(\.action), [.missed], "unknown rep action → conservative missed")
  }

  /// Upgrade path: a pre-existing v2 item row survives the v3 session migration.
  func testV2ItemSurvivesSessionMigration() throws {
    let store = try LibraryStore.upgradeTestStore(
      migratedTo: "v2_add_modality",
      seed: """
        INSERT INTO item
          (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
           created_at, updated_at, priority, deleted_at)
        VALUES ('p1', 'Legacy', 'piece', 'Bach', 'C', 'major', 'Allegro', 120, 'x',
                '["scale"]', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, NULL)
        """)
    XCTAssertEqual(try store.loadItems().count, 1, "the v2 item survives the v3 migration")
    XCTAssertTrue(try store.loadSessions().isEmpty, "the new session table starts empty")
  }

  /// A JSON column that will not decode reads back as empty and never fails the
  /// load: one damaged row must not take the whole library with it (#1117).
  func testDamagedJsonColumnsReadBackEmptyWithoutFailingTheLoad() throws {
    let store = try LibraryStore.upgradeTestStore(
      migratedTo: "v17_item_metre",
      seed: """
        INSERT INTO item
          (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
           linked_exercise_ids, created_at, updated_at, priority, chord_chart, photo_id, metre,
           deleted_at)
        VALUES ('i1', 'X', 'piece', NULL, NULL, NULL, NULL, NULL, NULL, 'not json', '{',
                '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, '[', NULL, 'x', NULL);
        INSERT INTO session
          (id, started_at, completed_at, total_duration_secs, completion_status,
           session_notes, session_intention, entries, updated_at, deleted_at)
        VALUES ('s1', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0, 'completed',
                NULL, NULL, '{', '2026-01-01T00:00:00Z', NULL)
        """)
    let item = try XCTUnwrap(try store.loadItems().first)
    XCTAssertEqual(item.tags, [])
    XCTAssertEqual(item.exerciseLinks, [])
    XCTAssertNil(item.chordChart)
    XCTAssertNil(item.metre)
    XCTAssertEqual(try XCTUnwrap(try store.loadSessions().first).entries, [])
  }

  // #2005: an unreadable list reads back empty, so a save that writes that
  // empty list back must not destroy what is still on disk.
  func testSavingAnItemKeepsItsUnreadableListsUntilItsListsChange() throws {
    let store = try LibraryStore.upgradeTestStore(
      migratedTo: "v17_item_metre",
      seed: """
        INSERT INTO item
          (id, title, kind, tags, linked_exercise_ids, created_at, updated_at, priority)
        VALUES ('i1', 'X', 'piece', 'not json', '[1, 2]',
                '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0)
        """)
    var loaded = try XCTUnwrap(try store.loadItems().first)
    loaded.title = "Renamed"
    try store.save(loaded)
    XCTAssertEqual(try store.rawText("tags", ofItem: "i1"), "not json")
    XCTAssertEqual(try store.rawText("linked_exercise_ids", ofItem: "i1"), "[1, 2]")

    loaded.tags = ["scales"]
    loaded.exerciseLinks = [
      ExerciseLink(
        id: "l1", exerciseId: "e1", sectionId: nil, position: 0,
        updatedAt: "2026-10-04T09:00:00Z", deletedAt: nil)
    ]
    try store.save(loaded)
    XCTAssertEqual(try store.rawText("tags", ofItem: "i1"), #"["scales"]"#)
    XCTAssertEqual(
      try store.rawText("linked_exercise_ids", ofItem: "i1"), "[1, 2]",
      "links live in their own table, so the old list is never written (#2248)")
  }

  func testSavingAnEmptyListOverReadableTagsClearsThem() throws {
    let store = try makeStore()
    var saved = item("i1")
    try store.save(saved)
    saved.tags = []
    try store.save(saved)
    XCTAssertEqual(try store.rawText("tags", ofItem: "i1"), "[]")
  }
}
