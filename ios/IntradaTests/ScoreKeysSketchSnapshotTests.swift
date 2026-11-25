import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class ScoreKeysSketchSnapshotTests: SnapshotTestCase {
  func testThreeKeysWithDMajorOpen() {
    assertSnapshot(of: host(sheet(.threeKeys)), as: config)
  }

  func testSixKeys() {
    assertSnapshot(of: host(sheet(.sixKeys)), as: tall(1000))
  }

  func testKeyNeverScored() {
    assertSnapshot(of: host(sheet(.neverScored)), as: config)
  }

  private func sheet(_ sketch: ScoreKeysSketch) -> some View {
    ZStack(alignment: .bottom) {
      PaperBackground()
      sketch
    }
  }

  private func tall(_ height: CGFloat) -> Snapshotting<UIViewController, UIImage> {
    .image(
      on: ViewImageConfig(
        safeArea: .zero, size: CGSize(width: 390, height: height), traits: .init(displayScale: 2)),
      perceptualPrecision: 0.98, traits: .init(displayScale: 2))
  }
}
