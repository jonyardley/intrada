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
