import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class KeyShareSketchSnapshotTests: SnapshotTestCase {
  private func shot(
    _ look: KeyShareSketch.Look, _ state: KeyShareSketch.State, largest: Bool = false
  ) -> UIViewController {
    host(
      KeySharePlayerSketch(look: look, state: state)
        .environment(\.dynamicTypeSize, largest ? .xxxLarge : .large))
  }

  func testCardFilling() {
    assertSnapshot(of: shot(.card, KeyShareSketch.filling), as: config)
  }

  func testCardOver() {
    assertSnapshot(of: shot(.card, KeyShareSketch.over), as: config)
  }

  func testCardLongestLargestText() {
    assertSnapshot(of: shot(.card, KeyShareSketch.longest, largest: true), as: config)
  }

  func testInlineFilling() {
    assertSnapshot(of: shot(.inline, KeyShareSketch.filling), as: config)
  }

  func testInlineOver() {
    assertSnapshot(of: shot(.inline, KeyShareSketch.over), as: config)
  }

  func testInlineLastKeyOver() {
    assertSnapshot(of: shot(.inline, KeyShareSketch.lastOver), as: config)
  }

  func testInlineLongestLargestText() {
    assertSnapshot(of: shot(.inline, KeyShareSketch.longest, largest: true), as: config)
  }

  func testOneKey() {
    assertSnapshot(of: shot(.inline, KeyShareSketch.oneKey), as: config)
  }
}
