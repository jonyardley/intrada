import SharedTypes
import XCTest

@testable import Intrada

@MainActor
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

/// The shell half of the tempo evidence contract (#1420): if `userSet` ever
/// stops tracking the stepper, every tempo becomes unevidenced and the trend
/// goes silently empty — the #846 failure mode with no other guard on it.
@MainActor
final class TrackedTempoTests: XCTestCase {
  func testAPreFillIsNotUserSet() {
    XCTAssertFalse(
      TrackedTempo(startingBpm: 96).userSet,
      "the sheet opening at a pre-filled number is not the user setting one")
  }

  func testSettingMarksItUserSet() {
    var tempo = TrackedTempo(startingBpm: 96)
    tempo.set(120)
    XCTAssertEqual(tempo.bpm, 120)
    XCTAssertTrue(tempo.userSet)
  }

  func testSettingBackToTheStartingValueStillCounts() {
    var tempo = TrackedTempo(startingBpm: 96)
    tempo.set(98)
    tempo.set(96)
    XCTAssertTrue(
      tempo.userSet, "stepping up and back is still the user considering the number")
  }

  func testAnOutOfRangeStartClampsWithoutCountingAsUserSet() {
    let tempo = TrackedTempo(startingBpm: 400)
    XCTAssertEqual(tempo.bpm, TempoStepper.range.upperBound)
    XCTAssertFalse(tempo.userSet, "clamping is the app tidying up, not the user choosing")
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
