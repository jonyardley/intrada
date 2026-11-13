import SharedTypes
import XCTest

@testable import Intrada

/// The v0.17 record crosses the real bincode bridge both ways (#846, #2249):
/// every new event from Swift, every new view and blob field back.
@MainActor
final class PracticeRecordBridgeTests: XCTestCase {
  private let start = "2026-10-04T09:00:00Z"

  /// Whole seconds, as chrono writes them, so a time comes back as sent.
  private func at(_ seconds: Int) -> String {
    let base = SessionClock.parseRFC3339(start) ?? Date()
    return ISO8601DateFormatter().string(from: base.addingTimeInterval(TimeInterval(seconds)))
  }

  private func sounding(_ bpm: UInt16) -> TempoReading {
    TempoReading(
      bpm: bpm, clickSounding: true,
      click: ClickState(metre: Metre(beats: 4, unit: 4, groups: nil), sounding: 0b1111))
  }

  /// A piece with a Dotted variation, built over twenty minutes, and started.
  private func practising() throws -> (RowsBridge, String, String) {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Nocturne", kind: .piece, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let itemId = try XCTUnwrap(try bridge.rendered().items.first?.id)
    _ = try bridge.update(
      .item(.updateItemVariations(id: itemId, variationIds: [], newLabels: ["Dotted"])))
    let dotted = try XCTUnwrap(try bridge.rendered().items.first?.variations.first?.id)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let building = try XCTUnwrap(try bridge.rendered().buildingSetlist)
    let lastTimes: [LastTimeView] = building.lastTimes
    XCTAssertEqual(lastTimes, [], "nothing played yet, so nothing to offer")
    let entryId = try XCTUnwrap(building.entries.first?.id)
    _ = try bridge.update(.session(.setEntryDuration(entryId: entryId, durationSecs: 1200)))
    XCTAssertNil(try bridge.rendered().error)

    _ = try bridge.update(.session(.startSession(now: start)))
    return (bridge, entryId, dotted)
  }

  private func switchToDotted(
    _ bridge: RowsBridge, _ entryId: String, _ dotted: String, at seconds: Int
  ) throws {
    _ = try bridge.update(
      .session(
        .switchPlay(
          entryId: entryId, key: nil, variationIds: [dotted], now: at(seconds),
          reading: sounding(84))))
  }

  /// Played dotted and saved, the piece offers Dotted next time, and the
  /// chip's tap plays it (#2249).
  func testRealBridgeLastTimesTapPlaysTheWayLastPlayed() throws {
    let (bridge, entryId, dotted) = try practising()
    try switchToDotted(bridge, entryId, dotted, at: 600)
    _ = try bridge.update(.session(.prepareReflection(now: at(900), reading: sounding(84))))
    _ = try bridge.update(
      .session(.nextItem(now: at(900), nextItemStartedAt: at(900), reading: sounding(84))))
    let saving = try bridge.update(.session(.saveSession(now: at(910))))
    let write = try XCTUnwrap(
      saving.first {
        if case .persistence(.saveSession) = $0.effect { return true } else { return false }
      })
    _ = try bridge.resolve(write.id, persistenceOutput: .ack)
    let itemId = try XCTUnwrap(try bridge.rendered().items.first?.id)
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    _ = try bridge.update(.session(.startSession(now: at(2000))))
    let offer: LastTimeView = try XCTUnwrap(try bridge.rendered().activeSession?.record.lastTime)
    XCTAssertEqual(offer.variationIds, [dotted])

    _ = try bridge.update(
      .session(FocusPlayerScreen.lastTimeEvent(offer, now: at(2060), reading: sounding(84))))

    let active = try XCTUnwrap(try bridge.rendered().activeSession)
    XCTAssertNil(try bridge.rendered().error)
    XCTAssertEqual(active.currentVariationIds, [dotted])
    XCTAssertNil(active.record.lastTime, "the chip goes once it is played")
  }

