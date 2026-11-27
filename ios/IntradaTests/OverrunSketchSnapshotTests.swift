import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class OverrunSketchSnapshotTests: SnapshotTestCase {
  private func sheet(
    _ option: OverrunSketch.Option, _ notice: OverrunSketch.Notice = OverrunSketch.two
  ) -> UIViewController {
    host(OverrunSheetSketch(option: option, notice: notice))
  }

  private var largestText: Snapshotting<UIViewController, UIImage> {
    .image(
      on: .iPhone13, perceptualPrecision: 0.98,
      traits: UITraitCollection { traits in
        traits.displayScale = 2
        traits.preferredContentSizeCategory = .extraExtraExtraLarge
      })
  }

  func testASwitchTwo() { assertSnapshot(of: sheet(.switchRow), as: config) }
  func testASwitchSqueezed() {
    assertSnapshot(of: sheet(.switchRow, OverrunSketch.squeezed), as: config)
  }
  func testASwitchOff() {
    assertSnapshot(
      of: host(OverrunSheetSketch(option: .switchRow, shortenOn: false)), as: config)
  }
  func testASwitchLargestText() { assertSnapshot(of: sheet(.switchRow), as: largestText) }

  func testBUnderSaveTwo() { assertSnapshot(of: sheet(.lineUnderSave), as: config) }
  func testBUnderSaveSqueezed() {
    assertSnapshot(of: sheet(.lineUnderSave, OverrunSketch.squeezed), as: config)
  }
  func testBUnderSaveLargestText() {
    assertSnapshot(of: sheet(.lineUnderSave), as: largestText)
  }

  func testCSheetUnchanged() { assertSnapshot(of: sheet(.playerOnly), as: config) }
  func testCNextPlayer() { assertSnapshot(of: host(OverrunNextPlayerSketch()), as: config) }
  func testCNextPlayerLargestText() {
    assertSnapshot(of: host(OverrunNextPlayerSketch()), as: largestText)
  }

  func testPlayer() { assertSnapshot(of: host(OverrunPlayerSketch()), as: config) }
  func testPlayerLargestText() {
    assertSnapshot(of: host(OverrunPlayerSketch()), as: largestText)
  }
}
