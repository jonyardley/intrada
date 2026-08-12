import Foundation
import SharedTypes
import Testing

@testable import Intrada

/// Row↔type codecs for the built-session entities (#1256 Phase A). Every
/// entity must survive save → loadBuiltSessionData unchanged, tombstones
/// must stay out of loads, and saves must upsert by id.
struct BuiltSessionStoreTests {
  private func sampleDrill(id: String = "01USERDRILL0000000000000001") -> UserDrill {
    UserDrill(
      id: id, name: "Descending run", criterion: "Three clean passes at 72",
      tempoBpm: 72, keys: ["F", "Bb"], passesToOpen: 3,
      serves: .node("alice-bridge"),
      createdAt: "2026-08-07T10:00:00Z", updatedAt: "2026-08-07T10:00:00Z", deletedAt: nil)
  }

  @Test func userDrillRoundTrips() throws {
    let store = try LibraryStore.inMemory()
    let drill = sampleDrill()
    try store.saveUserDrill(drill)
    let data = try store.loadBuiltSessionData()
    #expect(data.userDrills == [drill])
  }

  @Test func userDrillServesCircleRoundTrips() throws {
    let store = try LibraryStore.inMemory()
    var drill = sampleDrill()
    drill.serves = .circle(.hands)
    try store.saveUserDrill(drill)
    let data = try store.loadBuiltSessionData()
    #expect(data.userDrills.first?.serves == .circle(.hands))
  }

  @Test func userDrillUpsertsById() throws {
    let store = try LibraryStore.inMemory()
    try store.saveUserDrill(sampleDrill())
    var edited = sampleDrill()
    edited.criterion = "Five clean passes"
    edited.updatedAt = "2026-08-07T11:00:00Z"
    try store.saveUserDrill(edited)
    let data = try store.loadBuiltSessionData()
    #expect(data.userDrills == [edited])
  }

  @Test func tombstonedUserDrillStaysOutOfLoads() throws {
    let store = try LibraryStore.inMemory()
    var drill = sampleDrill()
    drill.deletedAt = "2026-08-07T11:00:00Z"
    try store.saveUserDrill(drill)
    let data = try store.loadBuiltSessionData()
    #expect(data.userDrills.isEmpty)
  }

  @Test func journalItemRoundTrips() throws {
    let store = try LibraryStore.inMemory()
    let journal = JournalItem(
      id: "01JOURNAL000000000000000001", name: "Rubato feel",
      notes: "Time and notes", linkedItemId: "01PIECE0000000000000000001",
      createdAt: "2026-08-07T10:00:00Z", updatedAt: "2026-08-07T10:00:00Z", deletedAt: nil)
    try store.saveJournalItem(journal)
    let data = try store.loadBuiltSessionData()
    #expect(data.journalItems == [journal])
  }

  @Test func builtSessionRoundTripsWithEveryTargetKind() throws {
    let store = try LibraryStore.inMemory()
    let session = BuiltSession(
      id: "01BUILT0000000000000000001", source: "From Friday's lesson",
      blocks: [
        BuiltBlock(id: "b1", target: .node(node: "shell-voicings"), minutes: 5),
        BuiltBlock(id: "b2", target: .userDrill(drillId: "d1"), minutes: nil),
        BuiltBlock(id: "b3", target: .journal(journalId: "j1"), minutes: 8),
        BuiltBlock(id: "b4", target: .piece(itemId: "p1"), minutes: 10),
      ],
      createdAt: "2026-08-07T10:00:00Z", updatedAt: "2026-08-07T10:00:00Z", deletedAt: nil)
    try store.saveBuiltSession(session)
    let data = try store.loadBuiltSessionData()
    #expect(data.builtSessions == [session])
  }

  @Test func playThroughRoundTripsWithSections() throws {
    let store = try LibraryStore.inMemory()
    let record = PlayThroughRecord(
      id: "01PLAY00000000000000000001", itemId: "p1",
      startedAt: "2026-08-07T10:00:00Z", endedAt: "2026-08-07T10:04:00Z",
      counted: true,
      sections: [
        SectionVerdict(section: "The bridge, from memory", held: true, at: "2026-08-07T10:01:00Z"),
        SectionVerdict(section: "Out head", held: false, at: "2026-08-07T10:03:00Z"),
      ],
      updatedAt: "2026-08-07T10:04:00Z", deletedAt: nil)
    try store.saveCoach(
      blocks: [], wanders: [], playThroughs: [record], updatedAt: "2026-08-07T10:04:00Z")
    let data = try store.loadBuiltSessionData()
    #expect(data.playThroughs == [record])
  }

