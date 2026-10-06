import Foundation
import IntradaCoreFFI
import SharedTypes
import Testing
import os

@testable import Intrada

@MainActor
struct SharedItemStoreTests {
  private final class Answering: StoreFfiProtocol, @unchecked Sendable {
    let answer: StoreAnswer
    init(_ answer: StoreAnswer) { self.answer = answer }
    func handle(operation: Data) throws -> StoreAnswer { answer }
  }

  private static func bytes(_ output: PersistenceOutput) throws -> Data {
    Data(try output.bincodeSerialize())
  }

  /// One unreadable row never empties History, and nothing is dropped silently (#2234, #949).
  @Test func everyValueTheStoreCouldNotReadIsReported() throws {
    let reports = OSAllocatedUnfairLock<[String]>(initialState: [])
    reportObserver.withLock {
      $0 = { error, context in
        guard context == SharedItemStore.decodeContext else { return }
        reports.withLock { $0.append(String(describing: error)) }
      }
    }
    defer { reportObserver.withLock { $0 = nil } }
    let store = SharedItemStore(
      Answering(
        StoreAnswer(
          output: try Self.bytes(.sessions([])),
          unreadable: ["session bad: refused-2234", "unknown CompletionStatus"], error: nil)))

    #expect(try store.handle(.loadSessions) == .sessions([]))
    #expect(
      reports.withLock { $0 } == ["session bad: refused-2234", "unknown CompletionStatus"])
  }

  /// A failed write is never a silent success (#816).
  @Test func anOperationTheStoreCouldNotCompleteThrows() throws {
    let store = SharedItemStore(
      Answering(
        StoreAnswer(output: try Self.bytes(.failed), unreadable: [], error: "disk full")))

    #expect(throws: SharedItemStore.StoreFailure.self) { try store.handle(.loadItems) }
  }

  @Test func aSavedItemReadsBackFromAFileOpenedAgain() throws {
    let path = FileManager.default.temporaryDirectory
      .appendingPathComponent("shared-store-\(UUID().uuidString).sqlite").path
    defer { try? FileManager.default.removeItem(atPath: path) }
    let item = Item(
      id: "p1", title: "Nocturne", kind: .piece, composer: "Chopin", key: nil, tempo: nil,
      notes: nil, tags: ["recital"], createdAt: "2026-09-01T10:00:00Z",
      updatedAt: "2026-09-01T10:00:00Z", priority: false, chordChart: nil, photoId: nil,
      metre: nil, sections: [], variationIds: [], keys: [], exerciseLinks: [])
    do {
      #expect(try SharedItemStore.at(path: path).handle(.saveItem(item)) == .ack)
    }

    #expect(try SharedItemStore.at(path: path).handle(.loadItems) == .items([item]))
  }
}
