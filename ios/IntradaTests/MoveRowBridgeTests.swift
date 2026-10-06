import SharedTypes
import Testing

@testable import Intrada

/// A drop in the builder's list through `LiveBridge` (#2231).
struct MoveRowBridgeTests {

  private func building(_ bridge: RowsBridge, titles: [String]) throws -> [SetlistEntryView] {
    _ = try bridge.update(.startApp)
    for title in titles {
      _ = try bridge.update(
        .item(
          .add(
            CreateItem(
              title: title, kind: .piece, composer: nil, key: nil, tempo: nil, notes: nil,
              tags: [], photoId: nil, variationLabels: []))))
    }
    _ = try bridge.update(.session(.startBuilding))
    for item in try bridge.rendered().items {
      _ = try bridge.update(.session(.addToSetlist(itemId: item.id)))
    }
    return try #require(try bridge.rendered().buildingSetlist?.entries)
  }

  @Test func aDroppedRowLandsWhereItWasDropped() throws {
    let bridge = RowsBridge()
    let entries = try building(bridge, titles: ["Clair de lune", "Gymnopédie No. 1"])
    let (first, second) = (entries[0], entries[1])
    let moved: BuilderRowRef = .entry(entryId: first.id)

    _ = try bridge.update(
      .session(.moveRow(moved: moved, before: nil, after: .entry(entryId: second.id))))

    let order = try bridge.rendered().buildingSetlist?.entries.map(\.id)
    #expect(order == [second.id, first.id])
  }

  @Test func aStandalonePieceCanBeRemovedAndAnchorsNoBlock() throws {
    let bridge = RowsBridge()
    _ = try building(bridge, titles: ["Clair de lune"])

    let block = try #require(try bridge.rendered().buildingSetlist?.blocks.first)
    #expect(block.entries.first?.removable == true)
    #expect(block.piece == nil)
    #expect(block.related.isEmpty)
  }
}