  func testRealBridgeTimeAwayRunsThroughThePracticeScreen() throws {
    let (bridge, entryId, dotted) = try practising()
    _ = try bridge.update(.session(.wentAway(at: at(60))))
    _ = try bridge.update(.session(.cameBack(at: at(420))))
    let live: ActiveRecordView = try XCTUnwrap(try bridge.rendered().activeSession?.record)
    XCTAssertNil(live.finish)
    let offer: AwayOfferView = try XCTUnwrap(live.awayOffer)
    XCTAssertEqual(offer.minutes, 6)
    XCTAssertEqual(offer.label, "Away 6 minutes. Leave it out?")
    _ = try bridge.update(.session(.leaveAwayOut))
    XCTAssertNil(try bridge.rendered().activeSession?.record.awayOffer)

    try switchToDotted(bridge, entryId, dotted, at: 1000)
    let active = try XCTUnwrap(try bridge.rendered().activeSession)
    let first = try XCTUnwrap(active.entries.first?.plays.first)
    XCTAssertEqual(first.seconds, 1000 - 360, "six minutes left out")
  }

  /// The finish sheet's rows and a way changed on one cross the bridge both
  /// ways: the row names the play, and the drafted way lands on it (#2249).
  func testRealBridgeAWayChangedOnTheFinishSheetLandsOnThePlay() throws {
    let (bridge, entryId, dotted) = try practising()
    _ = try bridge.update(
      .session(.prepareReflection(now: at(300), reading: TempoReading.silent)))
    let sheet: FinishSheetView = try XCTUnwrap(
      try bridge.rendered().activeSession?.record.finish)
    let row: FinishRowView = try XCTUnwrap(sheet.rows.first)
    XCTAssertEqual(sheet.rows.count, 1)

    let way = DraftWay(playId: row.playId, key: nil, variationIds: [dotted])
    _ = try bridge.update(
      .session(
        .submitReflection(
          nextItemStartedAt: at(301),
          answers: ReflectionAnswers(marks: [], note: "steady", tempos: [], ways: [way]))))

    XCTAssertNil(try bridge.rendered().error)
    let entry = try XCTUnwrap(try bridge.rendered().summary?.entries.first { $0.id == entryId })
    XCTAssertEqual(
      entry.plays.first?.variationIds, [dotted], "the confirmed way is what was played")
    XCTAssertEqual(entry.notes, "steady")
  }

  /// A practice in progress with every field set crosses Swift to Rust on
  /// resume and comes back in the next saved copy, unchanged.
  func testRealBridgeAPracticeInProgressRoundTripsWithEveryFieldSet() throws {
    let play = Play(
      id: "p1", key: nil, variationIds: [], startedAt: at(0), seconds: 0,
      tempoChanges: [], achievedTempo: nil,
      clickPattern: nil, score: nil,
      away: [
        Away(leftAt: at(60), backAt: at(420), leftOut: true),
        Away(leftAt: at(500), backAt: at(530), leftOut: false),
      ])
    let plan = PlannedItem(
      itemId: "i1", itemType: .exercise, intention: "A at 84", durationSecs: 1200,
      groupId: "block-1", variationIds: ["v-a", "v-b"],
      key: Key(letter: .g, accidental: .natural, mode: .major))
    let entry = SetlistEntry(
      id: "e1", plan: plan, itemTitle: "Nocturne", position: 0, durationSecs: 0,
      status: .notAttempted, notes: "bars 12-14 rough", plays: [play])
    let blob = ActiveSession(
      id: "resumed", entries: [entry], currentIndex: 0, currentItemStartedAt: at(0),
      sessionStartedAt: at(0), reflection: nil)
    XCTAssertEqual(
      try ActiveSession.bincodeDeserialize(input: try blob.bincodeSerialize()), blob,
      "the Swift decoder reads what the Swift encoder wrote")

    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(.session(.recoverSession(session: blob, now: at(0))))
    XCTAssertNil(try bridge.rendered().error)
    // The core reads the aim itself, so a swap of the two text fields in
    // the Swift encoder and decoder alike shows here, not in the copy back.
    XCTAssertEqual(
      try bridge.rendered().activeSession?.currentItemIntention, "A at 84",
      "the core reads the aim, not the group id (#2582)")
    let requests = try bridge.update(.session(.wentAway(at: at(700))))
    let saved = try XCTUnwrap(
      requests.lazy.compactMap { request -> ActiveSession? in
        if case .app(.saveSessionInProgress(let active)) = request.effect { return active }
        return nil
      }.first, "going away saves the copy")

    let back = try XCTUnwrap(saved.entries.first)
    XCTAssertEqual(back.plan, plan, "every plan field crosses the bridge (#2582)")
    XCTAssertEqual(back.status, entry.status)
    XCTAssertEqual(back.notes, entry.notes)
    XCTAssertEqual(back.plays.first?.away.prefix(2), play.away[...])
    XCTAssertEqual(back.plays.first?.away.last?.leftAt, at(700))
  }

}
