import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class OverrunSketchSnapshotTests: SnapshotTestCase {
  private func sheet(_ look: OverrunSketch.Look, _ notice: OverrunSketch.Notice)
    -> UIViewController
  {
    host(OverrunSheetSketch(look: look, notice: notice))
  }

  private var largestText: Snapshotting<UIViewController, UIImage> {
    .image(
      on: .iPhone13, perceptualPrecision: 0.98,
      traits: UITraitCollection { traits in
        traits.displayScale = 2
        traits.preferredContentSizeCategory = .extraExtraExtraLarge
      })
  }

  func testSheetSunkenTwo() { assertSnapshot(of: sheet(.sunken, OverrunSketch.two), as: config) }
  func testSheetSunkenOne() { assertSnapshot(of: sheet(.sunken, OverrunSketch.one), as: config) }
  func testSheetSunkenMany() { assertSnapshot(of: sheet(.sunken, OverrunSketch.many), as: config) }
  func testSheetSunkenSqueezed() {
    assertSnapshot(of: sheet(.sunken, OverrunSketch.squeezed), as: config)
  }
  func testSheetSunkenKept() { assertSnapshot(of: sheet(.sunken, OverrunSketch.kept), as: config) }
  func testSheetCardTwo() { assertSnapshot(of: sheet(.card, OverrunSketch.two), as: config) }
  func testSheetCardMany() { assertSnapshot(of: sheet(.card, OverrunSketch.many), as: config) }
  func testSheetPlainTwo() { assertSnapshot(of: sheet(.plain, OverrunSketch.two), as: config) }
  func testSheetPlainMany() { assertSnapshot(of: sheet(.plain, OverrunSketch.many), as: config) }
  func testSheetLargestText() {
    assertSnapshot(of: sheet(.sunken, OverrunSketch.many), as: largestText)
  }

  func testPlayerSunken() {
    assertSnapshot(of: host(OverrunPlayerSketch(look: .sunken)), as: config)
  }
  func testPlayerCard() { assertSnapshot(of: host(OverrunPlayerSketch(look: .card)), as: config) }
  func testPlayerPlain() { assertSnapshot(of: host(OverrunPlayerSketch(look: .plain)), as: config) }
  func testPlayerLargestText() {
    assertSnapshot(of: host(OverrunPlayerSketch(look: .sunken)), as: largestText)
  }
}
