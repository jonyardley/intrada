import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class MinutesUsedSketchSnapshotTests: SnapshotTestCase {
  func testOver() {
    assertSnapshot(of: host(MinutesUsedBuilderSketch(fit: .over)), as: config)
  }

  func testUnder() {
    assertSnapshot(of: host(MinutesUsedBuilderSketch(fit: .under)), as: config)
  }

  func testRowStepperInline() {
    assertSnapshot(
      of: host(MinutesUsedBuilderSketch(fit: .over, stepper: .inline)), as: config)
  }

  func testRowStepperBelow() {
    assertSnapshot(
      of: host(MinutesUsedBuilderSketch(fit: .over, stepper: .below)), as: config)
  }

  func testUntimed() {
    assertSnapshot(of: host(MinutesUsedBuilderSketch(fit: .untimed)), as: config)
  }

  func testOverLargestText() {
    assertSnapshot(
      of: host(MinutesUsedBuilderSketch(fit: .over).environment(\.dynamicTypeSize, .xxxLarge)),
      as: config)
  }

  func testRowsOpenSheet() {
    assertSnapshot(
      of: host(MinutesUsedBuilderSketch(fit: .over, opensSheet: true)), as: config)
  }

  func testEntrySheetStepper() {
    assertSnapshot(of: host(MinutesUsedEntrySheetSketch()), as: config)
  }

  func testEntrySheetStepperWithFit() {
    assertSnapshot(of: host(MinutesUsedEntrySheetSketch(showsFit: true)), as: config)
  }
}
