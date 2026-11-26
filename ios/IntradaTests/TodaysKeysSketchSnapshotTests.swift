import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class TodaysKeysSketchSnapshotTests: SnapshotTestCase {
  private var tall: Snapshotting<UIViewController, UIImage> {
    .image(
      on: ViewImageConfig(
        safeArea: .zero, size: CGSize(width: 390, height: 1300),
        traits: .init(displayScale: 1)),
      perceptualPrecision: 0.98, traits: .init(displayScale: 1))
  }

  private func dark(_ view: some View) -> UIViewController {
    let vc = host(view)
    vc.overrideUserInterfaceStyle = .dark
    return vc
  }

  func testRowChips() {
    assertSnapshot(of: host(TodaysKeysRowSketch(labels: .chips)), as: config)
  }

  func testRowLine() {
    assertSnapshot(of: host(TodaysKeysRowSketch(labels: .line)), as: config)
  }

  func testRowChipsLargestText() {
    assertSnapshot(
      of: host(TodaysKeysRowSketch(labels: .chips).environment(\.dynamicTypeSize, .xxxLarge)),
      as: config)
  }

  func testRowLineLargestText() {
    assertSnapshot(
      of: host(TodaysKeysRowSketch(labels: .line).environment(\.dynamicTypeSize, .xxxLarge)),
      as: config)
  }

  func testRowChipsDark() {
    assertSnapshot(of: dark(TodaysKeysRowSketch(labels: .chips)), as: config)
  }

  func testSheetCaption() {
    assertSnapshot(of: host(TodaysKeysSheetSketch(marking: .caption)), as: tall)
  }

  func testSheetGrouped() {
    assertSnapshot(of: host(TodaysKeysSheetSketch(marking: .grouped)), as: tall)
  }

  func testSheetCaptionDark() {
    assertSnapshot(of: dark(TodaysKeysSheetSketch(marking: .caption)), as: tall)
  }
}
