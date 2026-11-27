import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class OverrunSketchSnapshotTests: SnapshotTestCase {
  private var largestText: Snapshotting<UIViewController, UIImage> {
    .image(
      on: .iPhone13, perceptualPrecision: 0.98,
      traits: UITraitCollection { traits in
        traits.displayScale = 2
        traits.preferredContentSizeCategory = .extraExtraExtraLarge
      })
  }

  func testSwitchOn() { assertSnapshot(of: host(OverrunSheetSketch()), as: config) }
  func testSwitchOff() {
    assertSnapshot(of: host(OverrunSheetSketch(shortenOn: false)), as: config)
  }
  func testTooLittleTimeLeft() {
    assertSnapshot(of: host(OverrunSheetSketch(notice: OverrunSketch.squeezed)), as: config)
  }
  func testThreeItems() {
    assertSnapshot(of: host(OverrunSheetSketch(notice: OverrunSketch.three)), as: config)
  }
  func testThreeItemsLargestText() {
    assertSnapshot(of: host(OverrunSheetSketch(notice: OverrunSketch.three)), as: largestText)
  }

  func testPlayer() { assertSnapshot(of: host(OverrunPlayerSketch()), as: config) }
  func testPlayerLargestText() {
    assertSnapshot(of: host(OverrunPlayerSketch()), as: largestText)
  }
}
