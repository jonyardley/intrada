import SharedTypes
import XCTest

@testable import Intrada

/// Real-bridge bincode round-trip for #1739 and #2246. A stub bridge cannot
/// catch the #846 silent-drop class, so this drives start, switch, score and
/// finish through `LiveBridge` and reads the result back off the ViewModel.
final class VariationPlayBridgeTests: XCTestCase {

  private func exerciseWithTwoVariations(_ bridge: RowsBridge) throws -> String {
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Major Scales", kind: .exercise, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let id = try XCTUnwrap(try bridge.rendered().items.first?.id)
    _ = try bridge.update(
      .item(.updateItemVariations(id: id, variationIds: [], newLabels: ["C", "D"])))
    return id
  }

  private func variationId(_ bridge: RowsBridge, label: String) throws -> String {
    let variations = try XCTUnwrap(try bridge.rendered().items.first?.variations)
    return try XCTUnwrap(variations.first(where: { $0.label == label })?.id)
  }

  func testSwitchingMidItemRecordsTwoPlaysOverTheRealBridge() throws {
    let bridge = RowsBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inD = try variationId(bridge, label: "D")

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(
      try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))

    let atStart = try XCTUnwrap(try bridge.rendered().activeSession)
    XCTAssertEqual(atStart.currentVariationIds, [], "a practice starts plain")
    XCTAssertNil(atStart.currentPlayLabel)

    _ = try bridge.update(
      .session(
        .switchPlay(
          entryId: entryId, sectionId: nil, key: nil, variationIds: [inD],
          now: "2026-09-01T10:05:00Z", reading: .silent)))

