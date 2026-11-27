import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class OverrunSketchSnapshotTests: SnapshotTestCase {
  private func sheet(_ notice: OverrunSketch.Notice)
    -> UIViewController
  {
    host(OverrunSheetSketch(look: .plain, notice: notice))
  }

  private var largestText: Snapshotting<UIViewController, UIImage> {
    .image(
      on: .iPhone13, perceptualPrecision: 0.98,
      traits: UITraitCollection { traits in
        traits.displayScale = 2
        traits.preferredContentSizeCategory = .extraExtraExtraLarge
      })
  }

  func testSheetTwo() { assertSnapshot(of: sheet(OverrunSketch.two), as: config) }
  func testSheetOne() { assertSnapshot(of: sheet(OverrunSketch.one), as: config) }
  func testSheetMany() { assertSnapshot(of: sheet(OverrunSketch.many), as: config) }
  func testSheetSqueezed() { assertSnapshot(of: sheet(OverrunSketch.squeezed), as: config) }
  func testSheetKept() { assertSnapshot(of: sheet(OverrunSketch.kept), as: config) }
  func testSheetLargestText() { assertSnapshot(of: sheet(OverrunSketch.many), as: largestText) }

  func testPlayer() { assertSnapshot(of: host(OverrunPlayerSketch(look: .plain)), as: config) }
  func testPlayerLargestText() {
    assertSnapshot(of: host(OverrunPlayerSketch(look: .plain)), as: largestText)
  }
}
