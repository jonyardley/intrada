import IntradaCoreFFI
import SharedTypes
import XCTest

@testable import Intrada

@MainActor
final class SessionBridgeTests: XCTestCase {

  // ── Real bridge (Swift↔Rust bincode round-trip) ────────────────────────

  /// Real-bridge rung tag (#846, #1083): `SetEntryVariant` carries an optional
  /// String across the bincode wire (the absent-vs-present hazard). Drive it
  /// through the live bridge and assert it decodes without error.
  func testRealBridgeTagEntryWithVariantDecodesOnWire() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Scales", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let itemId = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(.item(.setVariants(id: itemId, labels: ["F major"])))
    let stepId = try XCTUnwrap(try bridge.view().items.first?.variants.first?.id)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.view().buildingSetlist?.entries.first?.id)

    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: stepId)))
    XCTAssertNil(try bridge.view().error, "tagging a rung must decode on the wire (#846)")
    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: nil)))
    XCTAssertNil(try bridge.view().error, "clearing the rung round-trips")
  }

  /// Answers the session write the way GRDB does, so the core commits the
  /// practice, clears the recovery copy and closes the summary (#974).
  private func acknowledgeSave(_ bridge: LiveBridge, _ requests: [Request]) throws {
    let write = try XCTUnwrap(
      requests.first {
        if case .persistence(.saveSession) = $0.effect { return true } else { return false }
      },
      "saveSession sends the write to the store")
    _ = try bridge.resolve(write.id, persistenceOutput: .ack)
  }

  private func bridgeWithCompletedEntry() throws -> (LiveBridge, String, String) {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Scales", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Arpeggios", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let ids = try bridge.view().items.map(\.id)
    XCTAssertEqual(ids.count, 2, "two distinct items: addToSetlist is idempotent by item id")

    _ = try bridge.update(.session(.startBuilding))
    for id in ids { _ = try bridge.update(.session(.addToSetlist(itemId: id))) }
    _ = try bridge.update(.session(.startSession(now: "2026-08-28T09:00:00Z")))
    // Advancing completes the first entry; tempo only lands on a completed one.
    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-08-28T09:10:00Z", nextItemStartedAt: "2026-08-28T09:10:00Z", reading: .silent))
    )

    let entries = try XCTUnwrap(try bridge.view().activeSession?.entries)
    XCTAssertEqual(
      entries.first?.status, .completed,
      "the negative test must be asserting against a real completed entry")
    let entry = try XCTUnwrap(entries.first)
    let playId = try XCTUnwrap(
      entry.plays.last?.id, "a completed entry records the stretch it was practised as")
    return (bridge, entry.id, playId)
  }

  /// Real-bridge wire pin for the pass counter (#1367): the timestamped tap
  /// events and the drawn-slots projection cross the bincode wire, and the
  /// core's rule holds end to end: an untouched entry records nothing, and the
  /// first tap writes the target along with itself.
  func testRealBridgeFirstTapWritesTheTargetAndItsTime() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Scales", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: id)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-03T09:00:00Z")))

    let untouched = try XCTUnwrap(try bridge.view().activeSession)
    XCTAssertNil(untouched.currentRepTarget, "nothing is recorded before the first tap")
    XCTAssertNil(untouched.currentRepCount)
    XCTAssertNil(untouched.currentRepHistory)
    XCTAssertEqual(untouched.currentRepSlots, 10, "the counter still has slots to draw")

    _ = try bridge.update(.session(.repGotIt(now: "2026-09-03T09:01:00Z")))
    _ = try bridge.update(.session(.repMissed(now: "2026-09-03T09:01:40Z")))

    let tapped = try XCTUnwrap(try bridge.view().activeSession)
    XCTAssertNil(try bridge.view().error)
    XCTAssertEqual(tapped.currentRepTarget, 10, "the first tap wrote the default target")
    XCTAssertEqual(tapped.currentRepCount, 0)
    let history = try XCTUnwrap(tapped.currentRepHistory)
    XCTAssertEqual(history.map(\.action), [.success, .missed])
    XCTAssertEqual(
      history.map { SessionClock.parseRFC3339($0.at) },
      [
        SessionClock.parseRFC3339("2026-09-03T09:01:00Z"),
        SessionClock.parseRFC3339("2026-09-03T09:01:40Z"),
      ], "each tap keeps the time the shell gave it")
  }

  /// Moved from `RepetitionCounterUITests` (#1825): a miss at zero floors rather than going negative.
  func testRealBridgeAMissAtZeroHoldsTheFloor() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Scales", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: id)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-14T09:00:00Z")))

    _ = try bridge.update(.session(.repGotIt(now: "2026-09-14T09:00:05Z")))
    XCTAssertEqual(
      try bridge.view().activeSession?.currentRepCount, 1, "a bank moves the counter up")

    _ = try bridge.update(.session(.repMissed(now: "2026-09-14T09:00:10Z")))
    XCTAssertEqual(
      try bridge.view().activeSession?.currentRepCount, 0, "a miss steps the banked count back")

    _ = try bridge.update(.session(.repMissed(now: "2026-09-14T09:00:20Z")))
    XCTAssertEqual(
      try bridge.view().activeSession?.currentRepCount, 0,
      "a second miss at zero stays on the floor, never negative")
  }

  /// Real-bridge wire pin for the manual path (#1499, #1761): a row set by hand
  /// to `♪ = 168` in 7/8 stores 84 crotchets, and never the pattern, which only
  /// a close writes.
  func testRealBridgeStoresAHandSetQuaverTempoAsCrotchets() throws {
    let (bridge, entryId, playId) = try bridgeWithCompletedEntry()
    let pattern = ClickState(
      metre: Metre(beats: 7, unit: 8, groups: [3, 2, 2]), sounding: 0b0101001)

    _ = try bridge.update(
      .session(
        .updateEntryTempo(
          entryId: entryId, playId: playId, tempo: 168, userSet: true, click: pattern)))

    let entry = try XCTUnwrap(
      try bridge.view().activeSession?.entries.first { $0.id == entryId })
    let play = try XCTUnwrap(entry.plays.last)
    XCTAssertEqual(play.achievedTempo, 84, "stored in crotchets, not quavers")
    XCTAssertNil(play.clickPattern, "a tempo set by hand never stores a pattern")
  }

  /// Real-bridge wire pin for the tempo evidence contract (#1420, #1761): the
  /// `userSet` flag has to cross the bincode wire intact and the
  /// core's ruling has to hold end to end. A wire break would let an
  /// unevidenced default through, and the trend would draw it as a measurement.
  func testRealBridgeRecordsATempoTheUserSetThemselves() throws {
    let (bridge, entryId, playId) = try bridgeWithCompletedEntry()

    _ = try bridge.update(
      .session(
        .updateEntryTempo(
          entryId: entryId, playId: playId, tempo: 132,
          userSet: true, click: nil)))

    let view = try bridge.view()
    XCTAssertNil(view.error, "the flag must decode on the wire (#846)")
    XCTAssertEqual(
      view.activeSession?.entries.first?.plays.last?.achievedTempo, 132,
      "a tempo the user set themselves is a measurement")
  }

  func testRealBridgeDoesNotRecordAnUntouchedPreFill() throws {
    let (bridge, entryId, playId) = try bridgeWithCompletedEntry()

    _ = try bridge.update(
      .session(
        .updateEntryTempo(
          entryId: entryId, playId: playId, tempo: 96,
          userSet: false, click: nil)))

    let view = try bridge.view()
    XCTAssertNil(view.error, "declining to record is a silent success, not an error")
    let play = try XCTUnwrap(
      view.activeSession?.entries.first?.plays.last,
      "the play the tempo was declined for is still on the entry")
    XCTAssertNil(
      play.achievedTempo,
      "a pre-fill nobody looked at leaves no point for the trend to draw")
  }

  /// "Practise this" (#1034): StartBuildingWith is a new bridge-crossing
  /// write, round-trip it through the real bincode bridge (#846).
  func testRealBridgePractiseThisSeedsBuilder() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Hanon No. 1", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let itemId = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(.session(.startBuildingWith(itemId: itemId)))
    let vm = try bridge.view()
    XCTAssertNotNil(vm.buildingSetlist, "startBuildingWith should open a seeded setlist")
    XCTAssertEqual(vm.buildingSetlist?.entries.count, 1)
    XCTAssertEqual(vm.buildingSetlist?.entries.first?.itemId, itemId)
    XCTAssertNil(vm.error)
  }

  /// "Practise your priorities" (#981): the event carries a timestamp, which is
  /// a different bincode shape than the itemId write above. A wire break would
  /// open an empty builder rather than fail, which is the #846 shape.
  func testRealBridgePrioritiesSeedTheBuilder() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)

    for title in ["Hanon No. 1", "Scales"] {
      _ = try bridge.update(
        .item(
          .add(
            CreateItem(
              title: title, kind: .exercise, composer: nil, key: nil, modality: nil,
              tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    }
    for item in try bridge.view().items {
      _ = try bridge.update(
        .item(
          .update(
            id: item.id,
            input: UpdateItem(
              title: item.title, kind: item.itemType, composer: nil, key: nil, modality: nil,
              tempo: nil, notes: nil, tags: nil, priority: true))))
    }
    XCTAssertTrue(try bridge.view().showsPriorities, "starred with nothing under way")

    _ = try bridge.update(.session(.startBuildingWithPriorities(now: SessionClock.nowRFC3339())))
    let vm = try bridge.view()
    XCTAssertEqual(
      vm.buildingSetlist?.entries.count, 2, "every starred item should reach the builder")
    XCTAssertNil(vm.error)
    XCTAssertFalse(vm.showsPriorities, "a session is now being built")
  }

  /// Real-bridge build→play→save lifecycle (#932): drives the actual bincode
  /// bridge through Building → Active → Summary → Idle, mirroring the
  /// SessionBuilder → FocusPlayer → Summary screens. A wire break surfaces here
  /// as a failed transition instead of the silent no-op the stub bridge would
  /// hide (#846).
  func testRealBridgeSessionFlowBuildPlaySave() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Etude", kind: .piece, composer: "Chopin", key: nil, modality: nil,
            tempo: nil, notes: "Watch the thumb crossing", tags: [], photoId: nil,
            variantLabels: []))))
    let itemId = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let building = try bridge.view()
    XCTAssertNotNil(building.buildingSetlist, "startBuilding + add should open a setlist")
    XCTAssertEqual(building.buildingSetlist?.entries.count, 1)
    XCTAssertNil(building.activeSession)

    _ = try bridge.update(.session(.startSession(now: "2026-06-16T10:00:00Z")))
    let active = try bridge.view()
    XCTAssertNotNil(active.activeSession, "startSession should enter the player")
    XCTAssertNil(active.buildingSetlist, "the builder should close on start")
    XCTAssertNil(active.summary)
    XCTAssertEqual(active.activeSession?.currentItemNotes, "Watch the thumb crossing")

    // Advancing past the last item is the only way a session finishes (#1761).
    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-06-16T10:20:00Z", nextItemStartedAt: "2026-06-16T10:20:00Z", reading: .silent))
    )
    let summary = try bridge.view()
    XCTAssertNotNil(summary.summary, "advancing past the last item should reach the summary")
    XCTAssertNil(summary.activeSession)
    XCTAssertEqual(summary.summary?.completedCount, 1, "the one item played counts as done")

    // Optional-payload events crossing bincode (the absent-vs-present wire
    // hazard, #846): set then clear a score and the session notes.
    let entryId = try XCTUnwrap(summary.summary?.entries.first?.id)
    let playId = try XCTUnwrap(summary.summary?.entries.first?.plays.last?.id)
    _ = try bridge.update(
      .session(.updateEntryScore(entryId: entryId, playId: playId, score: 4)))
    XCTAssertEqual(
      try bridge.view().summary?.entries.first?.plays.last?.score, 4,
      "score should round-trip")
    _ = try bridge.update(
      .session(.updateEntryScore(entryId: entryId, playId: playId, score: nil)))
    XCTAssertNil(
      try bridge.view().summary?.entries.first?.plays.last?.score,
      "clearing a score round-trips")
    // Per-entry notes: the hand-off reflection sheet's write, never previously
    // sent from Swift (#846). Round-trip set + clear through the live bridge.
    _ = try bridge.update(
      .session(.updateEntryNotes(entryId: entryId, notes: "RH evenness better at 96")))
    XCTAssertEqual(
      try bridge.view().summary?.entries.first?.notes, "RH evenness better at 96",
      "the reflection sheet's per-entry note should round-trip")
    _ = try bridge.update(.session(.updateEntryNotes(entryId: entryId, notes: nil)))
    XCTAssertNil(
      try bridge.view().summary?.entries.first?.notes, "clearing an entry note round-trips")
    // Achieved tempo: the hand-off sheet's TempoStepper write, never previously
    // sent from Swift (#846). Round-trip set + clear through the live bridge.
    _ = try bridge.update(
      .session(
        .updateEntryTempo(
          entryId: entryId, playId: playId, tempo: 96,
          userSet: true, click: nil)))
    XCTAssertEqual(
      try bridge.view().summary?.entries.first?.plays.last?.achievedTempo, 96,
      "the tempo stepper's achieved tempo should round-trip")
    _ = try bridge.update(
      .session(
        .updateEntryTempo(
          entryId: entryId, playId: playId, tempo: nil,
          userSet: true, click: nil)))
    XCTAssertNil(
      try bridge.view().summary?.entries.first?.plays.last?.achievedTempo,
      "clearing an achieved tempo round-trips")
    _ = try bridge.update(.session(.updateSessionNotes(notes: "Felt good")))
    XCTAssertEqual(try bridge.view().summary?.notes, "Felt good", "notes should round-trip")
    _ = try bridge.update(.session(.updateSessionNotes(notes: nil)))
    XCTAssertNil(try bridge.view().summary?.notes, "clearing notes round-trips")

    let saveRequests = try bridge.update(.session(.saveSession(now: "2026-06-16T10:20:30Z")))
    XCTAssertNotNil(
      try bridge.view().summary,
      "the summary waits for the store to confirm the save (#974)")
    try acknowledgeSave(bridge, saveRequests)
    let saved = try bridge.view()
    XCTAssertNil(saved.summary, "saveSession clears the summary once the store has it")
    XCTAssertNil(saved.activeSession)
    XCTAssertNil(saved.error, "a clean save surfaces no error")

    // Crash recovery: RecoverSession (with its new `now` re-anchor field) has
    // never crossed the live bridge from Swift before (#846, #962). The stale
    // anchor must come back re-anchored to the `now` we send.
    let blobEntry = SetlistEntry(
      id: "re1", itemId: "i1", itemTitle: "Recovered Scales", itemType: .exercise,
      position: 0, durationSecs: 0, status: .notAttempted,
      notes: nil, intention: nil, plannedDurationSecs: nil,
      groupId: nil, plannedVariationId: nil, plannedRepTarget: nil, plays: [])
    let blob = ActiveSession(
      id: "recovered", entries: [blobEntry], currentIndex: 0,
      currentItemStartedAt: "2026-06-16T08:00:00Z", sessionStartedAt: "2026-06-16T08:00:00Z")
    _ = try bridge.update(.session(.recoverSession(session: blob, now: "2026-06-16T11:00:00Z")))
    let recovered = try bridge.view()
    XCTAssertEqual(recovered.activeSession?.currentItemTitle, "Recovered Scales")
    XCTAssertEqual(
      recovered.activeSession?.currentItemStartedAt, "2026-06-16T11:00:00+00:00",
      "recovery must re-anchor the running item's timer to `now`")
  }

  /// Real-bridge step-ladder lifecycle (#1083 C1): SetVariants and
  /// UpdateEntryVariant have never crossed the live bincode bridge from Swift
  /// before; a wire break here is the silent no-op class (#846). Drives
  /// ladder create → reorder-preserves-ids → per-entry attribution set/clear.
  func testRealBridgeStepLadderAndEntryAttribution() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Shells", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let exId = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(.item(.setVariants(id: exId, labels: ["C", "F", "B♭"])))
    let afterSet = try bridge.view()
    let ladder = try XCTUnwrap(afterSet.items.first?.variants)
    XCTAssertEqual(
      ladder.map(\.label), ["C", "F", "B♭"],
      "the ladder should land (err=\(afterSet.error ?? "nil"))")
    XCTAssertTrue(ladder.allSatisfy { !$0.isSolid }, "an unrated ladder has no solid steps")
    let stepId = try XCTUnwrap(ladder.first?.id)

    // Reorder must keep ids (and so score history); reconcile-by-label.
    _ = try bridge.update(.item(.setVariants(id: exId, labels: ["F", "C", "B♭"])))
    let reordered = try XCTUnwrap(try bridge.view().items.first?.variants)
    XCTAssertEqual(
      reordered.first { $0.label == "C" }?.id, stepId, "reordering keeps each step's id")

    // #1739 makes SetEntryVariant the builder's plan, so set + clear it there.
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: exId)))
    let entryId = try XCTUnwrap(try bridge.view().buildingSetlist?.entries.first?.id)

    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: stepId)))
    let attributed = try bridge.view()
    XCTAssertEqual(
      attributed.buildingSetlist?.entries.first?.plannedVariationId, stepId,
      "the step attribution should round-trip (err=\(attributed.error ?? "nil"))")
    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: nil)))
    XCTAssertNil(
      try bridge.view().buildingSetlist?.entries.first?.plannedVariationId,
      "clearing the attribution round-trips")
  }

  // ── Builder reorder ────────────────────────────────────────────────────

  /// A built block of Clair de Lune with Hanon and Scales related, then
  /// Arpeggios standing alone after it.
  private func bridgeBuildingABlockAndAStandalone() throws -> LiveBridge {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    var ids: [String: String] = [:]
    for (title, kind) in [
      ("Clair de Lune", ItemKind.piece), ("Hanon No. 1", .exercise), ("Major Scales", .exercise),
      ("Arpeggios", .exercise),
    ] {
      _ = try bridge.update(
        .item(
          .add(
            CreateItem(
              title: title, kind: kind, composer: nil, key: nil, modality: nil, tempo: nil,
              notes: nil, tags: [], photoId: nil, variantLabels: []))))
      ids[title] = try XCTUnwrap(try bridge.view().items.first { $0.title == title }?.id)
    }
    let pieceId = try XCTUnwrap(ids["Clair de Lune"])
    for exercise in ["Hanon No. 1", "Major Scales"] {
      _ = try bridge.update(
        .item(.linkExercise(pieceId: pieceId, exerciseId: try XCTUnwrap(ids[exercise]))))
    }
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: pieceId)))
    _ = try bridge.update(.session(.addToSetlist(itemId: try XCTUnwrap(ids["Arpeggios"]))))
    return bridge
  }

  /// `moveRelated` crosses the wire and swaps the block's exercises; the
  /// piece still closes the block (#1957).
  func testRealBridgeMoveRelatedSwapsTheBlocksExercises() throws {
    let bridge = try bridgeBuildingABlockAndAStandalone()
    let before = try XCTUnwrap(try bridge.view().buildingSetlist)
    let block = try XCTUnwrap(before.blocks.first { $0.groupId != nil })
    guard block.entries.count == 3 else {
      return XCTFail("both linked exercises join the piece's block, got \(block.entries.count)")
    }

    _ = try bridge.update(.session(.moveRelated(entryId: block.entries[1].id, newPosition: 0)))

    let after = try XCTUnwrap(try bridge.view().buildingSetlist)
    XCTAssertNil(try bridge.view().error)
    XCTAssertEqual(
      after.blocks.first { $0.groupId == block.groupId }?.entries.map(\.id),
      [block.entries[1].id, block.entries[0].id, block.entries[2].id])
  }

  /// `entryVariations` is a new struct on the wire; the entry settings sheet
  /// reads its variations from it (#1957).
  func testRealBridgeBuilderEntryCarriesItsVariations() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Scales", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: ["C", "G"]))))
    let itemId = try XCTUnwrap(try bridge.view().items.first?.id)
    _ = try bridge.update(.session(.startBuildingWith(itemId: itemId)))

    let building = try XCTUnwrap(try bridge.view().buildingSetlist)
    let entryId = try XCTUnwrap(building.entries.first?.id)
    XCTAssertEqual(building.entryVariations.map(\.entryId), [entryId])
    XCTAssertEqual(building.entryVariations.first?.variations.map(\.label), ["C", "G"])
  }

  /// `moveUnit` crosses the wire: named by one of its exercises, the whole
  /// block moves past the standalone (#1957).
  func testRealBridgeMoveUnitMovesTheWholeBlock() throws {
    let bridge = try bridgeBuildingABlockAndAStandalone()
    let before = try XCTUnwrap(try bridge.view().buildingSetlist)
    let block = try XCTUnwrap(before.blocks.first)

    _ = try bridge.update(.session(.moveUnit(entryId: block.entries[1].id, newPosition: 1)))

    let after = try XCTUnwrap(try bridge.view().buildingSetlist)
    XCTAssertNil(try bridge.view().error)
    XCTAssertEqual(after.blocks.map(\.pieceTitle), [nil, "Clair de Lune"])
    XCTAssertEqual(after.blocks.last?.entries.map(\.id), block.entries.map(\.id))
  }

  /// A block names the items elsewhere in the session that it cannot take
  /// (#2075); the list is the trailing field, so a slip reads as empty.
  func testRealBridgeBlockNamesWhatIsTakenElsewhere() throws {
    let bridge = try bridgeBuildingABlockAndAStandalone()
    let view = try bridge.view()
    XCTAssertNil(view.error)
    let arpeggiosId = try XCTUnwrap(view.items.first { $0.title == "Arpeggios" }?.id)
    let setlist = try XCTUnwrap(view.buildingSetlist)
    let block = try XCTUnwrap(setlist.blocks.first { $0.groupId != nil })
    XCTAssertEqual(block.takenElsewhere, [arpeggiosId])
    let standalone = try XCTUnwrap(setlist.blocks.first { $0.groupId == nil })
    XCTAssertEqual(standalone.takenElsewhere, [], "only a grouped block lists what it cannot take")
  }

  // ── Real bridge (full-field contract, #846 class) ──────────────────────

  /// Real-bridge full-field round trip for `SetlistEntryView` and the
  /// `VariationPlayView` under it (#846, #1083, #1420, #1499, #1739): drives
  /// every setter that reaches a single entry (intention, rep target, planned
  /// duration, ladder attribution, block grouping, reps, tempo + click pattern,
  /// notes, score) and asserts every field of the resulting projection, so a
  /// dropped field anywhere in that pair fails here even though each setter's
  /// own dedicated spot-check elsewhere would stay green.
  func testRealBridgeSessionEntryFullFieldRoundTrip() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Prelude in C", kind: .piece, composer: "J.S. Bach", key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let pieceId = try XCTUnwrap(try bridge.view().items.first?.id)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Hanon No. 1", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let exerciseId = try XCTUnwrap(try bridge.view().items.first { $0.id != pieceId }?.id)
    _ = try bridge.update(.item(.linkExercise(pieceId: pieceId, exerciseId: exerciseId)))
    _ = try bridge.update(.item(.setVariants(id: exerciseId, labels: ["Slow"])))
    let variantId = try XCTUnwrap(
      try bridge.view().items.first { $0.id == exerciseId }?.variants.first?.id)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: pieceId)))
    let building = try bridge.view()
    let entry = try XCTUnwrap(
      building.buildingSetlist?.entries.first { $0.itemId == exerciseId },
      "the linked exercise should form a block with the piece")
    let entryId = entry.id
    let groupId = try XCTUnwrap(entry.groupId, "the piece's related exercise forms a block")

    _ = try bridge.update(
      .session(.setEntryIntention(entryId: entryId, intention: "Warm up before the Prelude")))
    _ = try bridge.update(.session(.setRepTarget(entryId: entryId, target: 3)))
    _ = try bridge.update(.session(.setEntryDuration(entryId: entryId, durationSecs: 300)))
    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: variantId)))

    _ = try bridge.update(.session(.startSession(now: "2026-09-04T09:00:00Z")))
    _ = try bridge.update(.session(.repMissed(now: "2026-09-04T09:01:00Z")))
    _ = try bridge.update(.session(.repGotIt(now: "2026-09-04T09:02:00Z")))
    _ = try bridge.update(.session(.repGotIt(now: "2026-09-04T09:03:00Z")))
    _ = try bridge.update(.session(.repGotIt(now: "2026-09-04T09:03:30Z")))
    let click = ClickState(metre: Metre(beats: 4, unit: 8, groups: [2, 2]), sounding: 0b0101)
    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-04T09:04:00Z", nextItemStartedAt: "2026-09-04T09:04:00Z",
          reading: TempoReading(bpm: 176, clickSounding: true, click: click))))

    let playId = try XCTUnwrap(
      try bridge.view().activeSession?.entries.first { $0.id == entryId }?.plays.last?.id,
      "the completed entry records the stretch it was practised as")
    _ = try bridge.update(
      .session(.updateEntryNotes(entryId: entryId, notes: "Fingers not fully relaxed yet")))
    _ = try bridge.update(
      .session(.updateEntryScore(entryId: entryId, playId: playId, score: 6)))

    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-04T09:10:00Z", nextItemStartedAt: "2026-09-04T09:10:00Z", reading: .silent))
    )

    let view = try bridge.view()
    XCTAssertNil(view.error, "every setter must decode cleanly (#846)")
    let summary = try XCTUnwrap(view.summary)
    let final = try XCTUnwrap(summary.entries.first { $0.id == entryId })

    XCTAssertEqual(final.itemId, exerciseId)
    XCTAssertEqual(final.itemTitle, "Hanon No. 1")
    XCTAssertEqual(final.itemType, .exercise)
    XCTAssertEqual(final.position, 0, "the related exercise leads the block")
    XCTAssertFalse(final.durationDisplay.isEmpty)
    XCTAssertEqual(final.status, .completed)
    XCTAssertEqual(final.notes, "Fingers not fully relaxed yet")
    XCTAssertEqual(final.intention, "Warm up before the Prelude")
    XCTAssertEqual(final.plannedDurationSecs, 300)
    XCTAssertNotNil(final.plannedDurationDisplay)
    XCTAssertEqual(final.groupId, groupId)
    XCTAssertEqual(final.plannedVariationId, variantId)
    XCTAssertEqual(final.plannedRepTarget, 3)
    XCTAssertEqual(final.scoreSummary, 6, "one play's mark is the whole entry's")

    XCTAssertEqual(final.plays.count, 1, "one stretch of practice, never switched")
    let play = try XCTUnwrap(final.plays.last)
    XCTAssertEqual(play.id, playId)
    XCTAssertEqual(play.variationId, variantId, "the plan seeds the play it opens")
    XCTAssertEqual(play.variationLabel, "Slow")
    XCTAssertFalse(play.durationDisplay.isEmpty)
    XCTAssertEqual(play.score, 6)
    XCTAssertEqual(play.repTarget, 3)
    XCTAssertEqual(play.repCount, 3, "missed then three got-its: 0, 1, 2, 3")
    XCTAssertEqual(play.repTargetReached, true)
    let history = try XCTUnwrap(play.repHistory)
    XCTAssertEqual(history.map(\.action), [.missed, .success, .success, .success])
    XCTAssertEqual(play.achievedTempo, 88, "176 quavers halves to 88 crotchets")
    XCTAssertEqual(play.tempoDisplay, 176, "and reads back in the click's quavers")
    XCTAssertEqual(play.clickPattern, click)
  }

  /// Real-bridge cross-domain round trip (#846, #1083): a session-side score,
  /// attributed to a ladder step via `SetEntryVariant` + `UpdateEntryScore`,
  /// must reappear as that step's `VariantView.latestScore` / `scoreHistory` /
  /// `isSolid` once the session is saved, a derivation that crosses BOTH the
  /// session and item domains, so a wire break in either side can hide behind
  /// the other's fields looking fine.
  func testRealBridgeVariantScoreAggregatesIntoLadderView() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Hanon No. 1", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let itemId = try XCTUnwrap(try bridge.view().items.first?.id)
    _ = try bridge.update(.item(.setVariants(id: itemId, labels: ["Slow", "Fast"])))
    let ladder = try XCTUnwrap(try bridge.view().items.first?.variants)
    let slowId = try XCTUnwrap(ladder.first { $0.label == "Slow" }?.id)
    let fastId = try XCTUnwrap(ladder.first { $0.label == "Fast" }?.id)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try XCTUnwrap(try bridge.view().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: slowId)))
    _ = try bridge.update(.session(.startSession(now: "2026-09-04T09:00:00Z")))
    _ = try bridge.update(
      .session(
        .nextItem(
          now: "2026-09-04T09:05:00Z", nextItemStartedAt: "2026-09-04T09:05:00Z", reading: .silent))
    )
    let playId = try XCTUnwrap(try bridge.view().summary?.entries.first?.plays.last?.id)
    _ = try bridge.update(
      .session(.updateEntryScore(entryId: entryId, playId: playId, score: 8)))
    try acknowledgeSave(
      bridge, try bridge.update(.session(.saveSession(now: "2026-09-04T09:06:00Z"))))

    let view = try bridge.view()
    XCTAssertNil(view.error, "the score, attribution and save must all decode cleanly (#846)")
    let item = try XCTUnwrap(view.items.first { $0.id == itemId })
    let slow = try XCTUnwrap(item.variants.first { $0.id == slowId })
    let fast = try XCTUnwrap(item.variants.first { $0.id == fastId })

    XCTAssertEqual(slow.latestScore, 8, "the score attributed to Slow must land on Slow, not Fast")
    let historyEntry = try XCTUnwrap(slow.scoreHistory.first)
    XCTAssertEqual(historyEntry.score, 8)
    XCTAssertFalse(historyEntry.sessionId.isEmpty)
    XCTAssertNotNil(SessionClock.parseRFC3339(historyEntry.sessionDate))
    XCTAssertTrue(slow.isSolid, "8 of 10 reaches SOLID_SCORE_MIN")

    XCTAssertNil(fast.latestScore, "Fast was never practised")
    XCTAssertTrue(fast.scoreHistory.isEmpty)
    XCTAssertFalse(fast.isSolid)
  }
}
