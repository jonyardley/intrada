import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class KeyShareSketchSnapshotTests: SnapshotTestCase {
  private func shot(_ mark: KeyShareSketch.Mark, _ state: KeyShareSketch.State)
    -> UIViewController
  {
    host(KeySharePlayerSketch(mark: mark, state: state))
  }

  func testGapsFilling() { assertSnapshot(of: shot(.gaps, KeyShareSketch.filling), as: config) }
  func testGapsOver() { assertSnapshot(of: shot(.gaps, KeyShareSketch.over), as: config) }
  func testGapsLastKeyOver() {
    assertSnapshot(of: shot(.gaps, KeyShareSketch.lastOver), as: config)
  }

  func testTicksFilling() { assertSnapshot(of: shot(.ticks, KeyShareSketch.filling), as: config) }
  func testTicksOver() { assertSnapshot(of: shot(.ticks, KeyShareSketch.over), as: config) }
  func testTicksLastKeyOver() {
    assertSnapshot(of: shot(.ticks, KeyShareSketch.lastOver), as: config)
  }

  func testTrackFilling() { assertSnapshot(of: shot(.track, KeyShareSketch.filling), as: config) }
  func testTrackOver() { assertSnapshot(of: shot(.track, KeyShareSketch.over), as: config) }
  func testTrackLastKeyOver() {
    assertSnapshot(of: shot(.track, KeyShareSketch.lastOver), as: config)
  }

  func testOneKey() { assertSnapshot(of: shot(.gaps, KeyShareSketch.oneKey), as: config) }
  func testNoPlannedMinutes() {
    assertSnapshot(of: shot(.gaps, KeyShareSketch.untimed), as: config)
  }

  func testStripMarks() {
    assertSnapshot(
      of: KeyShareStripMarksSketch(), as: .image(precision: 0.99, traits: .init(displayScale: 3)))
  }
}
