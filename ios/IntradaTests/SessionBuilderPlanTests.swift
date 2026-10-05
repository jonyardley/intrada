import SharedTypes
import Testing

@testable import Intrada

/// What the item sheet's sections card sends, run through the real core
/// (#2315, #2398).
@MainActor
struct SessionBuilderPlanTests {
  /// A twelve-minute piece split into A1, B and A2 by three taps on "Add a
  /// section", with a "Dotted" variation.
  private func splitPiece() throws -> (RowsBridge, String, String) {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Clair de Lune", kind: .piece, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: ["Dotted"]))))
    let itemId = try #require(try bridge.rendered().items.first?.id)
    _ = try bridge.update(
      .item(
        .updateSections(
          id: itemId,
          sections: ["A1", "B", "A2"].map {
            SectionEdit(id: nil, name: $0, bars: .blank, kind: .form, targetBpm: "")
          })))
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let building = try #require(try bridge.rendered().buildingSetlist)
    let entryId = try #require(building.entries.first?.id)
    let sections = try #require(building.entryVariations.first?.sections)
    let variation = try #require(building.entryVariations.first?.variations.first?.id)
    _ = try bridge.update(.session(.setEntryDuration(entryId: entryId, durationSecs: 720)))
    for section in sections {
      _ = try bridge.update(.session(.addSegment(entryId: entryId, sectionId: section.id)))
    }
    return (bridge, entryId, variation)
  }

  private func segments(_ bridge: RowsBridge) throws -> [SegmentView] {
    try #require(try bridge.rendered().buildingSetlist?.entries.first?.record.segments)
  }

  @Test("a new split starts even")
  func aNewSplitStartsEven() throws {
    let (bridge, _, _) = try splitPiece()
    #expect(try segments(bridge).map(\.plannedSecs) == [240, 240, 240])
  }

  @Test("removing a section from an untuned split shares evenly again")
  func removingFromAnUntunedSplitSharesEvenly() throws {
    let (bridge, entryId, _) = try splitPiece()
    let middle = try segments(bridge)[1].sectionId
    _ = try bridge.update(.session(.removeSegment(entryId: entryId, sectionId: middle)))
    #expect(try segments(bridge).map(\.plannedSecs) == [360, 360])
  }

  @Test("removing a section from a tuned split gives its minutes to the next")
  func removingFromATunedSplitGivesToTheNext() throws {
    let (bridge, entryId, _) = try splitPiece()
    let ids = try segments(bridge).map(\.sectionId)
    _ = try bridge.update(.session(.stepSegment(entryId: entryId, sectionId: ids[0], minutes: 1)))
    _ = try bridge.update(.session(.removeSegment(entryId: entryId, sectionId: ids[1])))
    #expect(try segments(bridge).map(\.plannedSecs) == [300, 420])
  }

  @Test(
    "a stepper moves a minute from the next section, or the one before for the last",
    arguments: [
      (0, Int8(1), [UInt32(300), 180, 240]),
      (1, Int8(-1), [240, 180, 300]),
      (2, Int8(1), [240, 180, 300]),
    ])
  func aStepperMovesAMinuteAndKeepsTheTotal(index: Int, minutes: Int8, expected: [UInt32])
    throws
  {
    let (bridge, entryId, _) = try splitPiece()
    let sectionId = try segments(bridge)[index].sectionId
    _ = try bridge.update(
      .session(.stepSegment(entryId: entryId, sectionId: sectionId, minutes: minutes)))
    #expect(try bridge.rendered().error == nil)
    #expect(try segments(bridge).map(\.plannedSecs) == expected)
  }

  @Test("choosing a variation keeps the split as it was")
  func aVariationKeepsTheSplit() throws {
    let (bridge, entryId, variation) = try splitPiece()
    let first = try segments(bridge)[0].sectionId
    for _ in 0..<2 {
      _ = try bridge.update(.session(.stepSegment(entryId: entryId, sectionId: first, minutes: 1)))
    }

    _ = try bridge.update(
      .session(.setEntryVariations(entryId: entryId, variationIds: [variation])))

    let entry = try #require(try bridge.rendered().buildingSetlist?.entries.first)
    #expect(try bridge.rendered().error == nil)
    #expect(entry.plannedVariationIds == [variation])
    #expect(entry.record.segments.map(\.plannedSecs) == [360, 120, 240])
  }
}
