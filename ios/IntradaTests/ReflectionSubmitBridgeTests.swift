import SharedTypes
import Testing

@testable import Intrada

/// The item-complete sheet's one submit and its tempo rows through `LiveBridge` (#2230).
struct ReflectionSubmitBridgeTests {

  private func sheetOpen(_ bridge: RowsBridge, reading: TempoReading = .silent) throws -> String {
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Clair de lune", kind: .piece, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let itemId = try #require(try bridge.rendered().items.first?.id)
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-21T10:00:00Z")))
    _ = try bridge.update(
      .session(.prepareReflection(now: "2026-09-21T10:05:00Z", reading: reading)))
    return try #require(try bridge.rendered().activeSession?.entries.first?.plays.last?.id)
  }

  private func answers(playId: String, note: String) -> ReflectionAnswers {
    ReflectionAnswers(
      marks: [DraftMark(playId: playId, score: 4)], note: note,
      tempos: [DraftTempo(playId: playId, tempo: 96, click: nil)], felt: nil, gotInTheWay: [],
      notePoints: [], intentionMet: nil, ways: [])
  }

  @Test func theSubmitLandsTheNoteTheMarkAndTheTempo() throws {
    let bridge = RowsBridge()
    let playId = try sheetOpen(bridge)

    _ = try bridge.update(
      .session(
        .submitReflection(
          nextItemStartedAt: "2026-09-21T10:05:30Z",
          answers: answers(playId: playId, note: "Pedal clearer"))))

    let entry = try #require(try bridge.rendered().summary?.entries.first)
    #expect(entry.notes == "Pedal clearer")
    #expect(entry.plays.last?.score == 4)
    #expect(entry.plays.last?.achievedTempo == 96)
  }

  @Test func aRefusedNoteKeepsTheItemCurrent() throws {
    let bridge = RowsBridge()
    let playId = try sheetOpen(bridge)
    let before = try bridge.rendered().errorSeq

    _ = try bridge.update(
      .session(
        .submitReflection(
          nextItemStartedAt: "2026-09-21T10:05:30Z",
          answers: answers(playId: playId, note: String(repeating: "a", count: 5001)))))

    #expect(try bridge.rendered().errorSeq != before)
    #expect(try bridge.rendered().activeSession?.reflection != nil, "the sheet stays up")
  }

  @Test func theOpenSheetCarriesEachRowsSeededTempo() throws {
    let bridge = RowsBridge()
    let playId = try sheetOpen(
      bridge, reading: TempoReading(bpm: 72, clickSounding: false, click: nil))

    let row: ReflectionTempoView = try #require(
      try bridge.rendered().activeSession?.reflection?.tempos.first)
    #expect(row.playId == playId)
    #expect(row.tempo == 72)
    #expect(row.band.unit == row.click.metre.unit)
    #expect(!row.setByHand)
  }
}
