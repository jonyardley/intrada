import Foundation
import GRDB
import SharedTypes
import Testing

@testable import Intrada

/// Writes the database the shared Rust store's iPhone-file test reads (#2421):
/// one row of every kind, through this store's own codecs. Run by hand when the
/// schema gains a migration, then check the file in:
/// `TEST_RUNNER_INTRADA_STORE_FIXTURE=<absolute path> just ios-test`.
@MainActor
struct SharedStoreFixtureTests {
  nonisolated private static let path = ProcessInfo.processInfo.environment["INTRADA_STORE_FIXTURE"]

  @Test(.enabled(if: path != nil))
  func writeTheFixture() throws {
    let path = try #require(Self.path)
    try? FileManager.default.removeItem(atPath: path)
    let store = try LibraryStore(DatabaseQueue(path: path))

    try store.save([Self.piece, Self.exercise, Self.deleted])
    try store.delete(id: Self.deleted.id, deletedAt: "2026-09-05T08:00:00Z")
    try store.save(Self.variations)
    try store.saveSession(Self.session)

    #expect(try store.loadItems().count == 2)
  }

  static let piece = Item(
    id: "p1", title: "Nocturne in E flat", kind: .piece, composer: "Chopin",
    key: Key(letter: .e, accidental: .flat, mode: .major),
    tempo: Tempo(marking: "Andante", bpm: 66), notes: "left hand / voicing",
    tags: ["romantic", "recital"], createdAt: "2026-09-01T10:00:00Z",
    updatedAt: "2026-09-02T11:30:00.123456789Z", priority: true,
    chordChart: ChordChart(
      key: Key(letter: .g, accidental: .natural, mode: .minor),
      sections: [
        ChartSection(
          label: "A",
          bars: [
            Bar(chords: [
              ChartChord(
                symbol: ChordSymbol(
                  root: 0, quality: .min7, extensions: ["9"], bass: 7, raw: "Cm9/G"))
            ])
          ])
      ]),
    photoId: "01JPHOTO", metre: Metre(beats: 7, unit: 8, groups: [2, 2, 3]),
    sections: [
      ItemSection(
        id: "s1", name: "A", bars: BarRange(first: 1, last: 8), kind: .form, targetBpm: nil,
        position: 0, updatedAt: "2026-09-02T11:30:00Z", deletedAt: nil),
      ItemSection(
        id: "s2", name: "", bars: BarRange(first: 12, last: 14), kind: .troubleSpot,
        targetBpm: 52, position: 1, updatedAt: "2026-09-02T11:30:00Z",
        deletedAt: "2026-09-03T09:00:00Z"),
    ],
    variationIds: ["00000000000000000000000001"],
    keys: [
      Key(letter: .c, accidental: .sharp, mode: .minor),
      Key(letter: .b, accidental: .flat, mode: nil),
    ],
    exerciseLinks: [
      ExerciseLink(
        id: "l1", exerciseId: "e1", sectionId: "s2", position: 0,
        updatedAt: "2026-09-02T11:30:00Z", deletedAt: nil)
    ])

  static let exercise = Item(
    id: "e1", title: "Scales in thirds", kind: .exercise, composer: nil, key: nil, tempo: nil,
    notes: nil, tags: [], createdAt: "2026-08-01T09:00:00Z", updatedAt: "2026-08-01T09:00:00Z",
    priority: false, chordChart: nil, photoId: nil, metre: nil, sections: [], variationIds: [],
    keys: [], exerciseLinks: [])

  static let deleted = Item(
    id: "gone", title: "Deleted", kind: .piece, composer: nil, key: nil, tempo: nil, notes: nil,
    tags: [], createdAt: "2026-07-01T09:00:00Z", updatedAt: "2026-07-01T09:00:00Z",
    priority: false, chordChart: nil, photoId: nil, metre: nil, sections: [], variationIds: [],
    keys: [], exerciseLinks: [])

  static let variations = [
    Variation(
      id: "00000000000000000000000001", label: "Hands separately",
      updatedAt: "2026-09-01T10:00:00Z", deletedAt: nil),
    Variation(
      id: "00000000000000000000000002", label: "Dotted rhythms",
      updatedAt: "2026-09-01T10:00:00Z", deletedAt: "2026-09-04T10:00:00Z"),
  ]

  static let session = PracticeSession(
    id: "sess-1",
    entries: [
      SetlistEntry(
        id: "en1", itemId: "p1", itemTitle: "Nocturne in E flat", itemType: .piece,
        position: 0, durationSecs: 600, status: .completed,
        notes: "steadier", intention: nil, plannedDurationSecs: 600,
        groupId: nil, plannedVariationIds: ["00000000000000000000000001"], plannedRepTarget: nil,
        plays: [
          Play(
            id: "en1-p1", sectionId: "s1", key: Key(letter: .e, accidental: .flat, mode: .major),
            variationIds: ["00000000000000000000000001"],
            startedAt: "2026-09-02T10:00:00Z", seconds: 600,
            repTarget: nil, repCount: nil, repHistory: nil,
            tempoChanges: [], achievedTempo: 60, clickPattern: nil, score: 8, away: [])
        ], segments: [], focus: nil, intentionMet: nil, felt: nil, gotInTheWay: [],
        notePoints: [], plannedKey: nil)
    ],
    sessionNotes: "good day",
    startedAt: "2026-09-02T10:00:00Z", completedAt: "2026-09-02T10:10:00Z",
    totalDurationSecs: 600, completionStatus: .completed, sessionScore: 7, captureVersion: 1)
}
