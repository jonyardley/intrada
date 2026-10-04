import SharedTypes
import Testing

@testable import Intrada

/// What the item sheet's sections and focus cards send, run through the real
/// core (#2303, #2315).
@MainActor
struct SessionBuilderPlanTests {
  /// A twelve-minute piece split into A1, B and A2, with a "Dotted" variation.
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
    _ = try bridge.update(
      .session(
        .setSegments(entryId: entryId, segments: EntrySegmentsCard.evenly(sections.map(\.id)))))
    return (bridge, entryId, variation)
  }

  private func segments(_ bridge: RowsBridge) throws -> [SegmentView] {
    try #require(try bridge.rendered().buildingSetlist?.entries.first?.record.segments)
  }

  private func planned(_ views: [SegmentView]) -> [Segment] {
    views.map { Segment(sectionId: $0.sectionId, plannedSecs: $0.plannedSecs) }
  }

  @Test("a new split starts even")
  func aNewSplitStartsEven() throws {
    let (bridge, _, _) = try splitPiece()
    #expect(try segments(bridge).map(\.plannedSecs) == [240, 240, 240])
  }

  @Test(
    "a stepper moves a minute from the next section, or the one before for the last",
    arguments: [
      (0, 60, [300, 180, 240]),
      (1, -60, [240, 180, 300]),
      (2, 60, [240, 180, 300]),
    ])
  func aStepperMovesAMinuteAndKeepsTheTotal(index: Int, delta: Int, expected: [UInt32]) throws {
    let (bridge, entryId, _) = try splitPiece()
    let next = try #require(
      EntrySegmentsCard.stepped(
        planned(try segments(bridge)), at: index, by: delta, minimum: 60))
    _ = try bridge.update(.session(.setSegments(entryId: entryId, segments: next)))
    #expect(try bridge.rendered().error == nil)
    #expect(try segments(bridge).map(\.plannedSecs) == expected)
  }

  @Test("a stepper never takes a section under the minimum")
  func aStepperStopsAtTheMinimum() {
    let segments = [
      Segment(sectionId: "a", plannedSecs: 60), Segment(sectionId: "b", plannedSecs: 600),
    ]
    #expect(EntrySegmentsCard.stepped(segments, at: 0, by: -60, minimum: 60) == nil)
    #expect(EntrySegmentsCard.stepped([segments[0]], at: 0, by: 60, minimum: 60) == nil)
  }

  @Test("choosing a variation keeps the split as it was")
  func aVariationKeepsTheSplit() throws {
    let (bridge, entryId, variation) = try splitPiece()
    let moved = try #require(
      EntrySegmentsCard.stepped(planned(try segments(bridge)), at: 0, by: 120, minimum: 60))
    _ = try bridge.update(.session(.setSegments(entryId: entryId, segments: moved)))

    for event in EntrySettingsSheet.planEvents(
      entryId: entryId, segments: try segments(bridge), variationId: variation)
    {
      _ = try bridge.update(event)
    }

    let entry = try #require(try bridge.rendered().buildingSetlist?.entries.first)
    #expect(try bridge.rendered().error == nil)
    #expect(entry.plannedVariationIds == [variation])
    #expect(entry.record.segments.map(\.plannedSecs) == [360, 120, 240])
  }

  @Test("a tempo focus starts from the section's own target, else the click's default")
  func aTempoStartsFromTheSection() {
    let limits = LimitsView.preview
    let section = SectionView(
      id: "s", name: "A1", kind: .form, targetBpm: 84, firstBar: nil, lastBar: nil, label: "A1",
      barsCaption: nil)
    #expect(EntryFocusCard.startingTarget(for: .tempo, section: section, limits: limits) == 84)
    #expect(
      EntryFocusCard.startingTarget(for: .tempo, section: nil, limits: limits)
        == limits.clickTempoDefault)
    #expect(
      EntryFocusCard.startingTarget(for: .fromMemory, section: section, limits: limits) == nil)
    #expect(EntryFocusCard.targetRange(for: .evenness, limits: limits) == nil)
  }
}
