import SharedTypes
import Testing

@testable import Intrada

/// The metronome and entry-settings sheets offer what the core validates rather
/// than numbers of their own (#1512), so the fixture's bounds are deliberately
/// unlike the ones shipping today.
@MainActor
struct SheetBoundsTests {
  private let limits = LimitsView(
    metreBeatsMin: 5, metreBeatsMax: 7, metreUnits: [4, 16],
    repTargetMin: 2, repTargetMax: 4, repTargetDefault: 3,
    plannedDurationMinSecs: 90, plannedDurationMaxSecs: 630)

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

  @Test func anEntryWithNoTargetStartsAtTheCoresDefault() {
    #expect(EntrySettingsSheet.initialRepTarget(for: .previewGroupedScales, limits: limits) == 3)
  }

  @Test func anEntryThatAlreadyHasATargetKeepsIt() {
    #expect(
      EntrySettingsSheet.initialRepTarget(for: .previewGroupedScalesConfigured, limits: limits) == 7
    )
  }
}
