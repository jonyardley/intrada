import SharedTypes
import Testing

@testable import Intrada

/// `setQuery` filtering, moved from `LibrarySearchUITests` (#1825): pure store state.
struct LibraryQueryFilterTests {
  private func seededBridge() throws -> LiveBridge {
    let bridge = LiveBridge()
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

    _ = try bridge.update(.setQuery(ListQuery(text: "hanon", itemType: nil, key: nil, tags: [])))

    #expect(try bridge.view().items.map(\.title) == ["Hanon No. 1"])
  }

  @Test("clearing the query restores the full list")
  func clearingTheQueryRestoresTheFullList() throws {
    let bridge = try seededBridge()
    _ = try bridge.update(.setQuery(ListQuery(text: "hanon", itemType: nil, key: nil, tags: [])))

    _ = try bridge.update(.setQuery(nil))

    #expect(
      Swift.Set(try bridge.view().items.map(\.title)) == ["Clair de Lune", "Hanon No. 1"])
  }

  @Test("the visible ids cross the wire and name the filtered rows")
  func visibleIdsCrossTheWire() throws {
    let bridge = try seededBridge()
    _ = try bridge.update(.setQuery(ListQuery(text: "hanon", itemType: nil, key: nil, tags: [])))

    let view = try bridge.view()
    #expect(view.visibleIds == view.items.map(\.id))
    #expect(view.visibleIds.count == 1)
    #expect(view.allItems.count == 2)
  }

  @Test("the recently practised ids cross the wire and name the recent rows")
  func recentlyPractisedIdsCrossTheWire() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.loadSampleData)

    let view = try bridge.view()
    #expect(!view.recentlyPractisedIds.isEmpty)
    #expect(view.recentlyPractisedIds == view.recentlyPractised.map(\.id))
  }
}
