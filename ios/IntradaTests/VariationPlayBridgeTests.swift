import SharedTypes
import XCTest

@testable import Intrada

/// Real-bridge bincode round-trip for #1739. A stub bridge cannot catch the
/// #846 silent-drop class, so this drives start, switch, score and finish
/// through `LiveBridge` and reads the result back off the ViewModel.
final class VariationPlayBridgeTests: XCTestCase {

  private func exerciseWithTwoVariations(_ bridge: LiveBridge) throws -> String {
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Major Scales", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)
    _ = try bridge.update(.item(.setVariants(id: id, labels: ["C", "D"])))
    return id
  }

  private func variationId(_ bridge: LiveBridge, label: String) throws -> String {
    let variants = try XCTUnwrap(try bridge.view().items.first?.variants)
    return try XCTUnwrap(variants.first(where: { $0.label == label })?.id)
  }

  func testSwitchingMidItemRecordsTwoPlaysOverTheRealBridge() throws {
    let bridge = LiveBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inC = try variationId(bridge, label: "C")
    let inD = try variationId(bridge, label: "D")

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(
      try bridge.view().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: inC)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))

    let atStart = try XCTUnwrap(try bridge.view().activeSession)
    XCTAssertEqual(atStart.currentVariationId, inC, "the plan seeds the first play")
    XCTAssertEqual(atStart.currentVariationLabel, "C")

    _ = try bridge.update(
      .session(
        .switchVariation(
          entryId: entryId, variationId: inD, now: "2026-09-01T10:05:00Z", reading: .silent)))

    let afterSwitch = try XCTUnwrap(try bridge.view().activeSession)
    let entry = try XCTUnwrap(afterSwitch.entries.first)
    XCTAssertEqual(entry.plays.count, 2, "the switch closed one play and opened another")
    XCTAssertEqual(entry.plays.first?.variationId, inC)
    XCTAssertEqual(entry.plays.first?.seconds, 300)
    XCTAssertEqual(entry.plays.last?.variationId, inD)
    XCTAssertEqual(afterSwitch.currentVariationLabel, "D")
  }

  /// Moved from `VariationPickerUITests` (#1825): switching variation restarts the rep count.
  func testSwitchingVariationRestartsTheRepetitionCountOverTheRealBridge() throws {
    let bridge = LiveBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inC = try variationId(bridge, label: "C")
    let inD = try variationId(bridge, label: "D")

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.view().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: inC)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))

    _ = try bridge.update(.session(.repGotIt(now: "2026-09-01T10:00:30Z")))
    _ = try bridge.update(.session(.repGotIt(now: "2026-09-01T10:00:40Z")))
    let beforeSwitch = try XCTUnwrap(try bridge.view().activeSession)
    XCTAssertEqual(beforeSwitch.currentRepCount, 2, "two repetitions banked against C")

    _ = try bridge.update(
      .session(
        .switchVariation(
          entryId: entryId, variationId: inD, now: "2026-09-01T10:05:00Z", reading: .silent)))

    let afterSwitch = try XCTUnwrap(try bridge.view().activeSession)
    XCTAssertEqual(afterSwitch.currentVariationLabel, "D", "the chip follows the switch")
    XCTAssertNil(
      afterSwitch.currentRepCount, "D's fresh play starts untouched rather than carrying C's 2")
  }

  /// A tempo lands on the play it was played on (#1761): the switch stamps C
  /// from the click sounding then and the hand-off stamps D, through real
  /// bincode with a quaver click on both events.
  func testEachPlayKeepsTheTempoItsClickSoundedAtOverTheRealBridge() throws {
    let bridge = LiveBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inC = try variationId(bridge, label: "C")
    let inD = try variationId(bridge, label: "D")
    let quavers = ClickState(
      metre: Metre(beats: 7, unit: 8, groups: [3, 2, 2]), sounding: 0b0101001)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.view().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: inC)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))

    _ = try bridge.update(
      .session(
        .switchVariation(
          entryId: entryId, variationId: inD, now: "2026-09-01T10:06:00Z",
          reading: TempoReading(bpm: 216, clickSounding: true, click: quavers))))
    let handOff = TempoReading(bpm: 200, clickSounding: true, click: quavers)
    _ = try bridge.update(
      .session(.prepareReflection(now: "2026-09-01T10:06:30Z", reading: handOff)))
    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-01T10:06:30Z", nextItemStartedAt: "2026-09-01T10:06:30Z",
          reading: handOff)))

    let view = try bridge.view()
    XCTAssertNil(view.error, "every reading must decode on the wire (#846)")
    let plays = try XCTUnwrap(view.summary?.entries.first?.plays)
    XCTAssertEqual(plays.map(\.variationId), [inC, inD])
    XCTAssertEqual(plays.map(\.achievedTempo), [108, 100], "stamped in crotchets")
    XCTAssertEqual(
      plays.map(\.tempoDisplay), [216, 200], "and read back in the click's quavers")
    XCTAssertEqual(plays.map(\.clickPattern), [quavers, quavers])
  }

  /// The picker's captions come from the core (#1784): the next read after a
  /// switch must show the new variation as playing and the old one as played
  /// this session, not stale "not yet played" text from a stub bridge.
  func testSwitchingVariationUpdatesCurrentVariationsOverTheRealBridge() throws {
    let bridge = LiveBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inC = try variationId(bridge, label: "C")
    let inD = try variationId(bridge, label: "D")

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.view().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: inC)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))

    let atStart = try XCTUnwrap(try bridge.view().activeSession)
    let captionsAtStart = Dictionary(
      uniqueKeysWithValues: atStart.currentVariations.map { ($0.id, $0.caption) })
    XCTAssertEqual(captionsAtStart[inC], "Playing now")
    XCTAssertEqual(captionsAtStart[inD], "Not yet played")

    _ = try bridge.update(
      .session(
        .switchVariation(
          entryId: entryId, variationId: inD, now: "2026-09-01T10:05:00Z", reading: .silent)))

    let afterSwitch = try XCTUnwrap(try bridge.view().activeSession)
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
    let bridge = LiveBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inD = try variationId(bridge, label: "D")

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.view().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))
    _ = try bridge.update(
      .session(
        .switchVariation(
          entryId: entryId, variationId: inD, now: "2026-09-01T10:05:00Z", reading: .silent)))
    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-01T10:10:00Z", nextItemStartedAt: "2026-09-01T10:10:00Z", reading: .silent))
    )

    let plays = try XCTUnwrap(try bridge.view().summary?.entries.first?.plays)
    XCTAssertEqual(plays.count, 2)

    _ = try bridge.update(
      .session(.updateEntryScore(entryId: entryId, playId: plays[0].id, score: 8)))
    _ = try bridge.update(
      .session(.updateEntryScore(entryId: entryId, playId: plays[1].id, score: 5)))

    let entry = try XCTUnwrap(try bridge.view().summary?.entries.first)
    XCTAssertEqual(entry.plays.map(\.score), [8, 5])
    XCTAssertEqual(entry.scoreSummary, 7, "13 over 2 rounds to 7")
  }

  /// `PrepareReflection` predicts a play's markability over the real bincode bridge (#1758).
  func testPrepareReflectionPredictsWhichPlaySurvivesOverTheRealBridge() throws {
    let bridge = LiveBridge()
    let itemId = try exerciseWithTwoVariations(bridge)
    let inD = try variationId(bridge, label: "D")

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.view().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.startSession(now: "2026-09-01T10:00:00Z")))
    let opened = try XCTUnwrap(try bridge.view().activeSession?.entries.first?.plays.first?.id)

    // A stray tap two seconds before the item ends.
    _ = try bridge.update(
      .session(
        .switchVariation(
          entryId: entryId, variationId: inD, now: "2026-09-01T10:04:58Z", reading: .silent)))
    let strayTap = try XCTUnwrap(try bridge.view().activeSession?.entries.first?.plays.last?.id)

    _ = try bridge.update(
      .session(.prepareReflection(now: "2026-09-01T10:05:00Z", reading: .silent)))
    let stamped = try XCTUnwrap(try bridge.view().activeSession?.entries.first)
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
    let survivors = try XCTUnwrap(try bridge.view().summary?.entries.first?.plays)
    XCTAssertEqual(survivors.map(\.id), [opened], "the prediction matched the drop")
  }

  /// A `playId` that belongs to another entry must be refused, or one row of
  /// the item-complete sheet could write another's mark.
  func testAForeignPlayIdIsRefusedOverTheRealBridge() throws {
    let bridge = LiveBridge()
    let itemId = try exerciseWithTwoVariations(bridge)

    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Clair de Lune", kind: .piece, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let pieceId = try XCTUnwrap(
      try bridge.view().items.first(where: { $0.itemType == .piece })?.id)

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

    let entries = try XCTUnwrap(try bridge.view().summary?.entries)
    let first = try XCTUnwrap(entries.first)
    let foreign = try XCTUnwrap(entries.last?.plays.first?.id)

    _ = try bridge.update(
      .session(.updateEntryScore(entryId: first.id, playId: foreign, score: 9)))

    let after = try XCTUnwrap(try bridge.view().summary?.entries.first)
    XCTAssertNil(after.plays.first?.score, "the foreign play id wrote nothing")
    XCTAssertNotNil(try bridge.view().error, "and the refusal is surfaced")
  }
}

extension TempoReading {
  /// A click that was not sounding: closes a play and stamps nothing (#1761).
  static var silent: TempoReading { TempoReading(bpm: 120, clickSounding: false, click: nil) }
}
