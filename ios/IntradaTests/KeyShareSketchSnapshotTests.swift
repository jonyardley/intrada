import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class KeyShareSketchSnapshotTests: SnapshotTestCase {
  private func shot(_ look: KeyShareSketch.Look, _ state: KeyShareSketch.State)
    -> UIViewController
  {
    host(KeySharePlayerSketch(look: look, state: state))
  }

  func testHintFilling() { assertSnapshot(of: shot(.hint, KeyShareSketch.filling), as: config) }
  func testHintShareUp() { assertSnapshot(of: shot(.hint, KeyShareSketch.over), as: config) }
  func testHintLastKey() { assertSnapshot(of: shot(.hint, KeyShareSketch.lastKey), as: config) }
  func testHintOneKey() { assertSnapshot(of: shot(.hint, KeyShareSketch.oneKey), as: config) }
  func testHintUntimed() { assertSnapshot(of: shot(.hint, KeyShareSketch.untimed), as: config) }

  func testBesideFilling() {
    assertSnapshot(of: shot(.beside, KeyShareSketch.filling), as: config)
  }
  func testBesideShareUp() { assertSnapshot(of: shot(.beside, KeyShareSketch.over), as: config) }
  func testBesideLastKey() {
    assertSnapshot(of: shot(.beside, KeyShareSketch.lastKey), as: config)
  }
  func testBesideOneKey() { assertSnapshot(of: shot(.beside, KeyShareSketch.oneKey), as: config) }
  func testBesideUntimed() {
    assertSnapshot(of: shot(.beside, KeyShareSketch.untimed), as: config)
  }

  func testRingFilling() { assertSnapshot(of: shot(.ring, KeyShareSketch.filling), as: config) }
  func testRingShareUp() { assertSnapshot(of: shot(.ring, KeyShareSketch.over), as: config) }
  func testRingLastKey() { assertSnapshot(of: shot(.ring, KeyShareSketch.lastKey), as: config) }
  func testRingOneKey() { assertSnapshot(of: shot(.ring, KeyShareSketch.oneKey), as: config) }
  func testRingUntimed() { assertSnapshot(of: shot(.ring, KeyShareSketch.untimed), as: config) }

  func testLargestTextShareUp() {
    assertSnapshot(
      of: shot(.hint, KeyShareSketch.over),
      as: .image(
        on: .iPhone13, perceptualPrecision: 0.98,
        traits: UITraitCollection { traits in
          traits.displayScale = 2
          traits.preferredContentSizeCategory = .extraExtraExtraLarge
        }))
  }

  func testStrip() {
    assertSnapshot(
      of: KeyShareStripSketch(), as: .image(precision: 0.99, traits: .init(displayScale: 3)))
  }
}
