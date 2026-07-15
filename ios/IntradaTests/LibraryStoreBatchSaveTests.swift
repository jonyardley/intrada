import SharedTypes
import Testing

@testable import Intrada

struct LibraryStoreBatchSaveTests {
  private func makeStore() throws -> LibraryStore { try LibraryStore.inMemory() }

  private func item(
    _ id: String, title: String = "Etude", kind: ItemKind = .exercise,
    linkedExerciseIds: [String] = [], createdAt: String = "2026-01-01T00:00:00Z"
  ) -> Item {
    Item(
      id: id, title: title, kind: kind, composer: nil, key: nil, modality: nil,
      tempo: nil, notes: nil, tags: [], linkedExerciseIds: linkedExerciseIds,
      createdAt: createdAt, updatedAt: createdAt, priority: false)
  }

  @Test func batchSavePersistsEveryItem() throws {
    let store = try makeStore()

    try store.save([
      item("ex-melody", title: "Learn the melody"),
      item("ex-shells", title: "Chord shells"),
      item(
        "piece-1", title: "Strasbourg / St. Denis", kind: .piece,
        linkedExerciseIds: ["ex-melody", "ex-shells"]),
    ])

    let loaded = try store.loadItems()
    #expect(loaded.count == 3)
    let piece = try #require(loaded.first { $0.id == "piece-1" })
    #expect(piece.linkedExerciseIds == ["ex-melody", "ex-shells"])
  }

  @Test func batchSaveUpsertsExistingRows() throws {
    let store = try makeStore()
    try store.save(item("ex-1", title: "Before"))

    try store.save([
      item("ex-1", title: "After"),
      item("ex-2", title: "New"),
    ])

    let loaded = try store.loadItems()
    #expect(loaded.count == 2)
    #expect(try #require(loaded.first { $0.id == "ex-1" }).title == "After")
  }

  @Test func emptyBatchIsANoOp() throws {
    let store = try makeStore()
    try store.save([Item]())
    #expect(try store.loadItems().isEmpty)
  }
}
