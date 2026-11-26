import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class PlanFromTimeSketchSnapshotTests: SnapshotTestCase {
  private func tall(_ height: CGFloat) -> Snapshotting<UIViewController, UIImage> {
    .image(
      on: ViewImageConfig(
        safeArea: .zero, size: CGSize(width: 390, height: height),
        traits: .init(displayScale: 1)),
      perceptualPrecision: 0.98, traits: .init(displayScale: 1))
  }

  func testEmptyWarmUpRings() {
    assertSnapshot(of: host(PlanFromTimeSketch.emptyWarmUp()), as: tall(900))
  }

  func testTypicalRings() {
    assertSnapshot(of: host(PlanFromTimeSketch.typical()), as: tall(900))
  }

  func testTypicalRingsWithHint() {
    assertSnapshot(of: host(PlanFromTimeSketch.typical(.ringsWithHint)), as: tall(900))
  }

  func testTypicalSystemStepper() {
    assertSnapshot(of: host(PlanFromTimeSketch.typical(.systemStepper)), as: tall(900))
  }

  func testLongestLargeText() {
    assertSnapshot(
      of: host(PlanFromTimeSketch.longest().environment(\.dynamicTypeSize, .xxxLarge)),
      as: tall(1700))
  }

  func testLongestLargeTextWithHint() {
    assertSnapshot(
      of: host(
        PlanFromTimeSketch.longest(.ringsWithHint).environment(\.dynamicTypeSize, .xxxLarge)),
      as: tall(1700))
  }
}
