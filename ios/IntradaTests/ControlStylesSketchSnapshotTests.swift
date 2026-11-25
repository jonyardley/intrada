import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class ControlStylesSketchSnapshotTests: SnapshotTestCase {
  func testRingsScreen() { assertSnapshot(of: host(screen(editing: false)), as: config) }
  func testSmallSheetEdit() { assertSnapshot(of: host(screen(editing: true)), as: config) }

  private func screen(editing: Bool) -> some View {
    ZStack {
      PaperBackground()
      ControlStylesSketch.threeKeys(editing: editing)
    }
  }
}
