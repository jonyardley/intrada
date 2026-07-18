import SharedTypes
import XCTest

@testable import Intrada

final class TempoStepperTests: XCTestCase {
  func testClampWithinRangeIsUnchanged() {
    XCTAssertEqual(TempoStepper.clamp(96), 96)
    XCTAssertEqual(TempoStepper.clamp(TempoStepper.range.lowerBound), 40)
    XCTAssertEqual(TempoStepper.clamp(TempoStepper.range.upperBound), 208)
  }

  func testClampBelowRangeSnapsToLowerBound() {
    XCTAssertEqual(TempoStepper.clamp(30), 40, "a Grave target below the UI range clamps up")
  }

  func testClampAboveRangeSnapsToUpperBound() {
    XCTAssertEqual(TempoStepper.clamp(220), 208, "a Presto target above the UI range clamps down")
  }

  func testIncrementFromOutOfRangeMovesTowardRangeNotAway() {
    XCTAssertEqual(TempoStepper.stepped(from: 30, by: 2), 40)
  }

  func testDecrementFromOutOfRangeMovesTowardRangeNotAway() {
    XCTAssertEqual(TempoStepper.stepped(from: 220, by: -2), 208)
  }

  func testStepWithinRangeMovesByStep() {
    XCTAssertEqual(TempoStepper.stepped(from: 96, by: 2), 98)
    XCTAssertEqual(TempoStepper.stepped(from: 96, by: -2), 94)
  }

  func testStepClampsAtTheBoundary() {
    XCTAssertEqual(TempoStepper.stepped(from: 208, by: 2), 208)
    XCTAssertEqual(TempoStepper.stepped(from: 40, by: -2), 40)
  }
}

final class ReflectionSheetTempoResolutionTests: XCTestCase {
  func testNoTempoTargetResolvesToNilRegardlessOfStepperValue() {
    XCTAssertNil(ReflectionSheet.resolvedAchievedTempo(tempoTarget: nil, current: 96))
    XCTAssertNil(ReflectionSheet.resolvedAchievedTempo(tempoTarget: nil, current: 0))
  }

  func testTempoTargetPresentResolvesToTheCurrentStepperValue() {
    XCTAssertEqual(
      ReflectionSheet.resolvedAchievedTempo(tempoTarget: 96, current: 96), 96,
      "untouched stepper reads as \"played at target\"")
    XCTAssertEqual(ReflectionSheet.resolvedAchievedTempo(tempoTarget: 96, current: 102), 102)
  }
}

@MainActor
final class ReflectionSheetStepSelectionTests: XCTestCase {
  private func step(_ id: String, _ position: UInt64) -> VariantView {
    VariantView(
      id: id, label: id, position: position, latestScore: nil, scoreHistory: [], isSolid: false,
      isCurrent: false)
  }

  func testNoStepsResolvesToNil() {
    XCTAssertNil(ReflectionSheet.initialVariantId(currentVariantId: nil, variants: []))
  }

  func testCurrentVariantIdWinsWhenPresent() {
    let variants = [step("s1", 0), step("s2", 1)]
    XCTAssertEqual(
      ReflectionSheet.initialVariantId(currentVariantId: "s2", variants: variants), "s2")
  }

  func testFallsBackToFirstStepByPositionWhenNoCurrentVariant() {
    let variants = [step("s1", 0), step("s2", 1)]
    XCTAssertEqual(
      ReflectionSheet.initialVariantId(currentVariantId: nil, variants: variants), "s1",
      "never leaves the picker unset — defaults to the first step")
  }
}

@MainActor
final class AddStepsSheetTests: XCTestCase {
  func testEmptyArrayTrimsToEmpty() {
    XCTAssertEqual(AddStepsSheet.trimmedLabels([]), [])
  }

  func testAllWhitespaceRowsAreDropped() {
    XCTAssertEqual(AddStepsSheet.trimmedLabels(["", "  ", "\n"]), [])
  }

  func testMixedBlankAndPopulatedRowsKeepsOnlyPopulated() {
    XCTAssertEqual(AddStepsSheet.trimmedLabels(["C", "", "G", "  "]), ["C", "G"])
  }

  func testLeadingAndTrailingWhitespaceIsTrimmed() {
    XCTAssertEqual(AddStepsSheet.trimmedLabels(["  C  ", " G"]), ["C", "G"])
  }
}
