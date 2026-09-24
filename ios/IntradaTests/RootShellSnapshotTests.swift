import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class RootShellSnapshotTests: SnapshotTestCase {
  func testRootShell() {
    assertSnapshot(of: host(RootView()), as: config)
  }

  func testRootShellAccessibilitySize() {
    assertSnapshot(of: host(RootView()), as: axConfig)
  }

  /// The tab shell after the core has panicked (#1946): the last screen stays
  /// up under a standing banner.
  func testRootShellHalted() {
    let bridge = SnapshotStubBridge()
    bridge.throwOnUpdate = CorePanic(underlying: Boom())
    let store = Store(bridge: bridge)
    store.send(.setQuery(nil))
    assertSnapshot(of: host(RootView(), store: store), as: config)
  }

  func testRootShellWithNotice() {
    let bridge = SnapshotStubBridge()
    bridge.nextViewModel = {
      var model = try emptyViewModel()
      model.notice = "That metronome setting doesn't give a crotchet tempo, so this play has none."
      return model
    }
    assertSnapshot(of: host(RootView(), store: Store(bridge: bridge)), as: config)
  }

  func testGlobalBanner() {
    let banners = ZStack {
      PaperBackground()
      VStack(spacing: 0) {
        GlobalBanner(message: "Couldn't delete that item.", onDismiss: {})
        GlobalBanner(message: "Storage unavailable · changes this session won't be saved.")
        GlobalBanner(
          message: "That metronome setting doesn't give a crotchet tempo, so this play has none.",
          tone: .notice, onDismiss: {})
        Spacer()
      }
    }
    assertSnapshot(of: host(banners), as: config)
  }
}
