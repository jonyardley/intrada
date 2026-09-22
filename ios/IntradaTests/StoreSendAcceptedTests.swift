import SharedTypes
import Testing

@testable import Intrada

private struct BridgeBroke: Error {}

@MainActor
struct StoreSendAcceptedTests {
  private let render = [Request(id: 1, effect: .render(RenderOperation()))]

  @Test func acceptsWhenTheCoreRendersWithoutAnError() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in render }
    let store = Store(bridge: bridge)

    #expect(store.sendAccepted(.setQuery(nil)))
  }

  @Test func refusesWhenTheCoreRaisesAnError() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in render }
    let store = Store(bridge: bridge)
    bridge.nextViewModel = {
      var vm = try emptyViewModel()
      vm.errorSeq += 1
      return vm
    }

    #expect(!store.sendAccepted(.setQuery(nil)))
  }

  @Test func refusesWhenTheBridgeThrowsOnUpdate() {
    let bridge = FakeBridge()
    let store = Store(bridge: bridge)
    bridge.throwOnUpdate = BridgeBroke()

    #expect(!store.sendAccepted(.setQuery(nil)))
  }

  @Test func refusesWhenTheBridgeThrowsOnTheRender() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in render }
    let store = Store(bridge: bridge)
    bridge.throwOnView = BridgeBroke()

    #expect(!store.sendAccepted(.setQuery(nil)))
  }

  @Test func refusesWhenThereIsNoViewModel() {
    let bridge = FakeBridge()
    bridge.throwOnView = BridgeBroke()
    let store = Store(bridge: bridge)
    bridge.throwOnView = nil

    #expect(store.viewModel == nil)
    #expect(!store.sendAccepted(.setQuery(nil)))
  }
}
