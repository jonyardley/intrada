import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class KeyShareSketchSnapshotTests: SnapshotTestCase {
  func testCanvasFilling() {
    assertSnapshot(
      of: host(KeySharePlayerSketch(look: .canvas, state: KeyShareSketch.filling)), as: config)
  }

  func testCanvasOver() {
    assertSnapshot(
      of: host(KeySharePlayerSketch(look: .canvas, state: KeyShareSketch.over)), as: config)
  }

  func testCanvasLastKeyOver() {
    assertSnapshot(
      of: host(KeySharePlayerSketch(look: .canvas, state: KeyShareSketch.lastOver)), as: config)
  }

  func testCanvasLongestLargestText() {
    assertSnapshot(
      of: host(
        KeySharePlayerSketch(look: .canvas, state: KeyShareSketch.longest)
          .environment(\.dynamicTypeSize, .xxxLarge)),
      as: config)
  }

  func testOneKey() {
    assertSnapshot(
      of: host(KeySharePlayerSketch(look: .canvas, state: KeyShareSketch.oneKey)), as: config)
  }

  func testSteppedFilling() {
    assertSnapshot(
      of: host(KeySharePlayerSketch(look: .stepped, state: KeyShareSketch.filling)), as: config)
  }

  func testSteppedOver() {
    assertSnapshot(
      of: host(KeySharePlayerSketch(look: .stepped, state: KeyShareSketch.over)), as: config)
  }

  func testQuietFilling() {
    assertSnapshot(
      of: host(KeySharePlayerSketch(look: .quiet, state: KeyShareSketch.filling)), as: config)
  }

  func testQuietOver() {
    assertSnapshot(
      of: host(KeySharePlayerSketch(look: .quiet, state: KeyShareSketch.over)), as: config)
  }
}
