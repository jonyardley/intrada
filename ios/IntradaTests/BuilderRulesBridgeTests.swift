import SharedTypes
import Testing

@testable import Intrada

/// The builder rules the core owns cross the real bridge (#846, #2393): the
/// variation and minute events from Swift, the tap and target fields back.
@MainActor
struct BuilderRulesBridgeTests {

  /// A piece with sections A (target 72), B and C and a Dotted variation, in
  /// a twelve minute entry split evenly into the three.
  private func twelveSplitThree() throws -> (RowsBridge, String, [String], String) {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Nocturne", kind: .piece, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let item = try #require(try bridge.rendered().items.first)
    _ = try bridge.update(
      .item(.updateItemVariations(id: item.id, variationIds: [], newLabels: ["Dotted"])))
    try bridge.addSections(
      [("A", "72"), ("B", ""), ("C", "")].map {
        SectionEdit(id: nil, name: $0.0, bars: .blank, kind: .form, targetBpm: $0.1)
      }, to: item.id)
    let sections = try #require(try bridge.rendered().items.first?.sections).map(\.id)
    let variationId = try #require(try bridge.rendered().items.first?.variations.first?.id)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: item.id)))
    let entryId = try #require(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.setEntryDuration(entryId: entryId, durationSecs: 720)))
    _ = try bridge.update(
      .session(
        .setSegments(
          entryId: entryId, segments: sections.map { Segment(sectionId: $0, plannedSecs: 0) })))
    #expect(try bridge.rendered().error == nil)
    return (bridge, entryId, sections, variationId)
  }

  private func entry(_ bridge: RowsBridge) throws -> SetlistEntryView {
    try #require(try bridge.rendered().buildingSetlist?.entries.first)
  }

  @Test func aVariationKeepsTheSplitAsItWas() throws {
    let (bridge, entryId, sections, variationId) = try twelveSplitThree()

    _ = try bridge.update(
      .session(.setEntryVariations(entryId: entryId, variationIds: [variationId])))

    let planned = try entry(bridge)
    #expect(planned.plannedVariationIds == [variationId])
    #expect(planned.record.segments.map(\.sectionId) == sections)
    #expect(planned.record.segments.map(\.plannedSecs) == [240, 240, 240])
  }

  @Test func minuteStepsTakeFromTheNextUntilItWouldEmpty() throws {
    let (bridge, entryId, sections, _) = try twelveSplitThree()
    for _ in 0..<3 {
      _ = try bridge.update(
        .session(.stepSegment(entryId: entryId, sectionId: sections[0], minutes: 1)))
      #expect(try bridge.rendered().error == nil)
    }

    let segments: [SegmentView] = try entry(bridge).record.segments
    #expect(segments.map(\.plannedSecs) == [420, 60, 240])
    #expect(segments.map(\.canAddMinute) == [false, true, false])
    #expect(segments.map(\.canTakeMinute) == [true, false, true])
  }

  @Test func aRefusedStepLeavesTheSplitAsItWas() throws {
    let (bridge, entryId, sections, _) = try twelveSplitThree()
    for _ in 0..<4 {
      _ = try bridge.update(
        .session(.stepSegment(entryId: entryId, sectionId: sections[0], minutes: 1)))
    }

    #expect(try bridge.rendered().error != nil)
    #expect(try entry(bridge).record.segments.map(\.plannedSecs) == [420, 60, 240])
  }

  @Test func aFocusStartsFromTheSectionsTargetAndShowsItsCaption() throws {
    let (bridge, entryId, sections, _) = try twelveSplitThree()
    let choices: [FocusChoiceView] = try #require(try bridge.rendered().buildingSetlist)
      .focusChoices
    #expect(
      choices.map(\.target) == [
        FocusTargetView(min: 40, max: 208, step: 2), FocusTargetView(min: 1, max: 100, step: 1),
        nil, nil,
      ])

    _ = try bridge.update(
      .session(
        .setFocus(
          entryId: entryId,
          focus: IntentionFocus(kind: .tempo, sectionId: sections[0], target: nil))))

    let focus: FocusView = try #require(try entry(bridge).record.focus)
    #expect(focus.focus.target == 72)
    #expect(focus.targetCaption == "\u{2669} = 72")
  }

  /// Tuned to 7, 1, 4 by three steps on A (#2398).
  private func tuned() throws -> (RowsBridge, String, [String]) {
    let (bridge, entryId, sections, _) = try twelveSplitThree()
    for _ in 0..<3 {
      _ = try bridge.update(
        .session(.stepSegment(entryId: entryId, sectionId: sections[0], minutes: 1)))
    }
    #expect(try entry(bridge).record.segments.map(\.plannedSecs) == [420, 60, 240])
    return (bridge, entryId, sections)
  }

  @Test func removingASectionGivesItsMinutesToTheNext() throws {
    let (bridge, entryId, sections) = try tuned()

    _ = try bridge.update(.session(.removeSegment(entryId: entryId, sectionId: sections[1])))

    let segments: [SegmentView] = try entry(bridge).record.segments
    #expect(try bridge.rendered().error == nil)
    #expect(segments.map(\.sectionId) == [sections[0], sections[2]])
    #expect(segments.map(\.plannedSecs) == [420, 300])
  }

  @Test func addingASectionTakesItsShareFromTheLargest() throws {
    let (bridge, entryId, sections) = try tuned()
    _ = try bridge.update(.session(.removeSegment(entryId: entryId, sectionId: sections[2])))

    _ = try bridge.update(.session(.addSegment(entryId: entryId, sectionId: sections[2])))

    let segments: [SegmentView] = try entry(bridge).record.segments
    #expect(try bridge.rendered().error == nil)
    #expect(segments.map(\.sectionId) == sections)
    #expect(segments.map(\.plannedSecs) == [180, 300, 240])
    #expect(try entry(bridge).record.canAddSection)
  }
}
