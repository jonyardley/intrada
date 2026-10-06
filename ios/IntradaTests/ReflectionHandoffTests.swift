import SharedTypes
import Testing

@testable import Intrada

/// The item-complete sheet's answers through `LiveBridge` (#1945, #2230): what
/// it drafts and submits lands on the completed entry.
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

  private func submit(_ bridge: RowsBridge, _ answers: ReflectionAnswers) throws {
    _ = try bridge.update(
      .session(.submitReflection(nextItemStartedAt: "2026-09-21T10:05:30Z", answers: answers)))
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

  // ── The finish answers (#2307, #2308, #2303) ──

  @Test func theKeptPointTheFeltWordAndWhatGotInTheWayLandOnTheEntry() throws {
    let bridge = RowsBridge()
    _ = try pieceMidSession(bridge)
    let note = "Left hand rushed in bar 12, got it at 84"
    _ = try bridge.update(.session(.updateReflectionDraft(answers: .preview(note: note))))
    let offers = try #require(try bridge.rendered().activeSession?.record.finish?.noteOffers)
    let bar = try #require(offers.first { $0.label.contains("12") })

    try submit(
      bridge,
      .preview(
        note: note, felt: .strained, gotInTheWay: [.rhythm, .tension], notePoints: [bar.span]))

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

    try submit(bridge, .preview(intentionMet: .partly))

    #expect(try bridge.rendered().summary?.entries.first?.record.intentionMet == .partly)
  }

  @Test func theFinishAnswersRideTheSheetsDraft() throws {
    let bridge = RowsBridge()
    _ = try pieceMidSession(bridge)

    _ = try bridge.update(
      .session(
        .updateReflectionDraft(
          answers: .preview(felt: .hardWork, gotInTheWay: [.memory], intentionMet: .notYet))))

    let saved = try #require(try bridge.rendered().activeSession?.reflection?.answers)
    #expect(saved.felt == .hardWork)
    #expect(saved.gotInTheWay == [.memory])
    #expect(saved.intentionMet == .notYet)
  }

  // ── The open sheet's draft (#2137) ──

  @Test func theSheetReopensFromTheSavedDraftWithWhatWasWritten() throws {
    let bridge = RowsBridge()
    let (_, plays) = try pieceMidSession(bridge)
    let playId = try #require(plays.first?.id)

    _ = try bridge.update(
      .session(
        .updateReflectionDraft(
          answers: .preview(
            marks: [DraftMark(playId: playId, score: 6)], note: "Pedal clearer",
            tempos: [DraftTempo(playId: playId, tempo: 88, click: nil)]))))

    let reflection = try #require(try bridge.rendered().activeSession?.reflection)
    #expect(reflection.answers.marks == [DraftMark(playId: playId, score: 6)])
    #expect(reflection.answers.note == "Pedal clearer")
    let row = try #require(reflection.tempos.first)
    #expect(row.tempo == 88)
    #expect(row.setByHand, "the reopened sheet still counts it as set by hand")
  }

  // ── What a row says was played (#2249) ──

  @Test func aRowChangedToAnotherKeyRecordsThatKeyOnThePlay() throws {
    let bridge = RowsBridge()
    let (_, plays) = try pieceMidSession(bridge)
    let play = try #require(plays.first)
    let g = Key(letter: .g, accidental: .natural, mode: .major)
    let changed = DraftWay(playId: play.id, sectionId: nil, key: g, variationIds: [])
    _ = try bridge.update(.session(.updateReflectionDraft(answers: .preview(ways: [changed]))))
    let row = try #require(try bridge.rendered().activeSession?.record.finish?.rows.first)
    #expect(row.label == "G major", "the row reads what the core drafted")
    let saved = try #require(try bridge.rendered().activeSession?.reflection?.answers)

    try submit(bridge, saved)

    let recorded = try #require(try bridge.rendered().summary?.entries.first?.plays.first)
    #expect(recorded.key == g)
  }

  @Test func aSkippedSheetLeavesThePlayAsItWas() throws {
    let bridge = RowsBridge()
    let (_, plays) = try pieceMidSession(bridge)
    let play = try #require(plays.first)
    let changed = DraftWay(
      playId: play.id, sectionId: nil, key: Key(letter: .g, accidental: .natural, mode: .major),
      variationIds: [])
    _ = try bridge.update(
      .session(
        .updateReflectionDraft(
          answers: .preview(ways: [changed]))))

    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-21T10:05:00Z", nextItemStartedAt: "2026-09-21T10:05:30Z",
          reading: .silent)))

    let recorded = try #require(try bridge.rendered().summary?.entries.first?.plays.first)
    #expect(recorded.key == nil)
  }

  @Test func aWayChangedBackToWhatWasRecordedLeavesTheDraft() throws {
    let bridge = RowsBridge()
    let (_, plays) = try pieceMidSession(bridge)
    let play = try #require(plays.first)
    let g = Key(letter: .g, accidental: .natural, mode: .major)
    for key in [g, nil] {
      let way = DraftWay(playId: play.id, sectionId: nil, key: key, variationIds: [])
      _ = try bridge.update(.session(.updateReflectionDraft(answers: .preview(ways: [way]))))
    }

    let saved = try #require(try bridge.rendered().activeSession?.reflection?.answers)
    #expect(saved.ways.isEmpty)
    let row = try #require(try bridge.rendered().activeSession?.record.finish?.rows.first)
    #expect(row.label == "No variation")
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

    let row = try #require(try bridge.rendered().activeSession?.reflection?.tempos.first)
    #expect(row.click.metre.unit == 8)
    #expect((row.band.min...row.band.max).contains(240), "quaver = 240 opens as itself")
    #expect(row.tempo == seed.clickSeedBpm)

    _ = try bridge.update(
      .session(
        .submitReflection(
          nextItemStartedAt: "2026-10-03T10:05:30Z",
          answers: .preview(
            tempos: [DraftTempo(playId: row.playId, tempo: 250, click: row.click)]))))

    let saved = try #require(try bridge.rendered().summary?.entries.first?.plays.last)
    #expect(saved.achievedTempo == 125, "quaver = 250 is crotchet = 125")
  }
}
