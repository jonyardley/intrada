import Foundation
import SharedTypes
import Testing

@testable import Intrada

@MainActor
struct StoreDiskQueueTests {

  @Test func diskWorkRunsOffTheMainThread() async throws {
    let disk = ThreadRecordingStore()
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 1, effect: .persistence(.loadItems))] }
    let store = Store(bridge: bridge, store: disk)

    store.send(.setQuery(nil))
    await store.settle()

    #expect(disk.ranOnMain == [false])
    #expect(bridge.persistenceResolved.map(\.id) == [1])
  }

  @Test func aLoadSentAfterASaveWaitsForTheSave() async throws {
    let disk = SlowSaveStore()
    let bridge = FakeBridge()
    var next: UInt32 = 0
    bridge.updateHandler = { event in
      next += 1
      if case .setQuery = event {
        return [Request(id: next, effect: .persistence(.saveItem(LibraryItemFixture.record())))]
      }
      return [Request(id: next, effect: .persistence(.loadItems))]
    }
    let store = Store(bridge: bridge, store: disk)

    store.send(.setQuery(nil))
    store.send(.setUtcOffset(minutes: 0))
    await store.settle()

    #expect(disk.calls == ["save", "load"])
    #expect(bridge.persistenceResolved.map(\.id) == [1, 2])
  }

  @Test func jobsInOneBatchRunInTheOrderTheCoreAsked() async throws {
    let disk = SlowSaveStore()
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [
        Request(id: 1, effect: .persistence(.saveItem(LibraryItemFixture.record()))),
        Request(id: 2, effect: .persistence(.loadItems)),
        Request(id: 3, effect: .persistence(.loadSessions)),
      ]
    }
    let store = Store(bridge: bridge, store: disk)

    store.send(.setQuery(nil))
    await store.settle()

    #expect(disk.calls == ["save", "load", "sessions"])
    #expect(bridge.persistenceResolved.map(\.id) == [1, 2, 3])
  }

  @Test func aJobTheCoreChainsFromAResolveIsAwaitedToo() async throws {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 1, effect: .persistence(.loadItems))] }
    bridge.resolveHandler = { id in
      id == 1 ? [Request(id: 2, effect: .persistence(.loadSessions))] : []
    }
    let store = Store(bridge: bridge)

    store.send(.setQuery(nil))
    await store.settle()

    #expect(bridge.persistenceResolved.map(\.id) == [1, 2])
  }

  @Test func aResultLandingAfterAHaltNeverReachesTheCore() async throws {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 1, effect: .persistence(.loadItems))] }
    let store = Store(bridge: bridge)

    store.send(.setQuery(nil))
    bridge.throwOnUpdate = CorePanic(underlying: StoreDiskQueueError())
    store.send(.setQuery(nil))
    await store.settle()

    #expect(store.halted)
    #expect(bridge.persistenceResolved.isEmpty)
  }
}

private struct StoreDiskQueueError: Error {}

private final class ThreadRecordingStore: ItemStore, @unchecked Sendable {
  private let lock = NSLock()
  private var recorded: [Bool] = []
  var ranOnMain: [Bool] { lock.withLock { recorded } }

  private func record() { lock.withLock { recorded.append(Thread.isMainThread) } }

  func loadItems() throws -> [Item] {
    record()
    return []
  }
  func save(_ item: Item) throws { record() }
  func save(_ items: [Item]) throws { record() }
  func delete(id: String, deletedAt: String) throws { record() }
  func loadSessions() throws -> [PracticeSession] {
    record()
    return []
  }
  func saveSession(_ session: PracticeSession) throws { record() }
}

/// A save slow enough that a load not made to wait would overtake it.
private final class SlowSaveStore: ItemStore, @unchecked Sendable {
  private let lock = NSLock()
  private var recorded: [String] = []
  var calls: [String] { lock.withLock { recorded } }

  private func record(_ call: String) { lock.withLock { recorded.append(call) } }

  func loadItems() throws -> [Item] {
    record("load")
    return []
  }
  func save(_ item: Item) throws {
    Thread.sleep(forTimeInterval: 0.05)
    record("save")
  }
  func save(_ items: [Item]) throws { record("save") }
  func delete(id: String, deletedAt: String) throws { record("delete") }
  func loadSessions() throws -> [PracticeSession] {
    record("sessions")
    return []
  }
  func saveSession(_ session: PracticeSession) throws { record("saveSession") }
}
