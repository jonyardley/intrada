import SharedTypes
import Testing

@testable import Intrada

/// The builder rules the core owns cross the real bridge (#846, #2393): the
/// variation events from Swift, the tags and captions back.
@MainActor
struct BuilderRulesBridgeTests {

  /// A piece marked at 72 with a Dotted variation, in a twelve minute entry.
  private func twelveMinutePiece() throws -> (RowsBridge, String, String) {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Nocturne", kind: .piece, composer: nil, key: nil,
            tempo: TempoInput(marking: nil, bpm: "72"), notes: nil, tags: [], photoId: nil,
            variationLabels: []))))
    let item = try #require(try bridge.rendered().items.first)
    _ = try bridge.update(
      .item(.updateItemVariations(id: item.id, variationIds: [], newLabels: ["Dotted"])))
    let variationId = try #require(try bridge.rendered().items.first?.variations.first?.id)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: item.id)))
    let entryId = try #require(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.setEntryDuration(entryId: entryId, durationSecs: 720)))
    #expect(try bridge.rendered().error == nil)
    return (bridge, entryId, variationId)
  }

  private func entry(_ bridge: RowsBridge) throws -> SetlistEntryView {
    try #require(try bridge.rendered().buildingSetlist?.entries.first)
  }

  /// The builder's lines arrive ready, so neither phone writes its own (#2467).
  @Test func theBuildersSummaryCaptionAndTagsArriveReady() throws {
    let (bridge, entryId, variationId) = try twelveMinutePiece()
    _ = try bridge.update(
      .session(.setEntryVariations(entryId: entryId, variationIds: [variationId])))

    let building = try #require(try bridge.rendered().buildingSetlist)
    let tags: [String] = try entry(bridge).planTags
    #expect(tags == ["Dotted"])
    let block: SetlistBlockView = try #require(building.blocks.first)
    #expect(block.caption == "Piece \u{B7} 12 min")
    #expect(block.plannedDisplay == "12 min")
    #expect(block.relatedLabel == "piece only")
    #expect(block.durationDisplay == "12 min")
    #expect(building.summary == "12 min \u{B7} 1 item")
    #expect(building.startTitle == "Start session \u{B7} 12 min")
  }

  @Test func anAimIsKeptTrimmedAndABlankOneIsNone() throws {
    let (bridge, entryId, _) = try twelveMinutePiece()

    _ = try bridge.update(
      .session(.setEntryIntention(entryId: entryId, intention: "  Even quavers \n")))
    #expect(try entry(bridge).intention == "Even quavers")

    _ = try bridge.update(.session(.setEntryIntention(entryId: entryId, intention: "   ")))
    #expect(try entry(bridge).intention == nil)
    #expect(try bridge.rendered().error == nil)
  }

  @Test func thePlannedTimeStepperReadsWholeMinutes() throws {
    let limits = try RowsBridge().rendered().limits
    #expect(limits.plannedDurationMinMins == 1)
    #expect(limits.plannedDurationMaxMins == 60)
    #expect(limits.plannedDurationDefaultMins == 6)
  }
}
