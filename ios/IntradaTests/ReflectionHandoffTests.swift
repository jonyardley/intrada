import SharedTypes
import Testing

@testable import Intrada

/// The item-complete hand-off through `LiveBridge` (#1945): the score and the
/// tempo land only on a completed entry, and a refused note must stop the move.
struct ReflectionHandoffTests {

  private func pieceMidSession(_ bridge: RowsBridge) throws -> (
    entryId: String, plays: [ReflectionPlay]
  ) {
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Clair de lune", kind: .piece, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let itemId = try #require(try bridge.rendered().items.first?.id)
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-21T10:00:00Z")))
    _ = try bridge.update(
      .session(.prepareReflection(now: "2026-09-21T10:05:00Z", reading: .silent)))
    let entry = try #require(try bridge.rendered().activeSession?.entries.first)
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

  private func accepting(_ bridge: RowsBridge) -> (Event) -> Bool {
    { event in
      let before = try? bridge.rendered().errorSeq
      _ = try? bridge.update(event)
      return (try? bridge.rendered().errorSeq) == before
    }
  }

  @Test func theNoteTheMarkAndTheTempoAllLandOnTheCompletedEntry() throws {
    let bridge = RowsBridge()
    let (entryId, plays) = try pieceMidSession(bridge)
    let handoff = plan(entryId: entryId, plays: plays, note: "Pedal clearer", marked: true)

    #expect(ReflectionHandoff.run(handoff, send: accepting(bridge)))

    let entry = try #require(try bridge.rendered().summary?.entries.first)
    #expect(entry.notes == "Pedal clearer")
    #expect(entry.plays.last?.score == 4)
    #expect(entry.plays.last?.achievedTempo == 96)
  }

  @Test func aRefusedNoteKeepsTheSheetUpAndTheItemCurrent() throws {
    let bridge = RowsBridge()
    let (entryId, plays) = try pieceMidSession(bridge)
    let handoff = plan(
      entryId: entryId, plays: plays, note: String(repeating: "a", count: 5001), marked: true)

    #expect(!ReflectionHandoff.run(handoff, send: accepting(bridge)))
    #expect(try bridge.rendered().activeSession != nil, "the item has not moved on")
    #expect(try bridge.rendered().summary == nil)
  }

  @Test(
    arguments: [
      ("Notes must not exceed 5000 characters", "Notes must not exceed 5000 characters"),
      (nil, "Couldn't save. Try again."),
    ] as [(String?, String)])
  @MainActor func theSheetShowsWhyTheSaveWasRefused(error: String?, shown: String) {
    #expect(ReflectionHandoff.refusalMessage(halted: false, error: error) == shown)
  }

  @Test(arguments: [nil, "Notes must not exceed 5000 characters"] as [String?])
  @MainActor func aStoppedAppSaysSoRatherThanAskingForARetry(error: String?) {
    #expect(ReflectionHandoff.refusalMessage(halted: true, error: error) == Store.haltedMessage)
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

  // ── The open sheet's draft (#2137) ──

  @Test func theSheetReopensFromTheSavedDraftWithWhatWasWritten() throws {
    let bridge = RowsBridge()
    let (_, plays) = try pieceMidSession(bridge)
    let playId = try #require(plays.first?.id)
    let result = ReflectionResult(
      marks: [playId: 6], note: "Pedal clearer",
      tempos: [ReflectionRowTempo(playId: playId, tempo: 88, userSet: true, click: nil)])

    _ = try bridge.update(
      .session(.updateReflectionDraft(answers: ReflectionHandoff.draft(result, plays: plays))))

    let saved = try #require(try bridge.rendered().activeSession?.reflection?.answers)
    let seed = ReflectionHandoff.seed(saved)
    #expect(seed.marks == [playId: 6])
    #expect(seed.note == "Pedal clearer")
    #expect(seed.tempos.map(\.tempo) == [88])
    #expect(
      seed.tempos.map(\.userSet) == [true], "the reopened sheet still counts it as set by hand")
  }

  @Test func aTempoNobodyMovedStaysOutOfTheDraft() {
    let plays = [
      ReflectionPlay(
        id: "p1", variationLabel: nil, durationDisplay: "5:00", repCount: nil, repTarget: nil,
        isMarkable: true, tempoDisplay: 92, clickPattern: nil)
    ]
    let result = ReflectionResult(
      marks: [:], note: "",
      tempos: [ReflectionRowTempo(playId: "p1", tempo: 92, userSet: false, click: nil)])

    #expect(ReflectionHandoff.draft(result, plays: plays).tempos.isEmpty)
  }

  // ── The stepper's beat value (#2304) ──

  @Test func anUntouchedClickOnAQuaverBarCountsAndSavesInQuavers() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Gigue", kind: .piece, composer: nil, key: nil, modality: nil,
            tempo: TempoInput(marking: nil, bpm: "240"), notes: nil, tags: [], photoId: nil,
            variantLabels: []))))
    let itemId = try #require(try bridge.rendered().items.first?.id)
    _ = try bridge.update(
      .item(.setMetre(id: itemId, metre: Metre(beats: 6, unit: 8, groups: [3, 3]))))
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    _ = try bridge.update(.session(.startSession(now: "2026-10-03T10:00:00Z")))
    let seed = try #require(try bridge.rendered().activeSession)
    let untouched = TempoReading(bpm: seed.clickSeedBpm, clickSounding: false, click: nil)
    _ = try bridge.update(
      .session(.prepareReflection(now: "2026-10-03T10:05:00Z", reading: untouched)))

    let active = try #require(try bridge.rendered().activeSession)
    let reading = try #require(active.reflection?.reading)
    let click = ReflectionHandoff.sheetClick(reading, active: active)
    let limits = try bridge.rendered().limits
    #expect(click.metre.unit == 8)
    #expect(limits.clickBand(unit: click.metre.unit).contains(240), "quaver = 240 opens as itself")

    let entry = try #require(active.entries.first)
    let plays = ReflectionPlay.rows(entry.plays)
    let result = ReflectionResult(
      marks: [:], note: "",
      tempos: plays.map {
        ReflectionRowTempo(playId: $0.id, tempo: 250, userSet: true, click: click)
      })
    let handoff = ReflectionHandoff.plan(
      entryId: entry.id, now: "2026-10-03T10:05:00Z",
      nextItemStartedAt: "2026-10-03T10:05:30Z", reading: reading, plays: plays, result: result)

    #expect(ReflectionHandoff.run(handoff, send: accepting(bridge)))
    let saved = try #require(try bridge.rendered().summary?.entries.first?.plays.last)
    #expect(saved.achievedTempo == 125, "quaver = 250 is crotchet = 125")
  }
}
