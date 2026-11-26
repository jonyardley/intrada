import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class PlanFromTimeSketchSnapshotTests: SnapshotTestCase {
  private var tall: Snapshotting<UIViewController, UIImage> {
    .image(
      on: ViewImageConfig(
        safeArea: .zero, size: CGSize(width: 390, height: 1400),
        traits: .init(displayScale: 1)),
      perceptualPrecision: 0.98, traits: .init(displayScale: 1))
  }

  private func largest(_ view: PlanFromTimeSketch) -> some View {
    view.environment(\.dynamicTypeSize, .xxxLarge)
  }

  func testTypicalA() {
    assertSnapshot(of: host(PlanFromTimeSketch.typical()), as: config)
  }

  func testTypicalB() {
    assertSnapshot(of: host(PlanFromTimeSketch.typical(.timer)), as: config)
  }

  func testEmptyWarmUpA() {
    assertSnapshot(of: host(PlanFromTimeSketch.emptyWarmUp()), as: config)
  }

  func testEmptyWarmUpB() {
    assertSnapshot(of: host(PlanFromTimeSketch.emptyWarmUp(.timer)), as: config)
  }

  func testLongestLargestTextA() {
    assertSnapshot(of: host(largest(PlanFromTimeSketch.longest())), as: tall)
  }

  func testLongestLargestTextB() {
    assertSnapshot(of: host(largest(PlanFromTimeSketch.longest(.timer))), as: tall)
  }

  func testWarmUpOffA() {
    assertSnapshot(of: host(PlanFromTimeSketch.warmUpOff), as: config)
  }

  func testUnderLengthA() {
    assertSnapshot(of: host(PlanFromTimeSketch.underLength), as: config)
  }

  func testLowerLimitA() {
    assertSnapshot(of: host(PlanFromTimeSketch.lowerLimit), as: config)
  }
}
