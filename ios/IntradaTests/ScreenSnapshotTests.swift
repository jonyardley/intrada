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
  override func setUp() {
    super.setUp()
    IntradaFonts.register()
  }

  private func host(_ view: some View, store: Store = Store(bridge: StubBridge()))
    -> UIViewController
  {
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
    assertSnapshot(of: host(NavigationStack { LibraryScreen() }), as: config)
  }

  func testLibraryScreenPopulated() {
    assertSnapshot(
      of: host(NavigationStack { LibraryScreen() }, store: .previewLibrary), as: config)
  }

  func testLibraryScreenFiltered() {
    assertSnapshot(
      of: host(NavigationStack { LibraryScreen() }, store: .previewLibraryFiltered), as: config)
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

  func testLibraryDetailScreen() {
    // Push via a preset path so the snapshot covers the real navigation chrome
    // (back chevron + transparent bar over the serif title), not just the body.
    // Keyed by id now, so the store must hold the pushed item.
    let store = Store(bridge: PreviewBridge(items: [.previewDetail]))
    let pushed = NavigationStack(path: .constant([LibraryItemView.previewDetail.id])) {
      LibraryScreen()
    }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  func testLibraryAddScreen() {
    assertSnapshot(of: host(LibraryAddScreen()), as: config)
  }

  func testLibraryAddScreenExercise() {
    assertSnapshot(of: host(LibraryAddScreen(defaultKind: .exercise)), as: config)
  }

  func testLibraryEditScreen() {
    assertSnapshot(of: host(LibraryEditScreen(item: .previewDetail)), as: config)
  }

  func testTypeBadges() {
    let badges = ZStack {
      PaperBackground()
      HStack(spacing: 12) {
        TypeBadge(kind: .piece)
        TypeBadge(kind: .exercise)
      }
    }
    assertSnapshot(of: host(badges), as: config)
  }

  func testLibraryFilterTabs() {
    let tabs = ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: 16) {
        LibraryFilterTabs(selection: .constant(.all))
        LibraryFilterTabs(selection: .constant(.pieces))
        LibraryFilterTabs(selection: .constant(.exercises))
      }
      .padding(16)
    }
    assertSnapshot(of: host(tabs), as: config)
  }

  func testKeyPickerCollapsed() {
    let pickers = ZStack {
      PaperBackground()
      VStack(spacing: 16) {
        VStack(spacing: 0) { KeyPicker(label: "Key", text: .constant("")) }.cardSurface()
        VStack(spacing: 0) { KeyPicker(label: "Key", text: .constant("Gb major")) }.cardSurface()
      }
      .padding(16)
    }
    assertSnapshot(of: host(pickers), as: config)
  }

  func testKeyPickerExpandedEmpty() {
    let picker = ZStack {
      PaperBackground()
      VStack(spacing: 0) {
        KeyPicker(label: "Key", text: .constant(""), initiallyExpanded: true)
      }
      .cardSurface()
      .padding(16)
    }
    assertSnapshot(of: host(picker), as: config)
  }

  func testKeyPickerExpandedEnharmonic() {
    let picker = ZStack {
      PaperBackground()
      VStack(spacing: 0) {
        KeyPicker(label: "Key", text: .constant("Gb major"), initiallyExpanded: true)
      }
      .cardSurface()
      .padding(16)
    }
    assertSnapshot(of: host(picker), as: config)
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
