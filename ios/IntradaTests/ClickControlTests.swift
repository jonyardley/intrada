import Testing

@testable import Intrada

@MainActor
struct ClickControlTests {
  private func control(
    bpm: Int = 96, running: Bool = false, unavailable: Bool = false, atSeed: Bool = true
  ) -> ClickControl {
    ClickControl(
      bpm: bpm, isRunning: running, unavailable: unavailable, atSeededTempo: atSeed,
      targetDisplay: "Andante · ♩ = 66", targetSpoken: "Andante, 66 beats per minute",
      onToggle: {}, onStep: { _ in })
  }

  private func withoutTarget(bpm: Int = 96, running: Bool = false, atSeed: Bool = true)
    -> ClickControl
  {
    ClickControl(
      bpm: bpm, isRunning: running, unavailable: false, atSeededTempo: atSeed,
      targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in })
  }

  @Test func atRestTheRowIsStillTheItemsDeclaredTempo() {
    #expect(control(bpm: 66).readout == "Andante · ♩ = 66")
  }

  @Test func aSoundingClickShowsTheTempoItIsKeepingAndNoMarking() {
    #expect(control(bpm: 72, running: true, atSeed: false).readout == "♩ = 72")
  }

  @Test func aStoppedClickStillShowsWhereItWasStepped() {
    #expect(control(bpm: 76, atSeed: false).readout == "♩ = 76")
    #expect(control(bpm: 76, atSeed: false).spokenValue == "76 beats per minute")
  }

  @Test func anItemWithNoDeclaredTempoNamesTheClickInstead() {
    #expect(withoutTarget().readout == "Click")
    #expect(withoutTarget(bpm: 96, running: true, atSeed: true).readout == "♩ = 96")
  }

  /// A marking with no number is not a tempo the click can play, so the player
  /// hands it `nil` rather than a row that reads "Andante" and sounds 96.
  @Test func aMarkingWithNoBpmIsNotATargetTheClickCanSpeakFor() {
    #expect(TempoFormatting.display(marking: "Andante", bpm: nil) == "Andante")
    #expect(withoutTarget().readout == "Click")
    #expect(withoutTarget().spokenValue == "no tempo set")
  }

  @Test func aClickThatCouldNotStartSaysSoInPlace() {
    #expect(control(unavailable: true).readout == "Click unavailable")
  }

  @Test func spokenTempoSpellsTheBpmOut() {
    #expect(control(bpm: 66).spokenValue == "Andante, 66 beats per minute")
    #expect(control(bpm: 72, running: true, atSeed: false).spokenValue == "72 beats per minute")
    #expect(!control(bpm: 66).spokenValue.contains("♩"))
  }

  @Test func spokenTempoNamesTheAbsenceOfATarget() {
    #expect(withoutTarget().spokenValue == "no tempo set")
  }

  @Test func spokenTempoCarriesTheFailure() {
    #expect(control(unavailable: true).spokenValue == "unavailable")
  }
}

@MainActor
struct ClickControllerTests {
  @Test func theClickSeedsFromTheItemsDeclaredTempo() {
    #expect(ClickController.seedBpm(from: 66) == 66)
  }

  @Test func anItemWithNoTempoSeedsTheNeutralDefault() {
    #expect(ClickController.seedBpm(from: nil) == TempoScale.defaultBpm)
  }

  /// The core validates 1 to 400 BPM; the click's steppers only reach 40 to 208,
  /// so a target outside that has to arrive clamped or the row shows a tempo
  /// the steppers cannot get back to.
  @Test func aTargetOutsideTheStepperRangeArrivesClamped() {
    #expect(ClickController.seedBpm(from: 30) == TempoScale.range.lowerBound)
    #expect(ClickController.seedBpm(from: 320) == TempoScale.range.upperBound)
  }

  @Test func movingToANewItemReseedsToThatItemsTempo() {
    let click = ClickController()

    click.reseed(target: 66)
    #expect(click.bpm == 66)

    click.reseed(target: nil)
    #expect(click.bpm == TempoScale.defaultBpm)
  }

  @Test func steppingMovesByTheSharedTempoStepAndStopsAtTheEnds() {
    let click = ClickController()
    click.reseed(target: 66)

    click.step(by: TempoScale.step)
    #expect(click.bpm == 68)
    click.step(by: -TempoScale.step)
    #expect(click.bpm == 66)

    click.reseed(target: UInt16(TempoScale.range.upperBound))
    click.step(by: TempoScale.step)
    #expect(click.bpm == TempoScale.range.upperBound)
  }

  @Test func steppingOffTheItemsTempoAndBackAgainIsNoticed() {
    let click = ClickController()
    click.reseed(target: 66)
    #expect(click.isAtSeededTempo)

    click.step(by: TempoScale.step)
    #expect(!click.isAtSeededTempo)

    click.step(by: -TempoScale.step)
    #expect(click.isAtSeededTempo)
  }

  @Test func aNewItemPutsTheRowBackOnItsOwnTempo() {
    let click = ClickController()
    click.reseed(target: 66)
    click.step(by: TempoScale.step)

    click.reseed(target: 120)
    #expect(click.bpm == 120)
    #expect(click.isAtSeededTempo)
  }
}
