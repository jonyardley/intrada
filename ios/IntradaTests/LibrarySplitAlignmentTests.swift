import SwiftUI
import Testing
import UIKit

@testable import Intrada

/// #1682 only shows up in a real window: a windowless `UIHostingController`
/// lays out no nav bar at all, so the snapshot suite cannot catch it.
@MainActor
struct LibrarySplitAlignmentTests {
  private static let size = CGSize(width: 1194, height: 834)
  /// Sub-pixel rounding only. The drift this covers was a whole nav bar.
  private static let tolerance: CGFloat = 1

  private func renderedSplit(selecting id: String?) -> (edges: [CGFloat], viewCount: Int) {
    IntradaFonts.register()
    let vc = UIHostingController(
      rootView: LibrarySplitView(previewSelection: id)
        .environment(\.horizontalSizeClass, .regular)
        .environment(Store.previewLibrary)
        .environment(\.locale, Locale(identifier: "en_US"))
        .environment(\.calendar, PreviewCalendar.utc)
        .environment(\.intradaMotionDisabled, true))
    vc.overrideUserInterfaceStyle = .light
    let window = UIWindow(frame: CGRect(origin: .zero, size: Self.size))
    window.rootViewController = vc
    window.isHidden = false
    vc.view.layoutIfNeeded()
    defer { window.rootViewController = nil }

    var found: [(minX: CGFloat, maxY: CGFloat)] = []
    let count = walk(vc.view, root: vc.view, into: &found)
    return (found.sorted { $0.minX < $1.minX }.map(\.maxY), count)
  }

  @discardableResult
  private func walk(
    _ view: UIView, root: UIView, into found: inout [(minX: CGFloat, maxY: CGFloat)]
  ) -> Int {
    var count = 1
    for subview in view.subviews {
      if subview is UINavigationBar {
        let frame = subview.convert(subview.bounds, to: root)
        found.append((minX: frame.minX, maxY: subview.isHidden ? 0 : frame.maxY))
      }
      count += walk(subview, root: root, into: &found)
    }
    return count
  }

  @Test func bothColumnsReserveTheSameTopChrome() {
    assertSymmetricChrome(renderedSplit(selecting: "piece-1"))
  }

  /// #1681's launch state: nothing selected, so the detail column is bare.
  @Test func bothColumnsReserveTheSameTopChromeWithNoSelection() {
    assertSymmetricChrome(renderedSplit(selecting: nil))
  }

  // Both columns hide their native bar entirely now (#1724), so finding none
  // is symmetric and fine; finding exactly one is the asymmetry this guards
  // against. Zero nav bars is only meaningful alongside a real render: the
  // view-count floor is the positive anchor that keeps a broken hierarchy
  // (which would also find zero) from passing in silence.
  private func assertSymmetricChrome(_ rendered: (edges: [CGFloat], viewCount: Int)) {
    #expect(
      rendered.viewCount > 10,
      "expected a fully rendered split, walked only \(rendered.viewCount) views")

    #expect(
      rendered.edges.count == 0 || rendered.edges.count == 2,
      "expected a matched pair of nav bars or none, found \(rendered.edges.count)")

    guard rendered.edges.count == 2 else { return }
    #expect(
      abs(rendered.edges[0] - rendered.edges[1]) <= Self.tolerance,
      "columns reserve different top chrome: list \(rendered.edges[0]), detail \(rendered.edges[1])"
    )
  }
}
