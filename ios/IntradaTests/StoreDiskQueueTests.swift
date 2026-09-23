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

  @Test func aLoadSentAfterASaveSeesTheSavedRow() async throws {
    let bridge = FakeBridge()
    var next: UInt32 = 0
    let item = StoreEffectLoopTests.sampleItem
    bridge.updateHandler = { event in
      next += 1
      if case .setQuery = event {
        return [Request(id: next, effect: .persistence(.saveItem(item)))]
      }
      return [Request(id: next, effect: .persistence(.loadItems))]
    }
    let store = Store(bridge: bridge)

    store.send(.setQuery(nil))
    store.send(.setUtcOffset(minutes: 0))
    await store.settle()

    #expect(bridge.persistenceResolved.map(\.id) == [1, 2])
    guard case .items(let items) = bridge.persistenceResolved.last?.output else {
      Issue.record("expected .items")
      return
    }
    #expect(items.map(\.id) == [item.id])
  }

  @Test func jobsInOneBatchResolveInTheOrderTheCoreAsked() async throws {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [
        Request(id: 1, effect: .persistence(.saveItem(StoreEffectLoopTests.sampleItem))),
        Request(id: 2, effect: .persistence(.loadItems)),
        Request(id: 3, effect: .persistence(.loadSessions)),
      ]
    }
    let store = Store(bridge: bridge)

    store.send(.setQuery(nil))
    await store.settle()

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
