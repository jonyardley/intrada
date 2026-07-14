import XCTest

@testable import Intrada

final class TempoStepperTests: XCTestCase {
  // ── Clamp is symmetric: a tap can only move a value toward the valid
  // range, never past it in the direction it was already out of range
  // (e.g. a Presto target at 220 must not jump *down* to 208 on an
  // increment tap) ────────────────────────────────────────────────────

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
    // A value already below range must not overshoot further away from the
    // range on an increment — it should land at the clamp, not `value + step`.
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
  // ── The onSave payload: whether an achieved-tempo write happens at all
  // hinges on this one branch, so it's pulled into a pure, directly
  // testable function rather than only living inside a button closure ──

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
