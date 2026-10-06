import SharedTypes
import Testing

@testable import Intrada

/// A drop in the builder's list, and the block it reads, through `LiveBridge` (#2231).
struct MoveRowBridgeTests {

  private struct Built {
    let block: SetlistBlockView
    let standalone: SetlistEntryView
    let ids: (piece: String, exercise: String)
  }

  /// A piece with one linked exercise, then a standalone piece.
  private func building(_ bridge: RowsBridge) throws -> Built {
    _ = try bridge.update(.startApp)
    for (title, kind) in [
      ("Clair de lune", ItemKind.piece), ("Arpeggios", .exercise), ("Gymnopédie No. 1", .piece),
    ] {
      _ = try bridge.update(
        .item(
          .add(
            CreateItem(
              title: title, kind: kind, composer: nil, key: nil, tempo: nil, notes: nil,
              tags: [], photoId: nil, variationLabels: []))))
    }
    let id = { (title: String) in try #require(bridge.items.first { $0.title == title }?.id) }
    let (piece, exercise, other) = (
      try id("Clair de lune"), try id("Arpeggios"), try id("Gymnopédie No. 1")
    )
    try bridge.link(exercise: exercise, to: piece)
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: piece)))
    _ = try bridge.update(.session(.addToSetlist(itemId: other)))
    let blocks = try #require(try bridge.rendered().buildingSetlist?.blocks)
    try #require(blocks.count == 2)
    let standalone = try #require(blocks[1].entries.first)
    return Built(block: blocks[0], standalone: standalone, ids: (piece, exercise))
  }

  private func order(_ bridge: RowsBridge) throws -> [String] {
    try bridge.rendered().buildingSetlist?.entries.map(\.itemId) ?? []
  }

  @Test func aBlockNamesItsPieceAndItsRelatedExercise() throws {
    let built = try building(RowsBridge())

    #expect(built.block.piece?.itemId == built.ids.piece)
    #expect(built.block.related.map(\.itemId) == [built.ids.exercise])
    #expect(built.block.piece?.removable == false)
    #expect(built.block.related.first?.removable == true)
    #expect(built.standalone.removable == true)
  }

  @Test func aDroppedRowLandsWhereItWasDropped() throws {
    let bridge = RowsBridge()
    let built = try building(bridge)
    let groupId = try #require(built.block.groupId)
    let exercise = try #require(built.block.related.first?.id)
    let standalone = built.standalone

    let header: BuilderRowRef = .header(groupId: groupId)
    _ = try bridge.update(
      .session(.moveRow(moved: header, before: nil, after: .entry(entryId: standalone.id))))
    #expect(try order(bridge) == [standalone.itemId, built.ids.exercise, built.ids.piece])

    _ = try bridge.update(
      .session(
        .moveRow(
          moved: .entry(entryId: standalone.id), before: .addRelated(groupId: groupId),
          after: .entry(entryId: exercise))))
    #expect(try order(bridge) == [built.ids.exercise, built.ids.piece, standalone.itemId])
  }
}
