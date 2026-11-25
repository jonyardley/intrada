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
}
