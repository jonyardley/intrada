import Testing

@testable import Intrada

@MainActor
struct StepperControlTests {
  private func stepper(_ score: Int, range: ClosedRange<Int> = 1...10) -> ScoreStepper {
    ScoreStepper(
      score: score, origin: .lastTime, range: range, accessibilityLabel: "Score for D major"
    ) { _ in }
  }

  @Test func minusStopsAtTheBottomOfTheRange() {
    #expect(!stepper(1).canLower)
    #expect(stepper(1).canRaise)
    #expect(stepper(1).stepped(by: -1) == 1)
  }

  @Test func plusStopsAtTheTopOfTheRange() {
    #expect(!stepper(10).canRaise)
    #expect(stepper(10).canLower)
    #expect(stepper(10).stepped(by: 1) == 10)
  }

  @Test func theRangeComesFromTheCaller() {
    #expect(!stepper(5, range: 1...5).canRaise)
    #expect(stepper(4, range: 1...5).stepped(by: 1) == 5)
  }

  @Test(arguments: [
    (ScoreStepper.Origin.lastTime, "5, last time"),
    (.today, "5, today"),
    (.suggested, "5, suggested"),
  ])
  func spokenValueSaysWhereTheScoreCameFrom(origin: ScoreStepper.Origin, spoken: String) {
    let stepper = ScoreStepper(
      score: 5, origin: origin, range: 1...10, accessibilityLabel: "Score for D major"
    ) { _ in }
    #expect(stepper.spokenValue == spoken)
  }

  private func length(_ minutes: Int) -> ScoreStepper {
    ScoreStepper(
      score: minutes, caption: .unit("minutes"), range: 5...120, step: 5,
      figure: IntradaFont.timer(32), typing: true,
      accessibilityLabel: "Session length", accessibilityValue: "\(minutes) minutes"
    ) { _ in }
  }

  @Test func aLengthStepsByTheCallersStepAndAddsWithoutSnapping() {
    #expect(length(20).stepped(by: 1) == 25)
    #expect(length(20).stepped(by: -1) == 15)
    #expect(length(17).stepped(by: 1) == 22)
  }

  @Test func aLengthStepStopsAtTheRangeEnds() {
    #expect(!length(5).canLower)
    #expect(length(7).stepped(by: -1) == 5)
    #expect(!length(120).canRaise)
    #expect(length(118).stepped(by: 1) == 120)
  }

  @Test func theCallersSpokenValueReplacesTheCaption() {
    #expect(length(20).spokenValue == "20 minutes")
    let unspoken = ScoreStepper(
      score: 20, caption: .unit("minutes"), range: 5...120, accessibilityLabel: "Session length"
    ) { _ in }
    #expect(unspoken.spokenValue == "20, minutes")
  }

  @Test(
    arguments: [
      ("17", 17),
      ("0", 5),
      ("3", 5),
      ("240", 120),
      (" 45 ", 45),
      ("", nil),
      ("abc", nil),
    ] as [(String, Int?)])
  func aTypedLengthIsClampedToTheRange(typed: String, expected: Int?) {
    #expect(length(20).typedValue(typed) == expected)
  }

  @Test(
    arguments: [
      ("17", 1, 22),
      ("", 1, 25),
      ("118", 1, 120),
      ("3", -1, 5),
    ] as [(String, Int, Int)])
  func aRingTapWhileTypingStepsFromTheDraftOrTheNumberBefore(
    draft: String, steps: Int, expected: Int
  ) {
    #expect(length(20).stepped(by: steps, fromDraft: draft) == expected)
  }
}
