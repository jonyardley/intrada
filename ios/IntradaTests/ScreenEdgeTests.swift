import SharedTypes
import SwiftUI
import Testing
import UIKit

@testable import Intrada

/// A control row wider than the device does not merely overflow: the scaffold's
/// `ZStack` centres it, so the screen loses characters off *both* edges (#1470).
/// `testLibraryScreenAccessibilityText` is the pixel gate for the Library; this
/// covers the other pillars on the shared scaffold without a reference PNG each,
/// and adds the narrow device the snapshot host does not run.
@MainActor
struct ScreenEdgeTests {
  private static let height: CGFloat = 844
  /// iPhone 13, then the narrowest width still supported.
  private static let widths: [CGFloat] = [390, 320]

  /// Sub-pixel rounding only. The regression shifted the layout by roughly a
  /// third of the screen, so nothing here needs slack for it.
  private static let tolerance: CGFloat = 1

  private struct Edges {
    var minX: CGFloat
    var maxX: CGFloat
    /// Guards against a vacuous pass: SwiftUI backs many views with no `UIView`
    /// of their own, so a traversal that found nothing must fail, not read zero.
    var wideViews: Int
  }

  private func edges(of view: some View, store: Store, width: CGFloat) -> Edges {
    IntradaFonts.register()
    // Pin the hosting root to the window: given an over-large root of its own,
    // UIHostingController centres it, which reads as a leading-edge overflow
    // the view under test is not responsible for (#1481).
    let vc = UIHostingController(
      rootView:
        Color.clear
        .frame(width: width, height: Self.height)
        .overlay(view)
        .environment(store)
        .environment(\.locale, Locale(identifier: "en_US"))
        .environment(\.calendar, PreviewCalendar.utc)
        .environment(\.intradaMotionDisabled, true)
        .dynamicTypeSize(.accessibility5))
    vc.overrideUserInterfaceStyle = .light
    let window = UIWindow(frame: CGRect(x: 0, y: 0, width: width, height: Self.height))
    window.rootViewController = vc
    window.isHidden = false
    vc.view.layoutIfNeeded()
    defer { window.rootViewController = nil }

    var result = Edges(minX: 0, maxX: width, wideViews: 0)
    walk(vc.view, root: vc.view, width: width, into: &result)
    return result
  }

  private func walk(_ view: UIView, root: UIView, width: CGFloat, into result: inout Edges) {
    for subview in view.subviews where !subview.isHidden && subview.alpha > 0 {
      let frame = subview.convert(subview.bounds, to: root)
      if frame.width > 0 && frame.height > 0 {
        result.minX = min(result.minX, frame.minX)
        // A horizontal scroller is meant to run past the edge; its content
        // being wider than the screen is the point, not a layout failure.
        if !(view is UIScrollView) {
          result.maxX = max(result.maxX, frame.maxX)
        }
        if frame.width > width / 2 { result.wideViews += 1 }
      }
      walk(subview, root: root, width: width, into: &result)
    }
  }

  private func expectOnScreen(_ view: some View, store: Store, _ name: String) {
    for width in Self.widths {
      let found = edges(of: view, store: store, width: width)
      #expect(found.wideViews > 0, "\(name) at \(width)pt: measured nothing")
      #expect(
        found.minX >= -Self.tolerance,
        "\(name) at \(width)pt: runs \(-found.minX)pt off the leading edge")
      #expect(
        found.maxX <= width + Self.tolerance,
        "\(name) at \(width)pt: runs \(found.maxX - width)pt off the trailing edge")
    }
  }

  @Test("The Library stays on screen at the largest accessibility size")
  func libraryStaysOnScreen() {
    expectOnScreen(NavigationStack { LibraryScreen() }, store: .previewLibrary, "Library")
  }

  @Test("Practice stays on screen at the largest accessibility size")
  func practiceStaysOnScreen() {
    expectOnScreen(
      PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
      store: .previewPractice, "Practice")
  }

  @Test("Progress stays on screen at the largest accessibility size")
  func progressStaysOnScreen() {
    expectOnScreen(AnalyticsScreen(), store: .previewProgress, "Progress")
  }

  /// The two sheets reuse the browse bar with the type filter and the star
  /// switched off, so their control row has a different width to the Library's.
  @Test("The related-exercise sheet stays on screen at the largest accessibility size")
  func relatedExerciseSheetStaysOnScreen() {
    expectOnScreen(
      AddRelatedExerciseSheet(groupId: ""), store: .previewLibrary, "Related-exercise sheet")
  }
}
