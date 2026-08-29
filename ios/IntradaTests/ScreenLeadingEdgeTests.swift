import SharedTypes
import SwiftUI
import Testing
import UIKit

@testable import Intrada

/// A `ZStack` centres a child wider than itself, so content that cannot shrink
/// at accessibility sizes slides off the leading edge and loses the first
/// characters of every line (#1470). `testLibraryScreenAccessibilityText` is the
/// pixel gate; this covers the other pillars without a reference PNG each.
@MainActor
struct ScreenLeadingEdgeTests {
  private static let width: CGFloat = 390
  private static let height: CGFloat = 844

  /// Shadows bleed a few points past the edge; the regression shifted the
  /// layout by roughly a third of the screen, so the two never overlap.
  private static let tolerance: CGFloat = 12

  private func leadingOverflow(of view: some View, store: Store) -> CGFloat {
    let vc = UIHostingController(
      rootView:
        view
        .environment(store)
        .environment(\.locale, Locale(identifier: "en_US"))
        .environment(\.calendar, PreviewCalendar.utc)
        .environment(\.intradaMotionDisabled, true)
        .dynamicTypeSize(.accessibility5))
    vc.overrideUserInterfaceStyle = .light
    let window = UIWindow(
      frame: CGRect(x: 0, y: 0, width: Self.width, height: Self.height))
    window.rootViewController = vc
    window.makeKeyAndVisible()
    vc.view.layoutIfNeeded()
    return minX(in: vc.view, root: vc.view)
  }

  private func minX(in view: UIView, root: UIView) -> CGFloat {
    var lowest: CGFloat = 0
    for subview in view.subviews where !subview.isHidden && subview.alpha > 0 {
      let frame = subview.convert(subview.bounds, to: root)
      if frame.width > 0 && frame.height > 0 {
        lowest = min(lowest, frame.minX)
      }
      lowest = min(lowest, minX(in: subview, root: root))
    }
    return lowest
  }

  @Test("The Library stays on screen at the largest accessibility size")
  func libraryStaysOnScreen() {
    let overflow = leadingOverflow(
      of: NavigationStack { LibraryScreen() }, store: .previewLibrary)
    #expect(overflow >= -Self.tolerance)
  }

  @Test("Practice stays on screen at the largest accessibility size")
  func practiceStaysOnScreen() {
    let overflow = leadingOverflow(
      of: PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
      store: .previewPractice)
    #expect(overflow >= -Self.tolerance)
  }

  @Test("Progress stays on screen at the largest accessibility size")
  func progressStaysOnScreen() {
    let overflow = leadingOverflow(of: AnalyticsScreen(), store: .previewProgress)
    #expect(overflow >= -Self.tolerance)
  }
}
