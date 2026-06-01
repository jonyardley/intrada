import SharedTypes
import XCTest

@testable import Intrada

/// Exercises the GRDB-backed local store against an in-memory database: the
/// row↔Item codec round-trips, upsert updates in place, and delete tombstones
/// (hides the row from loads) without dropping it.
final class LibraryStoreTests: XCTestCase {
  private func makeStore() throws -> LibraryStore { try LibraryStore.inMemory() }

  private func item(
    _ id: String, title: String = "Etude", kind: ItemKind = .piece,
    createdAt: String = "2026-01-01T00:00:00Z"
  ) -> Item {
    Item(
      id: id, title: title, kind: kind, composer: "Chopin", key: "C", modality: .major,
      tempo: Tempo(marking: "Allegro", bpm: 132), notes: "evenness",
      tags: ["scale", "warmup"], createdAt: createdAt, updatedAt: createdAt, priority: true)
  }

  func testSaveThenLoadRoundTrips() throws {
    let store = try makeStore()
    try store.save(item("p1"))
    let loaded = try store.loadItems()
    XCTAssertEqual(loaded.count, 1)
    let got = try XCTUnwrap(loaded.first)
    XCTAssertEqual(got.id, "p1")
    XCTAssertEqual(got.kind, .piece)
    XCTAssertEqual(got.key, "C")
    XCTAssertEqual(got.modality, .major)
    XCTAssertEqual(got.tempo, Tempo(marking: "Allegro", bpm: 132))
    XCTAssertEqual(got.tags, ["scale", "warmup"])
    XCTAssertTrue(got.priority)
  }

  func testExerciseKindAndNilTempoRoundTrip() throws {
    let store = try makeStore()
    var ex = item("e1", kind: .exercise)
    ex.tempo = nil
    ex.tags = []
    try store.save(ex)
    let got = try XCTUnwrap(try store.loadItems().first)
    XCTAssertEqual(got.kind, .exercise)
    XCTAssertNil(got.tempo)
    XCTAssertEqual(got.tags, [])
  }

  func testUpsertUpdatesInPlace() throws {
    let store = try makeStore()
    try store.save(item("p1", title: "Before"))
    try store.save(item("p1", title: "After"))
    let loaded = try store.loadItems()
    XCTAssertEqual(loaded.count, 1)
    XCTAssertEqual(loaded.first?.title, "After")
  }

  func testDeleteTombstonesAndHidesFromLoad() throws {
    let store = try makeStore()
    try store.save(item("p1"))
    try store.delete(id: "p1")
    XCTAssertTrue(try store.loadItems().isEmpty)
  }

  func testSaveRevivesADeletedRow() throws {
    let store = try makeStore()
    try store.save(item("p1"))
    try store.delete(id: "p1")
    try store.save(item("p1", title: "Revived"))
    let loaded = try store.loadItems()
    XCTAssertEqual(loaded.count, 1)
    XCTAssertEqual(loaded.first?.title, "Revived")
  }

  func testLoadOrdersNewestFirst() throws {
    let store = try makeStore()
    try store.save(item("old", title: "Old", createdAt: "2026-01-01T00:00:00Z"))
    try store.save(item("new", title: "New", createdAt: "2026-02-01T00:00:00Z"))
    XCTAssertEqual(try store.loadItems().map(\.id), ["new", "old"])
  }

  /// Offline-first invariant #2 (CLAUDE.md): persisted tables carry the sync columns.
  func testSchemaHasSyncColumns() throws {
    let columns = try makeStore().columnNames(ofTable: "item")
    XCTAssertTrue(
      columns.contains("updated_at"), "item table must carry updated_at; has \(columns)")
    XCTAssertTrue(
      columns.contains("deleted_at"), "item table must carry deleted_at; has \(columns)")
  }
}
