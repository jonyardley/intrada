import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

/// Deterministic bridge for snapshots: serves the core's initial (empty)
/// ViewModel and emits no effects, so the screen renders without networking.
private final class StubBridge: CoreBridge {
  private let core = CoreFfi()
  func update(_ event: Event) throws -> [Request] { [] }
  func resolve(_ id: UInt32, httpResult: HttpResult) throws -> [Request] { [] }
  func resolveEmpty(_ id: UInt32) throws -> [Request] { [] }
  func view() throws -> ViewModel {
    try ViewModel.bincodeDeserialize(input: [UInt8](core.view()))
  }
}

@MainActor
final class RootViewSnapshotTests: XCTestCase {
  func testFoundationScreen() {
    let store = Store(bridge: StubBridge())
    // Force light mode at the controller level (SwiftUI reads colorScheme from
    // here, not the snapshot `traits:`) so CI's dark-default sim can't invert
    // the image. Pin displayScale too so the size is host-independent.
    let vc = UIHostingController(rootView: RootView().environment(store))
    vc.overrideUserInterfaceStyle = .light
    assertSnapshot(
      of: vc,
      as: .image(on: .iPhone13, perceptualPrecision: 0.98, traits: .init(displayScale: 2))
    )
  }
}
