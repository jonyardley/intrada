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
    try store.savePlayThrough(record)
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
          durationS: 24, at: "2026-08-07T10:30:00Z",
          updatedAt: "2026-08-07T10:30:00Z", deletedAt: nil))
    }
    let data = try store.loadBuiltSessionData()
    #expect(data.reflections.count == 2)
    #expect(data.reflections.map(\.kind) == [.voiceNote, .sessionClose])
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
