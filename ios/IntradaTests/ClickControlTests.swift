import SharedTypes
import SwiftUI
import Testing

@testable import Intrada

@MainActor
struct ClickControlTests {
  private func control(
    bpm: Int = 96, running: Bool = false, unavailable: Bool = false, atSeed: Bool = true
  ) -> ClickControl {
    ClickControl(
      bpm: bpm, step: 2, band: 40...208, isRunning: running, unavailable: unavailable,
      atSeededTempo: atSeed,
      targetDisplay: "Andante · ♩ = 66", targetSpoken: "Andante, 66 beats per minute",
      onToggle: {}, onStep: { _ in }, onDragChange: { _ in })
  }

  private func withoutTarget(bpm: Int = 96, running: Bool = false, atSeed: Bool = true)
    -> ClickControl
  {
    ClickControl(
      bpm: bpm, step: 2, band: 40...208, isRunning: running, unavailable: false,
      atSeededTempo: atSeed,
      targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
      onDragChange: { _ in })
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

  @Test func anItemWithNoDeclaredTempoNamesTheMetronomeInstead() {
    #expect(withoutTarget().readout == "Metronome")
    #expect(withoutTarget(bpm: 96, running: true, atSeed: true).readout == "♩ = 96")
  }

  /// A marking with no number is not a tempo the click can play, so the player
  /// hands it `nil` rather than a row that reads "Andante" and sounds 96.
  @Test func aMarkingWithNoBpmIsNotATargetTheClickCanSpeakFor() {
    #expect(TempoFormatting.display(marking: "Andante", bpm: nil) == "Andante")
    #expect(withoutTarget().readout == "Metronome")
    #expect(withoutTarget().spokenValue == "96 beats per minute")
  }

  @Test func aClickThatCouldNotStartSaysSoInPlace() {
    #expect(control(unavailable: true).readout == "Metronome unavailable")
  }

  @Test func spokenTempoSpellsTheBpmOut() {
    #expect(control(bpm: 66).spokenValue == "Andante, 66 beats per minute")
    #expect(control(bpm: 72, running: true, atSeed: false).spokenValue == "72 beats per minute")
    #expect(!control(bpm: 66).spokenValue.contains("♩"))
  }

  // Reached by a VoiceOver swipe up and back to the seed (#2021).
  @Test func withNoTargetTheSpokenValueIsTheTempoTheNextTapPlays() {
    #expect(withoutTarget(bpm: 88).spokenValue == "88 beats per minute")
  }

  @Test func spokenTempoCarriesTheFailure() {
    #expect(control(unavailable: true).spokenValue == "unavailable")
  }

  @Test func adjustStepsByOneTempoStepEachWay() {
    var steps: [Int] = []
    let control = ClickControl(
      bpm: 96, step: 4, band: 40...208, isRunning: false, unavailable: false, atSeededTempo: true,
      targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { steps.append($0) },
      onDragChange: { _ in })
    control.adjust(.increment)
    control.adjust(.decrement)
    #expect(steps == [4, -4], "the step the core sends, not one of the row's own")
  }

  /// A drag commit inside the click engine's own lead-in cancels the pulse
  /// it just scheduled, so the click never sounds for the whole drag (#1823).
  @Test func theDragThrottleNeverCommitsFasterThanTheEngineCanSound() {
    #expect(ClickControl.commitInterval >= .milliseconds(Int(ClickEngine.leadInSeconds * 1000)))
  }
}

/// The seed is the core's answer (#2225); these check the controller plays it
/// and keeps to the core's band, not what the answer is.
@MainActor
private func seededClick(
  bpm: UInt16 = 66, metre: Metre = Metre(beats: 4, unit: 4, groups: nil),
  soundsTarget: Bool = true, sounding: UInt16 = 0b1111,
  presets: [ClickPresetOption]? = nil
) -> ClickController {
  var active = ActiveSessionView.previewActive
  active.clickSeedBpm = bpm
  active.clickSeedMetre = metre
  active.clickSeedSoundsTarget = soundsTarget
  active.clickSeedPresets = presets ?? LimitsView.preview.clickPresets(for: metre)
  active.currentClickSounding = sounding
  let click = ClickController()
  click.reseed(from: active, limits: .preview)
  return click
}

@MainActor
struct ClickControllerTests {
  @Test func theClickOpensOnTheCoresBarTempoAndBeats() {
    let sevenEight = Metre(beats: 7, unit: 8, groups: [3, 2, 2])
    let click = seededClick(bpm: 168, metre: sevenEight, soundsTarget: true, sounding: 0b0101001)
    #expect(click.bpm == 168)
    #expect(click.metre == sevenEight)
    #expect(click.sounding == 0b0101001)
    #expect(click.soundsTarget)
    #expect(click.isAtSeededTempo)
  }

