import SharedTypes
import Testing

@testable import Intrada

/// The item-complete hand-off through `LiveBridge` (#1945): the score and the
/// tempo land only on a completed entry, and a refused note must stop the move.
struct ReflectionHandoffTests {

  private func pieceMidSession(_ bridge: LiveBridge) throws -> (
    entryId: String, plays: [ReflectionPlay]
  ) {
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Clair de lune", kind: .piece, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let itemId = try #require(try bridge.view().items.first?.id)
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-21T10:00:00Z")))
    _ = try bridge.update(
      .session(.prepareReflection(now: "2026-09-21T10:05:00Z", reading: .silent)))
    let entry = try #require(try bridge.view().activeSession?.entries.first)
    return (entry.id, ReflectionPlay.rows(entry.plays))
  }

  private func plan(
    entryId: String, plays: [ReflectionPlay], note: String, marked: Bool
  ) -> ReflectionHandoff.Plan {
    let result = ReflectionResult(
      marks: marked ? Dictionary(uniqueKeysWithValues: plays.map { ($0.id, UInt8(4)) }) : [:],
      note: note,
      tempos: plays.map {
        ReflectionRowTempo(playId: $0.id, tempo: 96, userSet: true, click: nil)
      })
    return ReflectionHandoff.plan(
      entryId: entryId, now: "2026-09-21T10:05:00Z",
      nextItemStartedAt: "2026-09-21T10:05:30Z", reading: .silent, plays: plays, result: result)
  }

  private func accepting(_ bridge: LiveBridge) -> (Event) -> Bool {
    { event in
      let before = try? bridge.view().errorSeq
      _ = try? bridge.update(event)
      return (try? bridge.view().errorSeq) == before
    }
  }

  @Test func theNoteTheMarkAndTheTempoAllLandOnTheCompletedEntry() throws {
    let bridge = LiveBridge()
    let (entryId, plays) = try pieceMidSession(bridge)
    let handoff = plan(entryId: entryId, plays: plays, note: "Pedal clearer", marked: true)

    #expect(ReflectionHandoff.run(handoff, send: accepting(bridge)))

    let entry = try #require(try bridge.view().summary?.entries.first)
    #expect(entry.notes == "Pedal clearer")
    #expect(entry.plays.last?.score == 4)
    #expect(entry.plays.last?.achievedTempo == 96)
  }

  @Test func aRefusedNoteKeepsTheSheetUpAndTheItemCurrent() throws {
    let bridge = LiveBridge()
    let (entryId, plays) = try pieceMidSession(bridge)
    let handoff = plan(
      entryId: entryId, plays: plays, note: String(repeating: "a", count: 5001), marked: true)

    #expect(!ReflectionHandoff.run(handoff, send: accepting(bridge)))
    #expect(try bridge.view().activeSession != nil, "the item has not moved on")
    #expect(try bridge.view().summary == nil)
  }

  @Test func anEmptyNoteAndNoMarksSendNoNoteAndNoScores() throws {
    let plays = [
      ReflectionPlay(
        id: "p1", variationLabel: nil, durationDisplay: "5:00", repCount: nil, repTarget: nil,
        isMarkable: true, tempoDisplay: nil, clickPattern: nil)
    ]
    let handoff = plan(entryId: "e1", plays: plays, note: "", marked: false)

    #expect(handoff.note == nil)
    #expect(handoff.after.count == 1, "the tempo row, and no score")
  }
}
