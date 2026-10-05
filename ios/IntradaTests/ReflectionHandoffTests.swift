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
            title: "Clair de lune", kind: .piece, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
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

  // ── The finish answers (#2307, #2308, #2303) ──

  @Test func theKeptPointTheFeltWordAndWhatGotInTheWayLandOnTheEntry() throws {
    let bridge = RowsBridge()
    let (entryId, plays) = try pieceMidSession(bridge)
    let note = "Left hand rushed in bar 12, got it at 84"
    _ = try bridge.update(
      .session(
        .updateReflectionDraft(
          answers: ReflectionHandoff.draft(
            ReflectionResult(marks: [:], note: note, tempos: []), plays: plays))))
    let offers = try #require(try bridge.rendered().activeSession?.record.finish?.noteOffers)
    let bar = try #require(offers.first { $0.label.contains("12") })
    let result = ReflectionResult(
      marks: [:], note: note, tempos: [], felt: .strained, obstacles: [.rhythm, .tension],
      notePoints: [bar.span])

    #expect(
      ReflectionHandoff.run(
        ReflectionHandoff.plan(
          entryId: entryId, now: "2026-09-21T10:05:00Z",
          nextItemStartedAt: "2026-09-21T10:05:30Z", reading: .silent, plays: plays,
          result: result),
        send: accepting(bridge)))

    let record = try #require(try bridge.rendered().summary?.entries.first?.record)
    #expect(record.felt == .strained)
    #expect(record.gotInTheWay == [.rhythm, .tension])
    #expect(record.notePoints.map(\.span) == [bar.span])
  }

  @Test func anAimAnsweredOnTheSheetIsKept() throws {
    let bridge = RowsBridge()
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
    let entryId = try #require(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.setEntryIntention(entryId: entryId, intention: "From memory")))
    _ = try bridge.update(.session(.startSession(now: "2026-09-21T10:00:00Z")))
    _ = try bridge.update(
      .session(.prepareReflection(now: "2026-09-21T10:05:00Z", reading: .silent)))
    #expect(try bridge.rendered().activeSession?.record.finish?.asksIntention == true)
    let plays = try ReflectionPlay.rows(
      #require(try bridge.rendered().activeSession?.entries.first).plays)

    let handoff = ReflectionHandoff.plan(
      entryId: entryId, now: "2026-09-21T10:05:00Z", nextItemStartedAt: "2026-09-21T10:05:30Z",
      reading: .silent, plays: plays,
      result: ReflectionResult(marks: [:], note: "", tempos: [], intentionMet: .partly))
    #expect(ReflectionHandoff.run(handoff, send: accepting(bridge)))

    #expect(try bridge.rendered().summary?.entries.first?.record.intentionMet == .partly)
  }

  @Test func theFinishAnswersRideTheSheetsDraft() throws {
    let bridge = RowsBridge()
    let (_, plays) = try pieceMidSession(bridge)
    let result = ReflectionResult(
      marks: [:], note: "", tempos: [], felt: .hardWork, obstacles: [.memory],
      intentionMet: .notYet)

    _ = try bridge.update(
      .session(.updateReflectionDraft(answers: ReflectionHandoff.draft(result, plays: plays))))

    let saved = try #require(try bridge.rendered().activeSession?.reflection?.answers)
    let seed = ReflectionHandoff.seed(saved)
    #expect(seed.felt == .hardWork)
    #expect(seed.obstacles == [.memory])
    #expect(seed.intentionMet == .notYet)
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

  // ── What a row says was played (#2249) ──

  @Test func aRowChangedToAnotherKeyRecordsThatKeyOnThePlay() throws {
    let bridge = RowsBridge()
    let (entryId, plays) = try pieceMidSession(bridge)
    let play = try #require(plays.first)
    let g = Key(letter: .g, accidental: .natural, mode: .major)
    var changed = play.recordedWay
    changed.key = g
    let result = ReflectionResult(
      marks: [:], note: "", tempos: [],
      ways: PlayWayChoices.drafting(changed, recorded: play.recordedWay, into: []))
    _ = try bridge.update(
      .session(.updateReflectionDraft(answers: ReflectionHandoff.draft(result, plays: plays))))
    let saved = try #require(try bridge.rendered().activeSession?.reflection?.answers)
    let reopened = ReflectionHandoff.seed(saved)

    #expect(
      ReflectionHandoff.run(
        ReflectionHandoff.plan(
          entryId: entryId, now: "2026-09-21T10:05:00Z",
          nextItemStartedAt: "2026-09-21T10:05:30Z", reading: .silent, plays: plays,
          result: reopened),
        send: accepting(bridge)))

    let recorded = try #require(try bridge.rendered().summary?.entries.first?.plays.first)
    #expect(recorded.key == g)
  }

  @Test func aSkippedSheetLeavesThePlayAsItWas() throws {
    let bridge = RowsBridge()
    let (_, plays) = try pieceMidSession(bridge)
    let play = try #require(plays.first)
    var changed = play.recordedWay
    changed.key = Key(letter: .g, accidental: .natural, mode: .major)
    _ = try bridge.update(
      .session(
        .updateReflectionDraft(
          answers: ReflectionHandoff.draft(
            ReflectionResult(marks: [:], note: "", tempos: [], ways: [changed]), plays: plays))))

    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-21T10:05:00Z", nextItemStartedAt: "2026-09-21T10:05:30Z",
          reading: .silent)))

    let recorded = try #require(try bridge.rendered().summary?.entries.first?.plays.first)
    #expect(recorded.key == nil)
  }

  @Test func aWayChangedBackToWhatWasRecordedLeavesTheDraft() {
    let recorded = DraftWay(
      playId: "p1", sectionId: "s1", key: nil, variationIds: ["v1", "v2"])
    var changed = recorded
    changed.key = Key(letter: .g, accidental: .natural, mode: .major)
    let drafted = PlayWayChoices.drafting(changed, recorded: recorded, into: [])
    var back = recorded
    back.variationIds = ["v2", "v1"]

    #expect(drafted == [changed])
    #expect(PlayWayChoices.drafting(back, recorded: recorded, into: drafted).isEmpty)
  }

  // ── The stepper's beat value (#2304) ──

  @Test func anUntouchedClickOnAQuaverBarCountsAndSavesInQuavers() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Gigue", kind: .piece, composer: nil, key: nil,
            tempo: TempoInput(marking: nil, bpm: "240"), notes: nil, tags: [], photoId: nil,
            variationLabels: []))))
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
