import Foundation
import Observation
import SharedTypes
import Testing

@testable import Intrada

/// The library rows and history reach the Store as their own messages, and a
/// tap mid-practice leaves them alone so their screens are not redrawn (#1801).
@MainActor
struct StoreSectionsTests {
  private static func stamp(_ minute: Int) -> String {
    SessionClock.nowRFC3339(Date(timeIntervalSince1970: 1_767_225_600 + Double(minute) * 60))
  }

  private static func piece(_ title: String) -> Event {
    .item(
      .add(
        CreateItem(
          title: title, kind: .piece, composer: nil, key: nil, modality: nil, tempo: nil,
          notes: nil, tags: [], photoId: nil, variantLabels: [])))
  }

  private final class Flag: @unchecked Sendable {
    var raised = false
  }

  private func watch(_ read: @escaping () -> Void) -> Flag {
    let flag = Flag()
    withObservationTracking(read) { flag.raised = true }
    return flag
  }

  @Test("an added piece reaches the rows, and Start and Next leave the rows, history and weeks")
  func nextLeavesTheSectionsAlone() async throws {
    let suite = "StoreSectionsTests-\(UUID().uuidString)"
    let defaults = try #require(UserDefaults(suiteName: suite))
    defer { defaults.removePersistentDomain(forName: suite) }
    let store = Store(sortDefaults: defaults)
    store.send(.startApp)
    await store.settle()
    store.send(Self.piece("Clair de Lune"))
    store.send(Self.piece("Arabesque"))
    await store.settle()
    #expect(Swift.Set(store.libraryRows.map(\.title)) == ["Clair de Lune", "Arabesque"])

    store.send(.session(.startBuilding))
    for row in store.libraryRows {
      store.send(.session(.addToSetlist(itemId: row.id)))
    }
    let rows = watch { _ = store.libraryRows }
    let history = watch { _ = store.sessionHistory }
    let weeks = watch { _ = store.practiceWeeks }
    store.send(.session(.startSession(now: Self.stamp(0))))
    store.send(
      .session(.nextItem(now: Self.stamp(5), nextItemStartedAt: Self.stamp(5), reading: .silent)))
    await store.settle()

    #expect(store.viewModel?.activeSession?.currentPosition == 1)
    #expect(!rows.raised)
    #expect(!history.raised)
    #expect(!weeks.raised)
  }
}
