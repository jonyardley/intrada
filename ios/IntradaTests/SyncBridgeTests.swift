import IntradaCoreFFI
import SharedTypes
import Testing

@testable import Intrada

/// The sync shapes through the real core, never a stub (#846,
/// `specs/icloud-sync.md`).
@MainActor
struct SyncBridgeTests {
  private let pieceJSON =
    #"{"id":"piece","title":"Etude","kind":"piece","composer":null,"key":null,"tempo":null,"notes":null,"tags":[],"created_at":"2026-09-21T14:13:20Z","updated_at":"2026-09-21T14:13:20Z"}"#

  private func record(schemaVersion: UInt32 = 1) -> SyncRecord {
    SyncRecord(
      kind: .item, id: "piece", schemaVersion: schemaVersion, changedAt: "2026-09-21T14:13:20Z",
      deletedAt: nil, body: Array(pieceJSON.utf8))
  }

  @Test func anArrivedPieceIsComparedWithTheStoreThenWritten() throws {
    let bridge = LiveBridge()
    let arrived = try bridge.update(.sync(.recordsArrived([record()])))
    let keys = arrived.compactMap { request -> (UInt32, [RecordKey])? in
      if case .persistence(.loadRecords(let keys)) = request.effect { return (request.id, keys) }
      return nil
    }
    let load = try #require(keys.first, "an arrived record loads the stored copy")
    #expect(load.1 == [RecordKey(kind: .item, id: "piece")])

    let merged = try bridge.resolve(
      load.0,
      persistenceOutput: .records(
        StoredRecords(items: [], variations: [], sessions: [], unreadable: [])))
    let written = merged.compactMap { request -> StoredRecords? in
      if case .persistence(.applyMerged(let records)) = request.effect { return records }
      return nil
    }
    let items: [StoredItem] = written.first?.items ?? []
    #expect(items.map(\.item.title) == ["Etude"])
    #expect(items.map(\.deletedAt) == [nil])
  }

  @Test func anArrivalForARowThisAppCannotReadIsParkedNotWritten() throws {
    let bridge = LiveBridge()
    let arrived = record()
    let requests = try bridge.update(.sync(.recordsArrived([arrived])))
    let loadId = try #require(
      requests.compactMap { request -> UInt32? in
        if case .persistence(.loadRecords) = request.effect { return request.id }
        return nil
      }.first)

    let answered = try bridge.resolve(
      loadId,
      persistenceOutput: .records(
        StoredRecords(
          items: [], variations: [], sessions: [],
          unreadable: [RecordKey(kind: .item, id: "piece")])))
    let parked = answered.compactMap { request -> [SyncRecord]? in
      if case .sync(.park(let records)) = request.effect { return records }
      return nil
    }
    #expect(parked == [[arrived]])
    #expect(
      !answered.contains { request in
        if case .persistence(.applyMerged) = request.effect { return true }
        return false
      })
  }

  @Test func aRecordFromANewerAppIsParkedAndNeverResolved() throws {
    let bridge = LiveBridge()
    let tooNew = record(schemaVersion: 99)
    let requests = try bridge.update(.sync(.recordsArrived([tooNew])))
    let parked = requests.compactMap { request -> (UInt32, [SyncRecord])? in
      if case .sync(.park(let records)) = request.effect { return (request.id, records) }
      return nil
    }
    let park = try #require(parked.first)
    #expect(park.1 == [tooNew])
    #expect(
      requests.contains { request in
        if case .sync(.settled) = request.effect { return true }
        return false
      })
    #expect(throws: (any Error).self) { try bridge.resolveEmpty(park.0) }
  }

  @Test func theAccountAndFirstUploadEventsCrossTheWire() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.sync(.accountChanged))
    let requests = try bridge.update(.sync(.uploadEverything))
    #expect(
      requests.contains { request in
        if case .persistence(.loadAllRecords) = request.effect { return true }
        return false
      })
  }
}