  @Test func reflectionRoundTripsForBothKinds() throws {
    let store = try LibraryStore.inMemory()
    let kinds: [(String, ReflectionKind)] = [("r1", .voiceNote), ("r2", .sessionClose)]
    for (id, kind) in kinds {
      try store.saveReflection(
        Reflection(
          id: id, kind: kind, sessionRef: "01BUILT0000000000000000001",
          transcript: "The bridge still rushes", audioPath: "reflections/\(id).m4a",
          durationS: 24, at: "2026-08-07T10:30:00Z", steer: .unoffered, steerAt: nil,
          updatedAt: "2026-08-07T10:30:00Z", deletedAt: nil))
    }
    let data = try store.loadBuiltSessionData()
    #expect(data.reflections.count == 2)
    #expect(data.reflections.map(\.kind) == [.voiceNote, .sessionClose])
  }

  @Test func reflectionRoundTripsForEverySteerState() throws {
    let store = try LibraryStore.inMemory()
    let states: [(String, SteerState)] = [
      ("r1", .unoffered), ("r2", .accepted), ("r3", .declined),
    ]
    for (id, steer) in states {
      try store.saveReflection(
        Reflection(
          id: id, kind: .sessionClose, sessionRef: nil,
          transcript: "The bridge still rushes", audioPath: nil,
          durationS: 24, at: "2026-08-07T22:00:00Z", steer: steer,
          steerAt: steer == .unoffered ? nil : "2026-08-08T09:00:00Z",
          updatedAt: "2026-08-08T09:00:00Z", deletedAt: nil))
    }
    let data = try store.loadBuiltSessionData()
    #expect(data.reflections.map(\.steer) == [.unoffered, .accepted, .declined])
    #expect(data.reflections[1].steerAt == "2026-08-08T09:00:00Z")
  }

  /// #1256 Phase D, v15. A reflection written before the morning card existed
  /// was never offered a steer, and must not arrive as one on upgrade.
  @Test func reflectionsFromBeforeTheSteerColumnUpgradeAsUnoffered() throws {
    let store = try LibraryStore.upgradeTestStore(
      migratedTo: "v14_unmonitored_play",
      seed: """
        INSERT INTO reflection
          (id, kind, session_ref, transcript, audio_path, duration_s, at, updated_at, deleted_at)
        VALUES ('01REFLECTION00000000000001', 'session_close', NULL,
          'The bridge still rushes', 'reflections/r1.m4a', 24,
          '2026-07-01T22:00:00Z', '2026-07-01T22:00:00Z', NULL);
        """)
    let data = try store.loadBuiltSessionData()
    #expect(data.reflections.count == 1, "the historic reflection survives the upgrade")
    #expect(data.reflections[0].transcript == "The bridge still rushes")
    #expect(data.reflections[0].steer == .unoffered)
    #expect(data.reflections[0].steerAt == nil)
  }

  // ── #1269: an unreadable value quarantines its row ──────────────────
  //
  // Phase B loads a built session, edits it and saves it back, so a row
  // decoded *partially* would overwrite the only copy of the user's data with
  // less than it had. The row stays on disk untouched instead.

  private func storeSeeded(_ seed: String) throws -> LibraryStore {
    try LibraryStore.upgradeTestStore(migratedTo: "v11_built_session", seed: seed)
  }

