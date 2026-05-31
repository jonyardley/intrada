import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

/// Deterministic bridge for snapshots: serves the core's initial (empty)
/// ViewModel and emits no effects, so screens render without networking.
private final class StubBridge: CoreBridge {
  private let core = CoreFfi()
  func update(_ event: Event) throws -> [Request] { [] }
  func resolve(_ id: UInt32, httpResult: HttpResult) throws -> [Request] { [] }
  func resolveEmpty(_ id: UInt32) throws -> [Request] { [] }
  func view() throws -> ViewModel {
    try ViewModel.bincodeDeserialize(input: [UInt8](core.view()))
  }
}

/// UI-regression "eyes" for the tab shell and each placeholder screen. Force
/// light mode at the controller level (SwiftUI reads colorScheme from here, not
/// the snapshot `traits:`) so a dark-default sim can't invert the image. The
/// `.iPhone13` config + pinned displayScale fix the geometry regardless of host
/// sim, so the rasterizing iOS renderer is the only host variable — references
/// are recorded on iOS 26.5 / Xcode 26.5 to match CI (see ci.yml).
@MainActor
final class ScreenSnapshotTests: XCTestCase {
  private func host(_ view: some View) -> UIViewController {
    let store = Store(bridge: StubBridge())
    let vc = UIHostingController(rootView: view.environment(store))
    vc.overrideUserInterfaceStyle = .light
    return vc
  }

  private var config: Snapshotting<UIViewController, UIImage> {
    .image(on: .iPhone13, perceptualPrecision: 0.98, traits: .init(displayScale: 2))
  }

  func testRootShell() {
    assertSnapshot(of: host(RootView()), as: config)
  }

  func testLibraryScreen() {
    assertSnapshot(of: host(LibraryScreen()), as: config)
  }

  func testPracticeScreen() {
    assertSnapshot(of: host(PracticeScreen()), as: config)
  }

  func testRoutinesScreen() {
    assertSnapshot(of: host(RoutinesScreen()), as: config)
  }

  func testAnalyticsScreen() {
    assertSnapshot(of: host(AnalyticsScreen()), as: config)
  }

  func testLibraryItemCards() {
    let cards = ZStack {
      PaperBackground()
      VStack(spacing: 14) {
        LibraryItemCard(item: .previewPiece)
        LibraryItemCard(item: .previewExercise)
      }
      .padding(16)
    }
    assertSnapshot(of: host(cards), as: config)
  }
}
