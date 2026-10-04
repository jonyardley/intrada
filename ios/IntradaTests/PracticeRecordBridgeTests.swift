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

  /// A piece with sections A and B, built over twenty minutes as A then B with
  /// a tempo focus on A, and started.
  private func practisingAThenB() throws -> (RowsBridge, String, String, String) {
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
      .item(
        .updateSections(
          id: itemId,
          sections: ["A", "B"].map {
            SectionEdit(id: nil, name: $0, bars: .blank, kind: .form, targetBpm: "")
          })))
    let sections = try XCTUnwrap(try bridge.rendered().items.first?.sections)
    let (a, b) = (try XCTUnwrap(sections.first?.id), try XCTUnwrap(sections.last?.id))

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let building = try XCTUnwrap(try bridge.rendered().buildingSetlist)
    let lastTimes: [LastTimeView] = building.lastTimes
    XCTAssertEqual(lastTimes, [], "nothing played yet, so nothing to offer")
    let focusChoices: [FocusChoiceView] = building.focusChoices
    XCTAssertEqual(
      focusChoices.map(\.label), ["Tempo", "Clean in a row", "From memory", "Evenness"])
    XCTAssertEqual(
      focusChoices.map(\.kind),
      [FocusKind.tempo, FocusKind.cleanReps, FocusKind.fromMemory, FocusKind.evenness])
    let entryId = try XCTUnwrap(building.entries.first?.id)
    _ = try bridge.update(.session(.setEntryDuration(entryId: entryId, durationSecs: 1200)))
    let segments: [Segment] = [
      Segment(sectionId: a, plannedSecs: 0), Segment(sectionId: b, plannedSecs: 0),
    ]
    _ = try bridge.update(.session(.setSegments(entryId: entryId, segments: segments)))
    let focus = IntentionFocus(kind: FocusKind.tempo, sectionId: a, target: 84)
    _ = try bridge.update(.session(.setFocus(entryId: entryId, focus: focus)))
    XCTAssertNil(try bridge.rendered().error)

    let record: EntryRecordView = try XCTUnwrap(
      try bridge.rendered().buildingSetlist?.entries.first?.record)
    let planned: [SegmentView] = record.segments
    XCTAssertEqual(planned.map(\.plannedSecs), [600, 600], "the core splits twenty minutes evenly")
    XCTAssertEqual(planned.map(\.label), ["A", "B"])
    let shown: FocusView = try XCTUnwrap(record.focus)
    XCTAssertEqual(shown.focus, focus)
    XCTAssertEqual(shown.label, "A at 84")

    _ = try bridge.update(.session(.startSession(now: start)))
    return (bridge, entryId, a, b)
  }

  func testRealBridgeSegmentsAndTimeAwayRunThroughThePracticeScreen() throws {
    let (bridge, _, a, b) = try practisingAThenB()
    let live: ActiveRecordView = try XCTUnwrap(try bridge.rendered().activeSession?.record)
    let clock: SegmentClockView = try XCTUnwrap(live.segment)
    XCTAssertEqual(clock.label, "A")
    XCTAssertEqual(clock.moveLabel, "On to B")
    XCTAssertEqual(clock.stayLabel, "Stay on A, 2 more minutes taken from B")
    XCTAssertEqual(
      SessionClock.parseRFC3339(clock.endsAt), SessionClock.parseRFC3339(at(600)))
    XCTAssertEqual(try bridge.rendered().activeSession?.currentSectionId, a)

    _ = try bridge.update(.session(.wentAway(at: at(60))))
    _ = try bridge.update(.session(.cameBack(at: at(420))))
    let offer: AwayOfferView = try XCTUnwrap(try bridge.rendered().activeSession?.record.awayOffer)
    XCTAssertEqual(offer.minutes, 6)
    XCTAssertEqual(offer.label, "Away 6 minutes. Leave it out?")
    _ = try bridge.update(.session(.leaveAwayOut))
    XCTAssertNil(try bridge.rendered().activeSession?.record.awayOffer)

    _ = try bridge.update(.session(.stayOnSegment))
    _ = try bridge.update(.session(.moveToNextSegment(now: at(1000), reading: sounding(84))))
    let active = try XCTUnwrap(try bridge.rendered().activeSession)
    XCTAssertEqual(active.currentSectionId, b)
    let first = try XCTUnwrap(active.entries.first?.plays.first)
    XCTAssertEqual(first.seconds, 1000 - 360, "six minutes left out")
    XCTAssertEqual(active.record.segment?.label, "B")
    XCTAssertNil(active.record.segment?.moveLabel, "the last segment offers nothing")
  }

  func testRealBridgeTheFinishSheetsAnswersLandOnTheEntry() throws {
    let (bridge, entryId, a, _) = try practisingAThenB()
    _ = try bridge.update(.session(.moveToNextSegment(now: at(600), reading: sounding(84))))
    _ = try bridge.update(.session(.prepareReflection(now: at(900), reading: sounding(80))))

    let sheet: FinishSheetView = try XCTUnwrap(try bridge.rendered().activeSession?.record.finish)
    XCTAssertEqual(sheet.intentionMetRead, IntentionMet.yes, "A reached 84 with the click")
    XCTAssertFalse(sheet.asksIntention)
    let felt: [FeltChoiceView] = sheet.feltChoices
    XCTAssertEqual(felt.map(\.label), ["Comfortable", "Hard work", "Strained"])
    XCTAssertEqual(felt.map(\.felt), [Felt.comfortable, Felt.hardWork, Felt.strained])
    let obstacles: [ObstacleChoiceView] = sheet.obstacleChoices
    XCTAssertEqual(obstacles.first?.obstacle, Obstacle.notes)

    let note = "A section, bar 12 rushed at 84"
    let bar12 = NoteSpan(start: 11, end: 17)
    let answers = ReflectionAnswers(
      marks: [], note: note, tempos: [], felt: .strained, gotInTheWay: [.memory],
      notePoints: [bar12], intentionMet: nil)
    _ = try bridge.update(.session(.updateReflectionDraft(answers: answers)))
    let offers: [NotePointView] = try XCTUnwrap(
      try bridge.rendered().activeSession?.record.finish?.noteOffers)
    XCTAssertEqual(offers.map(\.label), ["Bar 12", "\u{2669} = 84"])
    XCTAssertEqual(offers.map(\.confirmed), [true, false])
    XCTAssertEqual(offers.first?.sectionLabel, "A")
    XCTAssertEqual(try bridge.rendered().activeSession?.reflection?.answers, answers)

    _ = try bridge.update(.session(.updateEntryNotes(entryId: entryId, notes: note)))
    _ = try bridge.update(
      .session(.nextItem(now: at(900), nextItemStartedAt: at(900), reading: sounding(80))))
    for event: SessionEvent in [
      .confirmNotePoint(entryId: entryId, span: bar12),
      .setFelt(entryId: entryId, felt: .strained),
      .toggleObstacle(entryId: entryId, obstacle: .memory),
      .answerIntention(entryId: entryId, answer: .notYet),
    ] {
      _ = try bridge.update(.session(event))
    }

    let entry = try XCTUnwrap(try bridge.rendered().summary?.entries.first)
    XCTAssertEqual(entry.record.felt, .strained)
    XCTAssertEqual(entry.record.gotInTheWay, [.memory])
    XCTAssertEqual(entry.record.notePoints.map(\.label), ["Bar 12"])
    XCTAssertEqual(entry.record.intentionMet, .notYet, "their own answer shows over the read")
    XCTAssertFalse(entry.record.intentionMetRead)
    XCTAssertEqual(entry.plays.first?.sectionId, a)
  }

  /// A v7 blob with every new field set goes Swift to Rust on resume and
  /// comes back Rust to Swift in the next saved copy, unchanged.
  func testRealBridgeAV7PracticeInProgressRoundTripsWithEveryFieldSet() throws {
    let play = Play(
      id: "p1", sectionId: "s-a", key: nil, variationIds: [], startedAt: at(0), seconds: 0,
      repTarget: nil, repCount: nil, repHistory: nil, tempoChanges: [], achievedTempo: nil,
      clickPattern: nil, score: nil,
      away: [
        Away(leftAt: at(60), backAt: at(420), leftOut: true),
        Away(leftAt: at(500), backAt: at(530), leftOut: false),
      ])
    let point = NotePoint(
      kind: NotePointKind.bars(BarRange(first: 12, last: 14)), sectionId: "s-a",
      span: NoteSpan(start: 0, end: 12))
    let entry = SetlistEntry(
      id: "e1", itemId: "i1", itemTitle: "Nocturne", itemType: .piece, position: 0,
      durationSecs: 0, status: .notAttempted, notes: "bars 12-14 rough", intention: "A at 84",
      plannedDurationSecs: 1200, groupId: nil, plannedVariationIds: [], plannedRepTarget: nil,
      plays: [play],
      segments: [
        Segment(sectionId: "s-a", plannedSecs: 600), Segment(sectionId: "s-b", plannedSecs: 600),
      ],
      focus: IntentionFocus(kind: .cleanReps, sectionId: "s-b", target: 5),
      intentionMet: .partly, felt: .hardWork, gotInTheWay: [.fingering, .tone],
      notePoints: [point])
    let clock = SegmentClock(
      index: 0, startedAt: at(0), allowanceSecs: 720, takenFromNextSecs: 120, leftOutSecs: 45)
    let blob = ActiveSession(
      id: "v7", entries: [entry], currentIndex: 0, currentItemStartedAt: at(0),
      sessionStartedAt: at(0), reflection: nil, segment: clock)
    XCTAssertEqual(
      try ActiveSession.bincodeDeserialize(input: try blob.bincodeSerialize()), blob,
      "the Swift decoder reads what the Swift encoder wrote")

    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(.session(.recoverSession(session: blob, now: at(0))))
    XCTAssertNil(try bridge.rendered().error)
    let requests = try bridge.update(.session(.wentAway(at: at(700))))
    let saved = try XCTUnwrap(
      requests.lazy.compactMap { request -> ActiveSession? in
        if case .app(.saveSessionInProgress(let active)) = request.effect { return active }
        return nil
      }.first, "going away saves the copy")

    let back = try XCTUnwrap(saved.entries.first)
    XCTAssertEqual(back.segments, entry.segments)
    XCTAssertEqual(back.focus, entry.focus)
    XCTAssertEqual(back.intentionMet, entry.intentionMet)
    XCTAssertEqual(back.felt, entry.felt)
    XCTAssertEqual(back.gotInTheWay, entry.gotInTheWay)
    XCTAssertEqual(back.notePoints, entry.notePoints)
    XCTAssertEqual(back.plays.first?.away.prefix(2), play.away[...])
    XCTAssertEqual(back.plays.first?.away.last?.leftAt, at(700))
    XCTAssertEqual(saved.segment?.allowanceSecs, 720)
    XCTAssertEqual(saved.segment?.takenFromNextSecs, 120)
    XCTAssertEqual(saved.segment?.leftOutSecs, 45)
  }

  func testRealBridgeEachFeltChoiceRoundTripsInAPracticeInProgress() throws {
    for felt: Felt in [.comfortable, .hardWork, .strained] {
      let entry = SetlistEntry(
        id: "e1", itemId: "i1", itemTitle: "Nocturne", itemType: .piece, position: 0,
        durationSecs: 0, status: .notAttempted, notes: nil, intention: nil,
        plannedDurationSecs: nil, groupId: nil, plannedVariationIds: [], plannedRepTarget: nil,
        plays: [
          Play(
            id: "p1", sectionId: nil, key: nil, variationIds: [], startedAt: at(0), seconds: 0,
            repTarget: nil, repCount: nil, repHistory: nil, tempoChanges: [], achievedTempo: nil,
            clickPattern: nil, score: nil, away: [])
        ], segments: [], focus: nil, intentionMet: nil, felt: felt, gotInTheWay: [],
        notePoints: [])
      let blob = ActiveSession(
        id: "felt", entries: [entry], currentIndex: 0, currentItemStartedAt: at(0),
        sessionStartedAt: at(0), reflection: nil, segment: nil)

      let bridge = RowsBridge()
      _ = try bridge.update(.startApp)
      _ = try bridge.update(.session(.recoverSession(session: blob, now: at(0))))
      XCTAssertNil(try bridge.rendered().error)
      let requests = try bridge.update(.session(.wentAway(at: at(60))))
      let saved = try XCTUnwrap(
        requests.lazy.compactMap { request -> ActiveSession? in
          if case .app(.saveSessionInProgress(let active)) = request.effect { return active }
          return nil
        }.first, "going away saves the copy")
      XCTAssertEqual(saved.entries.first?.felt, felt)
    }
  }
}