  @Test func aSessionWithAnUnknownBlockKindIsLeftOutWholeRatherThanPartial() throws {
    let store = try storeSeeded(
      """
      INSERT INTO built_session (id, source, blocks, created_at, updated_at, deleted_at)
      VALUES ('01BUILT0000000000000000001', 'From Friday''s lesson',
        '[{"id":"b1","targetKind":"node","targetValue":"shell-voicings","minutes":5},
          {"id":"b2","targetKind":"from_the_future","targetValue":"x","minutes":5}]',
        '2026-08-07T10:00:00Z', '2026-08-07T10:00:00Z', NULL);
      """)
    let data = try store.loadBuiltSessionData()
    #expect(
      data.builtSessions.isEmpty,
      "a session missing a block is the wrong session, not a smaller one")
  }

  @Test func quarantineIsPerRowNotPerLoad() throws {
    let store = try storeSeeded(
      """
      INSERT INTO built_session (id, source, blocks, created_at, updated_at, deleted_at)
      VALUES ('01BUILT0000000000000000001', NULL,
        '[{"id":"b1","targetKind":"from_the_future","targetValue":"x","minutes":5}]',
        '2026-08-07T10:00:00Z', '2026-08-07T10:00:00Z', NULL);
      INSERT INTO built_session (id, source, blocks, created_at, updated_at, deleted_at)
      VALUES ('01BUILT0000000000000000002', NULL,
        '[{"id":"b1","targetKind":"journal","targetValue":"j1","minutes":5}]',
        '2026-08-07T11:00:00Z', '2026-08-07T11:00:00Z', NULL);
      INSERT INTO journal_item (id, name, notes, linked_item_id, created_at, updated_at, deleted_at)
      VALUES ('j1', 'Rubato feel', NULL, NULL,
        '2026-08-07T10:00:00Z', '2026-08-07T10:00:00Z', NULL);
      """)
    let data = try store.loadBuiltSessionData()
    #expect(data.builtSessions.map(\.id) == ["01BUILT0000000000000000002"])
    #expect(data.journalItems.count == 1, "one bad row must not cost the whole library")
  }

  @Test func theQuarantinedRowIsStillOnDiskForANewerBinary() throws {
    let store = try storeSeeded(
      """
      INSERT INTO built_session (id, source, blocks, created_at, updated_at, deleted_at)
      VALUES ('01BUILT0000000000000000001', NULL,
        '[{"id":"b1","targetKind":"from_the_future","targetValue":"x","minutes":5}]',
        '2026-08-07T10:00:00Z', '2026-08-07T10:00:00Z', NULL);
      """)
    _ = try store.loadBuiltSessionData()
    #expect(
      try store.rawBuiltSessionBlocks(id: "01BUILT0000000000000000001")?.contains(
        "from_the_future") == true,
      "reading must never rewrite: the block is intact for the binary that understands it")
  }

  @Test func anUnknownReflectionKindQuarantinesItsRowRatherThanDefaulting() throws {
    let store = try storeSeeded(
      """
      INSERT INTO reflection (id, kind, session_ref, transcript, audio_path, duration_s,
        at, updated_at, deleted_at)
      VALUES ('r1', 'from_the_future', NULL, 'The bridge still rushes', NULL, NULL,
        '2026-08-07T10:30:00Z', '2026-08-07T10:30:00Z', NULL);
      """)
    #expect(try store.loadBuiltSessionData().reflections.isEmpty)
  }

  @Test func anUnknownFeelQuarantinesItsRowRatherThanDefaulting() throws {
    let store = try storeSeeded(
      """
      INSERT INTO feel_entry (id, block_id, feel, at, updated_at, deleted_at)
      VALUES ('f1', 'b1', 'from_the_future',
        '2026-08-07T10:15:00Z', '2026-08-07T10:15:00Z', NULL);
      """)
    #expect(try store.loadBuiltSessionData().feelEntries.isEmpty)
  }

  @Test func anUnknownServesKindQuarantinesItsDrillRatherThanDroppingTheTag() throws {
    let store = try storeSeeded(
      """
      INSERT INTO user_drill (id, name, criterion, tempo_bpm, keys, passes_to_open,
        serves_kind, serves_value, created_at, updated_at, deleted_at)
      VALUES ('d1', 'Descending run', 'Three clean passes at 72', 72, '[]', 3,
        'from_the_future', 'x', '2026-08-07T10:00:00Z', '2026-08-07T10:00:00Z', NULL);
      """)
    #expect(try store.loadBuiltSessionData().userDrills.isEmpty)
  }

  @Test func feelEntryRoundTripsForEveryFeel() throws {
    let store = try LibraryStore.inMemory()
    let feels: [(String, Feel)] = [("f1", .foughtIt), ("f2", .gettingThere), ("f3", .itSang)]
    for (id, feel) in feels {
      try store.saveFeelEntry(
        FeelEntry(
          id: id, blockId: "b1", feel: feel, at: "2026-08-07T10:15:00Z",
          updatedAt: "2026-08-07T10:15:00Z", deletedAt: nil))
    }
    let data = try store.loadBuiltSessionData()
    #expect(data.feelEntries.map(\.feel) == [.foughtIt, .gettingThere, .itSang])
  }
}
