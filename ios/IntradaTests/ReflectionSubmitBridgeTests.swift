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
      tempos: [DraftTempo(playId: playId, tempo: 96, click: nil)], ways: [],
      notes: [DraftNote(playId: playId, note: "thumb under")])
  }

  private func saveSession(_ bridge: RowsBridge, now: String) throws {
    let write = try #require(
      try bridge.update(.session(.saveSession(now: now))).first {
        if case .persistence(.saveSession) = $0.effect { return true } else { return false }
      })
    _ = try bridge.resolve(write.id, persistenceOutput: .ack)
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
    let saved = try bridge.update(.session(.saveSession(now: "2026-09-21T10:06:00Z"))).lazy
      .compactMap { request -> PracticeSession? in
        if case .persistence(.saveSession(let session)) = request.effect { return session }
        return nil
      }.first
    #expect(saved?.entries.first?.plays.last?.note == "thumb under")
  }

  /// Last time's score crosses on the row, and an untouched score keeps it (#3004).
  @Test func aSecondPracticeStartsOnLastTimesScoreAndKeepsItUntouched() throws {
    let bridge = RowsBridge()
    let playId = try sheetOpen(bridge)
    _ = try bridge.update(
      .session(
        .submitReflection(
          nextItemStartedAt: "2026-09-21T10:05:30Z", answers: answers(playId: playId, note: ""))))
    try saveSession(bridge, now: "2026-09-21T10:06:00Z")
    let itemId = try #require(try bridge.rendered().items.first?.id)
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-22T10:00:00Z")))
    _ = try bridge.update(
      .session(.prepareReflection(now: "2026-09-22T10:05:00Z", reading: .silent)))

    let row = try #require(try bridge.rendered().activeSession?.reflection?.tempos.first)
    #expect(row.lastScore == 4)
    #expect(row.startScore == 4)

    _ = try bridge.update(
      .session(
        .submitReflection(
          nextItemStartedAt: "2026-09-22T10:05:30Z",
          answers: ReflectionAnswers(marks: [], note: "", tempos: [], ways: [], notes: []))))
    #expect(try bridge.rendered().summary?.entries.first?.plays.last?.score == 4)
  }

  /// The measured tempo card's words cross the wire inside the item's practice (#2486).
  @Test func aSavedTempoReachesTheTrendCardsWords() throws {
    let bridge = RowsBridge()
    let playId = try sheetOpen(bridge)
    _ = try bridge.update(
      .session(
        .submitReflection(
          nextItemStartedAt: "2026-09-21T10:05:30Z", answers: answers(playId: playId, note: ""))))
    let write = try #require(
      try bridge.update(.session(.saveSession(now: "2026-09-21T10:06:00Z"))).first {
        if case .persistence(.saveSession) = $0.effect { return true } else { return false }
      })
    _ = try bridge.resolve(write.id, persistenceOutput: .ack)
    let trend = try #require(try bridge.rendered().items.first?.practice?.tempoTrend)
    #expect(trend.chip == nil)
    #expect(trend.caption == "Measured once, at ♩ = 96 · no trend yet")
    #expect(trend.spoken == "Measured tempo, measured once at 96 beats per minute, no trend yet")
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

  /// An over-long key note from the key's sheet comes back as the message the sheet shows (#3004).
  @Test func aRefusedKeyNoteKeepsTheItemCurrentWithItsMessage() throws {
    let bridge = RowsBridge()
    let playId = try sheetOpen(bridge)

    _ = try bridge.update(
      .session(
        .submitReflection(
          nextItemStartedAt: "2026-09-21T10:05:30Z",
          answers: ReflectionAnswers(
            marks: [], note: "", tempos: [], ways: [],
            notes: [DraftNote(playId: playId, note: String(repeating: "a", count: 501))]))))

    #expect(try bridge.rendered().error == "Keep the note under 500 characters.")
    #expect(try bridge.rendered().activeSession?.reflection != nil, "the sheet stays up")
  }

  /// A key sheet's Done with an over-long note: the refusal comes back, and the
  /// rest of the draft still reaches crash recovery (#3004).
  @Test func anOverLongKeyNoteInTheDraftIsRefusedButTheMarkIsKept() throws {
    let bridge = RowsBridge()
    let playId = try sheetOpen(bridge)
    let limit = Int(try bridge.rendered().limits.playNoteMaxChars)

    _ = try bridge.update(
      .session(
        .updateReflectionDraft(
          answers: ReflectionAnswers(
            marks: [DraftMark(playId: playId, score: 8)], note: "", tempos: [], ways: [],
            notes: [DraftNote(playId: playId, note: String(repeating: "a", count: limit + 1))]))))

    #expect(limit == 500)
    #expect(try bridge.rendered().error == "Keep the note under 500 characters.")
    let draft = try #require(try bridge.rendered().activeSession?.reflection?.answers)
    #expect(draft.marks == [DraftMark(playId: playId, score: 8)])
    #expect(draft.notes.isEmpty)
  }

  /// A key never scored and left untouched saves the score it suggested (decided 2026-11-25).
  @Test func anUntouchedKeyNeverScoredSavesItsSuggestedScore() throws {
    let bridge = RowsBridge()
    _ = try sheetOpen(bridge)

    _ = try bridge.update(
      .session(
        .submitReflection(
          nextItemStartedAt: "2026-09-21T10:05:30Z",
          answers: ReflectionAnswers(marks: [], note: "", tempos: [], ways: [], notes: []))))

    #expect(try bridge.rendered().summary?.entries.first?.plays.last?.score == 5)
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
    #expect(row.lastScore == nil)
    #expect(row.startScore == 5)
  }
}