  @Test func aBarInAnotherUnitLeavesTheSeed() {
    let click = seededClick(bpm: 96)
    click.setMetre(Metre(beats: 6, unit: 8, groups: nil))
    #expect(click.bpm == 96)
    #expect(!click.isAtSeededTempo)
  }

  @Test func steppingMovesByTheCoresStepAndStopsAtTheEndsOfItsBand() {
    let click = seededClick(bpm: 66)
    let step = LimitsView.preview.clickStep
    let top = LimitsView.preview.clickBand(unit: 4).upperBound

    click.step(by: step)
    #expect(click.bpm == 66 + step)
    click.step(by: -step)
    #expect(click.bpm == 66)

    let atTop = seededClick(bpm: UInt16(top))
    atTop.step(by: step)
    #expect(atTop.bpm == top)
  }

  @Test func steppingOffTheItemsTempoAndBackAgainIsNoticed() {
    let click = seededClick(bpm: 66)
    click.step(by: click.tempoStep)
    #expect(!click.isAtSeededTempo)

    click.step(by: -click.tempoStep)
    #expect(click.isAtSeededTempo)
  }

  @Test func aNewItemPutsTheRowBackOnItsOwnTempo() {
    let click = seededClick(bpm: 66)
    click.step(by: click.tempoStep)

    var next = ActiveSessionView.previewActive
    next.clickSeedBpm = 120
    click.reseed(from: next, limits: .preview)
    #expect(click.bpm == 120)
    #expect(click.isAtSeededTempo)
  }

  @Test func anItemStartsOnTheCoresBeatsAndABarChangeTakesOver() {
    let click = seededClick(sounding: 0b1010)
    #expect(click.sounding == 0b1010)
    #expect(click.clickState == nil)

    click.setMetre(Metre(beats: 5, unit: 4, groups: nil))
    #expect(click.sounding == 0b11111, "a new bar starts on the core's every-beat pattern")
  }

  /// 4/8 counted 2 + 2 is a valid bar the core's table does not list, so its
  /// patterns arrive with the seed rather than from the table.
  @Test func theItemsOwnBarOffersThePatternsSentWithIt() {
    let twoTwo = Metre(beats: 4, unit: 8, groups: [2, 2])
    let sent = [
      ClickPresetOption(preset: .everyBeat, sounding: 0b1111),
      ClickPresetOption(preset: .groupStarts, sounding: 0b0101),
    ]
    let click = seededClick(bpm: 240, metre: twoTwo, presets: sent)
    #expect(click.presets == sent)

    click.apply(.groupStarts)
    #expect(click.sounding == 0b0101)
    #expect(click.matchingPreset == .groupStarts)

    click.setMetre(Metre(beats: 3, unit: 4, groups: nil))
    #expect(click.presets == LimitsView.preview.clickPresets(for: click.metre))
  }

  @Test func aPatternTheBarDoesNotOfferChangesNothing() {
    let click = seededClick(
      metre: Metre(beats: 7, unit: 8, groups: [3, 2, 2]), sounding: 0b111_1111)
    click.apply(.backbeat)
    #expect(click.sounding == 0b111_1111)
    #expect(click.clickState == nil)
  }

  @Test func aHandToggledPatternMatchesNoPreset() {
    let click = seededClick()
    click.toggleBeat(1)
    #expect(click.sounding == 0b1101)
    #expect(click.matchingPreset == nil)
  }

  @Test func enteringTheBackgroundArmsATimedStopAndReturningDisarmsIt() {
    let click = ClickController()

    click.enteredBackground()
    #expect(click.backgroundStopArmed)

    click.enteredForeground()
    #expect(!click.backgroundStopArmed)
  }

  @Test func theTimedStopWaitsForTheGraceAndThenFires() async throws {
    let click = ClickController()
    click.backgroundGrace = .milliseconds(200)

    click.enteredBackground()
    try await Task.sleep(for: .milliseconds(20))
    #expect(click.backgroundStopArmed)

    try await Task.sleep(for: .milliseconds(500))
    #expect(!click.backgroundStopArmed)
  }

  /// An orphaned task would fire into the next arming and silence a restarted click.
  @Test func returningToTheForegroundCancelsTheOldTaskNotJustItsHandle() async throws {
    let click = ClickController()
    click.backgroundGrace = .milliseconds(20)
    click.enteredBackground()
    click.enteredForeground()

    click.backgroundGrace = .seconds(600)
    click.enteredBackground()
    try await Task.sleep(for: .milliseconds(300))

    #expect(click.backgroundStopArmed)
  }

