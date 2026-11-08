import SharedTypes
import Testing

@testable import Intrada

/// What the item sheet sends, run through the real core (#2249).
@MainActor
struct SessionBuilderPlanTests {
  /// A twelve-minute piece in the builder.
  private func plannedPiece() throws -> (RowsBridge, String) {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Clair de Lune", kind: .piece, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let itemId = try #require(try bridge.rendered().items.first?.id)
    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let entryId = try #require(try bridge.rendered().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.setEntryDuration(entryId: entryId, durationSecs: 720)))
    return (bridge, entryId)
  }

  @Test("choosing a key sets the entry's planned key")
  func choosingAKeySetsThePlannedKey() throws {
    let (bridge, entryId) = try plannedPiece()
    let pieceId = try #require(try bridge.rendered().items.first?.id)
    let d = Key(letter: .d, accidental: .natural, mode: .major)
    _ = try bridge.update(.item(.updateKeys(id: pieceId, keys: [d])))
    let choices = try #require(
      try bridge.rendered().buildingSetlist?.entryKeys.first { $0.entryId == entryId }?.keys)
    let choice = try #require(choices.first { $0.key == d })

    _ = try bridge.update(.session(.setEntryKey(entryId: entryId, key: choice.key)))

    let entry = try #require(try bridge.rendered().buildingSetlist?.entries.first)
    #expect(try bridge.rendered().error == nil)
    #expect(entry.plannedKey == d)
  }
}