    let afterSwitch = try XCTUnwrap(try bridge.rendered().activeSession)
    let entry = try XCTUnwrap(afterSwitch.entries.first)
    XCTAssertEqual(entry.plays.count, 2, "the switch closed one play and opened another")
    XCTAssertEqual(entry.plays.first?.variationIds, [])
    XCTAssertEqual(entry.plays.first?.seconds, 300)
    XCTAssertEqual(entry.plays.last?.variationIds, [inD])
    XCTAssertEqual(afterSwitch.currentPlayLabel, "D")
  }

  /// A play names a key and several variations at once (#2246), and both
  /// survive the wire into the record.
  func testAPlayInAKeyWithTwoVariationsRoundTripsOverTheRealBridge() throws {
    let bridge = RowsBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inC = try variationId(bridge, label: "C")
    let inD = try variationId(bridge, label: "D")
    let eFlat = Key(letter: .e, accidental: .flat, mode: .major)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))
    _ = try bridge.update(
      .session(
        .switchPlay(
          entryId: entryId, sectionId: nil, key: eFlat, variationIds: [inD, inC],
          now: "2026-09-01T10:01:00Z", reading: .silent)))

    let active = try XCTUnwrap(try bridge.rendered().activeSession)
    XCTAssertEqual(active.currentKey, eFlat)
    XCTAssertEqual(active.currentVariationIds, [inD, inC], "in the order named")

    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-01T10:06:00Z", nextItemStartedAt: "2026-09-01T10:06:00Z", reading: .silent))
    )
    let view = try bridge.rendered()
    XCTAssertNil(view.error)
    let played: PlayView = try XCTUnwrap(view.summary?.entries.first?.plays.last)
    XCTAssertEqual(played.key, eFlat)
    XCTAssertEqual(played.variationIds, [inD, inC])
  }

  /// The plan the builder sets rides the wire as sections and variations.
  func testTheBuildersPlanRoundTripsOverTheRealBridge() throws {
    let bridge = RowsBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inC = try variationId(bridge, label: "C")

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(
      .session(.setEntryPlan(entryId: entryId, sectionIds: [], variationIds: [inC])))

    let entry = try XCTUnwrap(try bridge.rendered().buildingSetlist?.entries.first)
    XCTAssertEqual(entry.plannedVariationIds, [inC])
    XCTAssertEqual(entry.plannedLabel, "C")
  }

  /// Got it counts past the target, and undo takes the last tap back without
  /// calling it a miss (#2107).
  func testUndoAndThePushPastTheTargetOverTheRealBridge() throws {
    let bridge = RowsBridge()
    let itemId = try exerciseWithTwoVariations(bridge)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.setRepTarget(entryId: entryId, target: 3)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))
    let sounding = TempoReading(bpm: 96, clickSounding: true, click: nil)
    for second in 10...13 {
      _ = try bridge.update(
        .session(.repGotIt(now: "2026-09-01T10:00:\(second)Z", reading: sounding)))
    }
    let past = try XCTUnwrap(try bridge.rendered().activeSession)
    XCTAssertEqual(past.currentRepCount, 4)
    XCTAssertEqual(past.currentRepsPastTarget, 1)
    XCTAssertTrue(past.currentCanUndo)
    XCTAssertEqual(past.currentRepHistory?.last?.tempo, 96)
    XCTAssertEqual(past.currentRepHistory?.last?.clickSounding, true)

    _ = try bridge.update(
      .session(.repUndo(now: "2026-09-01T10:00:20Z", reading: sounding)))
    XCTAssertNil(try bridge.rendered().error)
    let undone = try XCTUnwrap(try bridge.rendered().activeSession)
    XCTAssertEqual(undone.currentRepCount, 3)
    XCTAssertEqual(undone.currentRepHistory?.last?.action, .undo)
  }

  func testATempoChangeIsKeptOverTheRealBridge() throws {
    let bridge = RowsBridge()
    let itemId = try exerciseWithTwoVariations(bridge)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))
    let requests = try bridge.update(
      .session(
        .tempoChanged(
          now: "2026-09-01T10:01:00Z",
          reading: TempoReading(bpm: 84, clickSounding: true, click: nil))))

    XCTAssertNil(try bridge.rendered().error, "the change decodes on the wire (#846)")
    let saved = requests.compactMap { request -> ActiveSession? in
      guard case .app(.saveSessionInProgress(let session)) = request.effect else { return nil }
      return session
    }
    let play: Play? = saved.last?.entries.first?.plays.first
    XCTAssertEqual(
      play?.tempoChanges,
      [TempoChange(at: "2026-09-01T10:01:00Z", tempo: 84, clickSounding: true)],
      "the change is kept on the open play, in the practice in progress")
  }

  /// Moved from `VariationPickerUITests` (#1825): switching variation restarts the rep count.
  func testSwitchingVariationRestartsTheRepetitionCountOverTheRealBridge() throws {
    let bridge = RowsBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inD = try variationId(bridge, label: "D")

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))

    _ = try bridge.update(.session(.repGotIt(now: "2026-09-01T10:00:30Z", reading: .silent)))
    _ = try bridge.update(.session(.repGotIt(now: "2026-09-01T10:00:40Z", reading: .silent)))
    let beforeSwitch = try XCTUnwrap(try bridge.rendered().activeSession)
    XCTAssertEqual(beforeSwitch.currentRepCount, 2, "two repetitions banked on the plain play")

    _ = try bridge.update(
      .session(
        .switchPlay(
          entryId: entryId, sectionId: nil, key: nil, variationIds: [inD],
          now: "2026-09-01T10:05:00Z", reading: .silent)))

    let afterSwitch = try XCTUnwrap(try bridge.rendered().activeSession)
    XCTAssertEqual(afterSwitch.currentPlayLabel, "D", "the chip follows the switch")
    XCTAssertNil(
      afterSwitch.currentRepCount, "D's fresh play starts untouched rather than carrying the 2")
  }

  /// A tempo lands on the play it was played on (#1761): the switch stamps the
  /// plain play from the click sounding then and the hand-off stamps D, through real
  /// bincode with a quaver click on both events.
  func testEachPlayKeepsTheTempoItsClickSoundedAtOverTheRealBridge() throws {
    let bridge = RowsBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inD = try variationId(bridge, label: "D")
    let quavers = ClickState(
      metre: Metre(beats: 7, unit: 8, groups: [3, 2, 2]), sounding: 0b0101001)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))

    _ = try bridge.update(
      .session(
        .switchPlay(
          entryId: entryId, sectionId: nil, key: nil, variationIds: [inD],
          now: "2026-09-01T10:06:00Z",
          reading: TempoReading(bpm: 216, clickSounding: true, click: quavers))))
    let handOff = TempoReading(bpm: 200, clickSounding: true, click: quavers)
    _ = try bridge.update(
      .session(.prepareReflection(now: "2026-09-01T10:06:30Z", reading: handOff)))
    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-01T10:06:30Z", nextItemStartedAt: "2026-09-01T10:06:30Z",
          reading: handOff)))

    let view = try bridge.rendered()
    XCTAssertNil(view.error, "every reading must decode on the wire (#846)")
    let plays = try XCTUnwrap(view.summary?.entries.first?.plays)
    XCTAssertEqual(plays.map(\.variationIds), [[], [inD]])
    XCTAssertEqual(plays.map(\.achievedTempo), [108, 100], "stamped in crotchets")
    XCTAssertEqual(
      plays.map(\.tempoDisplay), [216, 200], "and read back in the click's quavers")
    XCTAssertEqual(plays.map(\.clickPattern), [quavers, quavers])
  }

  /// The picker's captions come from the core (#1784): the next read after a
  /// switch must show the new variation as playing and the old one as played
  /// this session, not stale "not yet played" text from a stub bridge.
  func testSwitchingVariationUpdatesCurrentVariationsOverTheRealBridge() throws {
    let bridge = RowsBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inC = try variationId(bridge, label: "C")
    let inD = try variationId(bridge, label: "D")

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))
    _ = try bridge.update(
      .session(
        .switchPlay(
          entryId: entryId, sectionId: nil, key: nil, variationIds: [inC],
          now: "2026-09-01T10:00:00Z", reading: .silent)))

    let atStart = try XCTUnwrap(try bridge.rendered().activeSession)
    let captionsAtStart = Dictionary(
      uniqueKeysWithValues: atStart.currentVariations.map { ($0.id, $0.caption) })
    XCTAssertEqual(captionsAtStart[inC], "Playing now")
    XCTAssertEqual(captionsAtStart[inD], "Not yet played")

    _ = try bridge.update(
      .session(
        .switchPlay(
          entryId: entryId, sectionId: nil, key: nil, variationIds: [inD],
          now: "2026-09-01T10:05:00Z", reading: .silent)))

    let afterSwitch = try XCTUnwrap(try bridge.rendered().activeSession)
    let captionsAfterSwitch = Dictionary(
      uniqueKeysWithValues: afterSwitch.currentVariations.map { ($0.id, $0.caption) })
    XCTAssertEqual(
      captionsAfterSwitch[inD], "Playing now",
      "the picker's next read shows the new current variation")
    XCTAssertEqual(
      captionsAfterSwitch[inC], "Played this session · 5m 0s",
      "the closed play still shows as played this session, not not-yet-played")
  }

  func testScoringEachPlayLandsOnItsOwnRowOverTheRealBridge() throws {
    let bridge = RowsBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inD = try variationId(bridge, label: "D")

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))
    _ = try bridge.update(
      .session(
        .switchPlay(
          entryId: entryId, sectionId: nil, key: nil, variationIds: [inD],
          now: "2026-09-01T10:05:00Z", reading: .silent)))
    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-01T10:10:00Z", nextItemStartedAt: "2026-09-01T10:10:00Z", reading: .silent))
    )

    let plays = try XCTUnwrap(try bridge.rendered().summary?.entries.first?.plays)
    XCTAssertEqual(plays.count, 2)

    _ = try bridge.update(
      .session(.updateEntryScore(entryId: entryId, playId: plays[0].id, score: 8)))
    _ = try bridge.update(
      .session(.updateEntryScore(entryId: entryId, playId: plays[1].id, score: 5)))

    let entry = try XCTUnwrap(try bridge.rendered().summary?.entries.first)
    XCTAssertEqual(entry.plays.map(\.score), [8, 5])
    XCTAssertEqual(entry.scoreSummary, 8, "the piece's own score reads the plain play only")
  }

  /// Each section's mark and the weakest one cross the wire inside the item and
  /// Up next (#2250).
  func testSectionMarksAndTheWeakestCrossTheRealBridge() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    for (title, kind) in [("Rondo", ItemKind.piece), ("Scales", .exercise)] {
      _ = try bridge.update(
        .item(
          .add(
            CreateItem(
              title: title, kind: kind, composer: nil, key: nil,
              tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    }
    let ids = Dictionary(
      uniqueKeysWithValues: try bridge.rendered().items.map { ($0.title, $0.id) })
    let pieceId = try XCTUnwrap(ids["Rondo"])
    try bridge.addSections(named: ["A", "B"], to: pieceId)
    let sectionIds = try XCTUnwrap(
      try bridge.rendered().items.first { $0.id == pieceId }?.sections.map(\.id))

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: pieceId)))
    let entryId = try XCTUnwrap(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))
    for (index, sectionId) in sectionIds.enumerated() {
      _ = try bridge.update(
        .session(
          .switchPlay(
            entryId: entryId, sectionId: sectionId, key: nil, variationIds: [],
            now: "2026-09-01T10:0\(index + 1):00Z", reading: .silent)))
    }
    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-01T10:10:00Z", nextItemStartedAt: "2026-09-01T10:10:00Z", reading: .silent))
    )
    let plays = try XCTUnwrap(try bridge.rendered().summary?.entries.first?.plays)
    for (sectionId, score) in zip(sectionIds, [8, 5] as [UInt8]) {
      let play = try XCTUnwrap(plays.first { $0.sectionId == sectionId })
      _ = try bridge.update(
        .session(.updateEntryScore(entryId: entryId, playId: play.id, score: score)))
    }
    let write = try XCTUnwrap(
      try bridge.update(.session(.saveSession(now: "2026-09-01T10:11:00Z"))).first {
        if case .persistence(.saveSession) = $0.effect { return true } else { return false }
      })
    _ = try bridge.resolve(write.id, persistenceOutput: .ack)
    // Linked only now, so the setlist held the piece alone.
    try bridge.link(exercise: try XCTUnwrap(ids["Scales"]), to: pieceId)

    let vm = try bridge.rendered()
    let sections = try XCTUnwrap(vm.items.first { $0.id == pieceId }?.sections)
    XCTAssertEqual(sections.map(\.latestScore), [8, 5])
    XCTAssertEqual(sections.map(\.caption), ["8 of 10", "5 of 10"])
    XCTAssertEqual(sections.map(\.isWeakest), [false, true])
    XCTAssertEqual(sections.map { $0.scoreHistory.count }, [1, 1])
    let pieceRow = try XCTUnwrap(
      vm.upNext?.blocks.first?.items.first { $0.itemId == pieceId }, "the piece leads Up next")
    XCTAssertEqual(pieceRow.weakestSection, "Weakest section · B")
  }

  /// `PrepareReflection` predicts a play's markability over the real bincode bridge (#1758).
  func testPrepareReflectionPredictsWhichPlaySurvivesOverTheRealBridge() throws {
    let bridge = RowsBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inD = try variationId(bridge, label: "D")

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))
    let opened = try XCTUnwrap(try bridge.rendered().activeSession?.entries.first?.plays.first?.id)

    // A stray tap two seconds before the item ends.
    _ = try bridge.update(
      .session(
        .switchPlay(
          entryId: entryId, sectionId: nil, key: nil, variationIds: [inD],
          now: "2026-09-01T10:04:58Z", reading: .silent)))
    let strayTap = try XCTUnwrap(try bridge.rendered().activeSession?.entries.first?.plays.last?.id)

    _ = try bridge.update(
      .session(.prepareReflection(now: "2026-09-01T10:05:00Z", reading: .silent)))
    let stamped = try XCTUnwrap(try bridge.rendered().activeSession?.entries.first)
    XCTAssertEqual(
      stamped.plays.last?.seconds, 2, "PrepareReflection stamped the real duration")
    XCTAssertEqual(stamped.plays.first(where: { $0.id == opened })?.isMarkable, true)
    XCTAssertEqual(stamped.plays.first(where: { $0.id == strayTap })?.isMarkable, false)

    // The same `now` PrepareReflection used, or the prediction goes stale.
    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-01T10:05:00Z", nextItemStartedAt: "2026-09-01T10:05:00Z", reading: .silent))
    )
    let survivors = try XCTUnwrap(try bridge.rendered().summary?.entries.first?.plays)
    XCTAssertEqual(survivors.map(\.id), [opened], "the prediction matched the drop")
  }

  /// A `playId` that belongs to another entry must be refused, or one row of
  /// the item-complete sheet could write another's mark.
  func testAForeignPlayIdIsRefusedOverTheRealBridge() throws {
    let bridge = RowsBridge()
    let itemId = try exerciseWithTwoVariations(bridge)

    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Clair de Lune", kind: .piece, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let pieceId = try XCTUnwrap(
      try bridge.rendered().items.first(where: { $0.itemType == .piece })?.id)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    _ = try bridge.update(.session(.addToSetlist(itemId: pieceId)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))
    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-01T10:05:00Z", nextItemStartedAt: "2026-09-01T10:05:00Z", reading: .silent))
    )
    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-01T10:10:00Z", nextItemStartedAt: "2026-09-01T10:10:00Z", reading: .silent))
    )

    let entries = try XCTUnwrap(try bridge.rendered().summary?.entries)
    let first = try XCTUnwrap(entries.first)
    let foreign = try XCTUnwrap(entries.last?.plays.first?.id)

    _ = try bridge.update(
      .session(.updateEntryScore(entryId: first.id, playId: foreign, score: 9)))

    let after = try XCTUnwrap(try bridge.rendered().summary?.entries.first)
    XCTAssertNil(after.plays.first?.score, "the foreign play id wrote nothing")
    XCTAssertNotNil(try bridge.rendered().error, "and the refusal is surfaced")
  }

  /// The notice channel over the real bridge (#1325, #846): a reading the core
  /// cannot keep reaches the shell as a notice, not an error.
  func testAnUnusableReadingArrivesAsANoticeOverTheRealBridge() throws {
    let bridge = RowsBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inD = try variationId(bridge, label: "D")
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))
    let before = try bridge.rendered()
    XCTAssertNil(before.notice)

    let minimsPastTheCeiling = TempoReading(
      bpm: 260, clickSounding: true,
      click: ClickState(metre: Metre(beats: 2, unit: 2, groups: nil), sounding: 0b11))
    _ = try bridge.update(
      .session(
        .switchPlay(
          entryId: entryId, sectionId: nil, key: nil, variationIds: [inD],
          now: "2026-09-01T10:05:00Z",
          reading: minimsPastTheCeiling)))

    let after = try bridge.rendered()
    XCTAssertEqual(
      after.notice, "That metronome setting doesn't give a crotchet tempo, so this play has none.")
    XCTAssertEqual(after.noticeSeq, before.noticeSeq + 1)
    XCTAssertEqual(after.errorSeq, before.errorSeq, "a notice is not a refusal")
    XCTAssertNil(after.error)

    _ = try bridge.update(.clearNotice)
    XCTAssertNil(try bridge.rendered().notice)
  }
}

extension TempoReading {
  /// A click that was not sounding: closes a play and stamps nothing (#1761).
  static var silent: TempoReading { TempoReading(bpm: 120, clickSounding: false, click: nil) }
}
