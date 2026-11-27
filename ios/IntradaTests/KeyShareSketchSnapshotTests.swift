import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class KeyShareSketchSnapshotTests: SnapshotTestCase {
  private func shot(_ count: KeyShareSketch.Count, _ state: KeyShareSketch.State)
    -> UIViewController
  {
    host(KeySharePlayerSketch(count: count, state: state))
  }

  func testChipFilling() { assertSnapshot(of: shot(.chip, KeyShareSketch.filling), as: config) }
  func testChipShareUp() { assertSnapshot(of: shot(.chip, KeyShareSketch.over), as: config) }
  func testChipLastKey() { assertSnapshot(of: shot(.chip, KeyShareSketch.lastKey), as: config) }
  func testChipOneKey() { assertSnapshot(of: shot(.chip, KeyShareSketch.oneKey), as: config) }
  func testChipVariations() {
    assertSnapshot(of: shot(.chip, KeyShareSketch.variations), as: config)
  }

  func testUnderChipFilling() {
    assertSnapshot(of: shot(.underChip, KeyShareSketch.filling), as: config)
  }
  func testUnderChipShareUp() {
    assertSnapshot(of: shot(.underChip, KeyShareSketch.over), as: config)
  }
  func testUnderChipLastKey() {
    assertSnapshot(of: shot(.underChip, KeyShareSketch.lastKey), as: config)
  }
  func testUnderChipOneKey() {
    assertSnapshot(of: shot(.underChip, KeyShareSketch.oneKey), as: config)
  }
  func testUnderChipVariations() {
    assertSnapshot(of: shot(.underChip, KeyShareSketch.variations), as: config)
  }

  func testBandFilling() { assertSnapshot(of: shot(.band, KeyShareSketch.filling), as: config) }
  func testBandShareUp() { assertSnapshot(of: shot(.band, KeyShareSketch.over), as: config) }
  func testBandLastKey() { assertSnapshot(of: shot(.band, KeyShareSketch.lastKey), as: config) }
  func testBandOneKey() { assertSnapshot(of: shot(.band, KeyShareSketch.oneKey), as: config) }
  func testBandVariations() {
    assertSnapshot(of: shot(.band, KeyShareSketch.variations), as: config)
  }

  func testLargestTextShareUp() {
    assertSnapshot(
      of: shot(.band, KeyShareSketch.over),
      as: .image(
        on: .iPhone13, perceptualPrecision: 0.98,
        traits: UITraitCollection { traits in
          traits.displayScale = 2
          traits.preferredContentSizeCategory = .extraExtraExtraLarge
        }))
  }
}