  @Test func aTapToStopCancelsTheOldTaskNotJustItsHandle() async throws {
    let click = ClickController()
    click.backgroundGrace = .milliseconds(20)
    click.enteredBackground()
    click.stop()

    click.backgroundGrace = .seconds(600)
    click.enteredBackground()
    try await Task.sleep(for: .milliseconds(300))

    #expect(click.backgroundStopArmed)
  }

  @Test func aTapToStopDisarmsTheTimedStop() {
    let click = ClickController()

    click.enteredBackground()
    click.stop()

    #expect(!click.backgroundStopArmed)
  }
}

@MainActor
struct ClickBarTests {
  private let common = Metre(beats: 4, unit: 4, groups: nil)
  private let sevenEight = Metre(beats: 7, unit: 8, groups: [3, 2, 2])

  /// The controller seeds the piece's metre, sounds every beat, and never lets
  /// the last sounding beat go silent.
  @Test func theControllerSeedsTheBarAndKeepsOneBeatSounding() {
    let click = seededClick(bpm: 168, metre: sevenEight, sounding: 0b1111111)
    #expect(click.metre == sevenEight)
    #expect(click.sounding == 0b1111111)
    #expect(click.clickState == nil, "an untouched click asserts no metre")

    click.toggleBeat(1)
    click.toggleBeat(1)
    #expect(
      click.clickState == ClickState(metre: sevenEight, sounding: 0b1111111),
      "a click the player has touched reports the bar it was set to")

    click.apply(.downbeat)
    #expect(click.sounding == 1)
    click.toggleBeat(0)
    #expect(click.sounding == 1, "a click that sounds nothing is not a click")
    click.toggleBeat(3)
    #expect(click.sounding == 0b1001)

    click.setMetre(Metre(beats: 3, unit: 4, groups: nil))
    #expect(click.sounding == 0b111, "a new bar starts with every beat sounding")
  }

  @Test func theBarLineSpeaksTheBeatsThatSound() {
    let all = ClickBarLine(metre: common, sounding: 0b1111, currentBeat: nil, onTap: {})
    #expect(all.spokenValue == "4 crotchet beats, metronome on every beat")
    let four = ClickBarLine(metre: common, sounding: 0b1000, currentBeat: nil, onTap: {})
    #expect(four.spokenValue == "4 crotchet beats, metronome on beat 4")
    let groups = ClickBarLine(metre: sevenEight, sounding: 0b0101001, currentBeat: nil, onTap: {})
    #expect(groups.spokenValue == "7 quaver beats, metronome on beats 1, 4, 6")
  }

  @Test func aQuaverClickStepsPastTheCrotchetCeiling() {
    let click = seededClick(
      bpm: 240, metre: Metre(beats: 6, unit: 8, groups: [3, 3]), sounding: 0b111111)
    #expect(click.bpm == 240)

    click.step(by: click.tempoStep)
    #expect(click.bpm == 240 + click.tempoStep)
  }

  @Test func changingTheBeatValueKeepsTheNumberTheBandAllows() {
    let click = seededClick(bpm: 132)

    click.setMetre(Metre(beats: 6, unit: 8, groups: [3, 3]))
    #expect(click.bpm == 132, "the pulse does not change because the beat was renamed")

    click.setMetre(Metre(beats: 2, unit: 2, groups: nil))
    #expect(
      click.bpm == LimitsView.preview.clickBand(unit: 2).upperBound,
      "a minim beat stops at the top of the core's minim band")
  }

  @Test func anUngroupedBarWiderThanTheSheetIsSplitIntoRows() {
    let rows = ClickSheet.gridRows(Metre(beats: 12, unit: 8, groups: nil))
    #expect(rows.map(\.count) == [6, 6])
    #expect(rows.flatMap { $0 } == Array(0..<12), "every beat still has a cell, in order")

    #expect(
      ClickSheet.gridRows(Metre(beats: 7, unit: 8, groups: [3, 2, 2])).map(\.count) == [3, 2, 2],
      "a declared grouping reads as the shape of the bar")
    #expect(
      ClickSheet.gridRows(Metre(beats: 6, unit: 8, groups: [3, 3])).map(\.count) == [6],
      "a bar that fits stays on one row")
  }
}

@MainActor
struct ReflectionSheetHeadingTests {
  @Test func aMeasuredItemShowsItsTime() {
    #expect(ReflectionSheet.heading(elapsedDisplay: "7:00") == "Item complete · 7:00")
  }

  @Test func anUnmeasuredItemShowsNoTime() {
    #expect(ReflectionSheet.heading(elapsedDisplay: nil) == "Item complete")
  }
}
