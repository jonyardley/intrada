import SharedTypes
import Testing

@testable import Intrada

/// `setQuery` filtering, moved from `LibrarySearchUITests` (#1825): pure store state.
struct LibraryQueryFilterTests {
  private func seededBridge() throws -> RowsBridge {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Clair de Lune", kind: .piece, composer: "Debussy", key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Hanon No. 1", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    return bridge
  }

  @Test("a text query narrows the list to the matching item")
  func queryFiltersToTheMatchingTitle() throws {
    let bridge = try seededBridge()

    _ = try bridge.update(
      .setQuery(ListQuery(text: "hanon", itemType: nil, key: nil, tags: [], priorityOnly: false)))

    #expect(try bridge.rendered().visibleItems.map(\.title) == ["Hanon No. 1"])
  }

  @Test("clearing the query restores the full list")
  func clearingTheQueryRestoresTheFullList() throws {
    let bridge = try seededBridge()
    _ = try bridge.update(
      .setQuery(ListQuery(text: "hanon", itemType: nil, key: nil, tags: [], priorityOnly: false)))

    _ = try bridge.update(.setQuery(nil))

    #expect(
      Swift.Set(try bridge.rendered().visibleItems.map(\.title)) == [
        "Clair de Lune", "Hanon No. 1",
      ])
  }

  @Test("the star filter crosses the wire and leaves only priorities")
  func theStarFilterLeavesOnlyPriorities() throws {
    let bridge = try seededBridge()
    let hanon = try #require(try bridge.rendered().items.first { $0.title == "Hanon No. 1" })
    _ = try bridge.update(
      .item(
        .update(
          id: hanon.id,
          input: UpdateItem(
            title: hanon.title, kind: hanon.itemType, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: nil, priority: true))))

    _ = try bridge.update(
      .setQuery(ListQuery(text: nil, itemType: nil, key: nil, tags: [], priorityOnly: true)))

    #expect(try bridge.rendered().visibleItems.map(\.title) == ["Hanon No. 1"])
    #expect(try bridge.rendered().activeQuery?.priorityOnly == true)
  }

  @Test("a query leaves the whole library in items for the pickers")
  func aQueryLeavesTheWholeLibraryInItems() throws {
    let bridge = try seededBridge()
    _ = try bridge.update(
      .setQuery(ListQuery(text: "hanon", itemType: nil, key: nil, tags: [], priorityOnly: false)))

    #expect(try bridge.rendered().items.count == 2)
  }

  @Test("the recently practised ids cross the wire and name library rows")
  func recentlyPractisedIdsCrossTheWire() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.loadSampleData)

    let view = try bridge.rendered()
    #expect(!view.recentlyPractisedIds.isEmpty)
    #expect(view.recentlyPractisedItems.map(\.id) == view.recentlyPractisedIds)
  }
}
