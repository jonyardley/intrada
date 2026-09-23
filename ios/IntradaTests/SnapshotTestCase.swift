import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class SnapshotStubBridge: CoreBridge {
  private let core = CoreFfi()
  var throwOnUpdate: Error?
  var nextViewModel: (() throws -> ViewModel)?
  func update(_ event: Event) throws -> [Request] {
    if let throwOnUpdate { throw throwOnUpdate }
    return []
  }
  func resolve(_ id: UInt32, persistenceOutput: PersistenceOutput) throws -> [Request] { [] }
  func resolve(_ id: UInt32, recognitionOutput: RecognitionOutput) throws -> [Request] { [] }
  func resolveEmpty(_ id: UInt32) throws -> [Request] { [] }
  func view() throws -> ViewModel {
    if let nextViewModel { return try nextViewModel() }
    return try ViewModel.bincodeDeserialize(input: [UInt8](core.view()))
  }
}

struct Boom: Error {}

/// Force light mode at the controller level (SwiftUI reads colorScheme from
/// here, not the snapshot `traits:`) and pin `.iPhone13` + displayScale so the
/// host sim can't change the image; references recorded on iOS 26.5 to match CI.
@MainActor
class SnapshotTestCase: XCTestCase {
  override func setUp() {
    super.setUp()
    IntradaFonts.register()
  }

  func host(_ view: some View, store: Store = Store(bridge: SnapshotStubBridge()))
    -> UIViewController
  {
    // Pin locale + calendar so date-driven UI (SessionCard's date, the week
    // strip) is deterministic regardless of host region/timezone: CI runs
    // en-US/UTC, dev sims often en-GB/local, which reorder dates and shift
    // day boundaries.
    // Suppress intro motion: the refreshed screens' entrance/one-shot animations
    // (fadeUp, count-up, ring-draw, barGrow, confetti) collapse to their final
    // state, so the captured frame is the settled layout, never a mid-reveal.
    // (`accessibilityReduceMotion` is read-only, so we use our settable flag.)
    // The marker follows the store's profile here as it does under RootView.
    let vc = UIHostingController(
      rootView: view.environment(store)
        .environment(\.marker, IntradaColor.marker(store.viewModel?.profile.colour ?? .butter))
        .environment(\.locale, Locale(identifier: "en_US"))
        .environment(\.calendar, PreviewCalendar.utc)
        .environment(\.intradaMotionDisabled, true))
    vc.overrideUserInterfaceStyle = .light
    return vc
  }

  var config: Snapshotting<UIViewController, UIImage> {
    .image(on: .iPhone13, perceptualPrecision: 0.98, traits: .init(displayScale: 2))
  }

  /// A frame tall enough to hold the whole add form: the chart and the staged
  /// rows sit below an iPhone 13's fold, so a device-sized frame captures the
  /// banner and nothing the marks do (#1595). Scale 1 keeps the reference
  /// smaller than a device-sized one despite the height.
  var tallFormConfig: Snapshotting<UIViewController, UIImage> {
    .image(
      on: ViewImageConfig(
        safeArea: .zero, size: CGSize(width: 390, height: 1500), traits: .init(displayScale: 1)),
      perceptualPrecision: 0.98, traits: .init(displayScale: 1))
  }

  var splitConfig: Snapshotting<UIViewController, UIImage> {
    .image(on: .iPadPro11(.landscape), perceptualPrecision: 0.98, traits: .init(displayScale: 1))
  }

  /// Largest accessibility text size: proves layouts reflow rather than clip/wrap.
  var axConfig: Snapshotting<UIViewController, UIImage> {
    .image(
      on: .iPhone13, perceptualPrecision: 0.98,
      traits: UITraitCollection { traits in
        traits.displayScale = 2
        traits.preferredContentSizeCategory = .accessibilityExtraExtraExtraLarge
      })
  }

  var practiceHeaderAxConfig: Snapshotting<UIViewController, UIImage> {
    .image(
      on: ViewImageConfig(
        safeArea: .zero, size: CGSize(width: 390, height: 1400), traits: .init(displayScale: 1)),
      perceptualPrecision: 0.98,
      traits: UITraitCollection { traits in
        traits.displayScale = 1
        traits.preferredContentSizeCategory = .accessibilityExtraExtraExtraLarge
      })
  }

  /// Flat fills only: the reference stays byte-stable and cheap as lossless PNG.
  static let page: UIImage = {
    let size = CGSize(width: 600, height: 850)
    return UIGraphicsImageRenderer(size: size).image { context in
      UIColor(white: 0.98, alpha: 1).setFill()
      context.fill(CGRect(origin: .zero, size: size))
      UIColor(white: 0.55, alpha: 1).setFill()
      for stave in 0..<5 {
        for line in 0..<5 {
          context.fill(
            CGRect(x: 60, y: 140 + stave * 130 + line * 12, width: 480, height: 2))
        }
      }
    }
  }()
}
