import SharedTypes
import Testing

@testable import Intrada

/// The metronome and entry-settings sheets and the mark pills offer what the
/// core validates rather than numbers of their own (#1512, #2042), so the
/// fixture's bounds are deliberately unlike the ones shipping today.
@MainActor
struct SheetBoundsTests {
  private let limits = LimitsView(
    metreBeatsMin: 5, metreBeatsMax: 7, metreUnits: [4, 16],
    repTargetMin: 2, repTargetMax: 4, repTargetDefault: 3,
    plannedDurationMinSecs: 90, plannedDurationMaxSecs: 630, plannedDurationDefaultSecs: 480,
    sessionLengthMinMins: 15, sessionLengthMaxMins: 45, sessionLengthStepMins: 15,
    sessionLengthDefaultMins: 30,
    scoreMin: 2, scoreMax: 6,
    clickTempoStep: 2, clickTempoDefault: 96,
    clickTempoBands: [TempoBand(unit: 4, min: 40, max: 208)],
    clickMetrePresets: [], clickBars: [])

  private func clickSheet() -> ClickSheet {
    ClickSheet(click: ClickController(), bpm: 120, limits: limits)
  }

  private func entrySheet() -> EntrySettingsSheet {
    EntrySettingsSheet(entry: .previewGroupedScales, limits: limits)
  }

  @Test func theBeatsStepperOffersTheMetreRangeTheCoreAccepts() {
    #expect(clickSheet().beatsRange == 5...7)
  }

  @Test func theBeatValuePillsOfferTheUnitsTheCoreAccepts() {
    #expect(clickSheet().unitOptions == [4, 16])
  }

  @Test func theRepStepperOffersTheTargetRangeTheCoreAccepts() {
    #expect(entrySheet().repTargetRange == 2...4)
  }

  /// 90s to 630s is a minute and a half to ten and a half, and every minute the
  /// stepper offers has to be one the core would accept, so both ends round in.
  @Test func theDurationStepperOffersTheCoresSecondsAsWholeMinutes() {
    #expect(entrySheet().durationRange == 2...10)
  }

  private func scoreSelector(score: Int) -> ScoreSelector {
    ScoreSelector(score: score, range: limits.scoreRange, accessibilityLabel: "Mark") { _ in }
  }

  @Test func theMarkPillsOfferTheScoresTheCoreAccepts() {
    #expect(scoreSelector(score: 0).pillValues == [2, 3, 4, 5, 6])
  }

  @Test func theSpokenMarkIsOutOfTheCoresTopScore() {
    #expect(scoreSelector(score: 4).spokenValue == "4 of 6")
  }

  @Test func aSwipeUpFromUnmarkedLandsOnTheCoresLowestScore() {
    #expect(scoreSelector(score: 0).markAbove == 2)
  }

  @Test func aSwipeUpStopsAtTheCoresTopScore() {
    #expect(scoreSelector(score: 6).markAbove == nil)
  }

  @Test func aSwipeDownFromTheCoresLowestScoreClears() {
    #expect(scoreSelector(score: 2).markBelow == nil)
  }

  @Test func aSwipeDownWithinTheRangeStepsOneMark() {
    #expect(scoreSelector(score: 4).markBelow == 3)
  }

  @Test func anEntryWithNoTargetStartsAtTheCoresDefault() {
    #expect(EntrySettingsSheet.initialRepTarget(for: .previewGroupedScales, limits: limits) == 3)
  }

  @Test func anEntryThatAlreadyHasATargetKeepsIt() {
    #expect(
      EntrySettingsSheet.initialRepTarget(for: .previewGroupedScalesConfigured, limits: limits) == 7
    )
  }

  @Test func anEntryWithNoPlannedDurationStartsAtTheCoresDefault() {
    #expect(
      EntrySettingsSheet.initialPlannedMinutes(for: .previewGroupedScales, limits: limits) == 8)
  }

  @Test func anEntryThatAlreadyHasAPlannedDurationKeepsIt() {
    #expect(
      EntrySettingsSheet.initialPlannedMinutes(
        for: .previewGroupedScalesConfigured, limits: limits) == 6
    )
  }
}
