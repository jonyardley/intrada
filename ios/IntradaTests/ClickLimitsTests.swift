import CoreGraphics
import SharedTypes
import Testing
import XCTest

@testable import Intrada

/// The click reads the core's band and bars (#2225). The fixture's click
/// fields are deliberately unlike the core's, so a lookup that ignored them
/// and fell back on numbers of its own would fail here.
@MainActor
struct ClickLimitsTests {
  private let limits: LimitsView = {
    var limits = LimitsView.preview
    limits.clickTempoStep = 5
    limits.clickTempoDefault = 100
    limits.clickTempoBands = [
      TempoBand(unit: 4, min: 50, max: 150), TempoBand(unit: 8, min: 100, max: 300),
    ]
    limits.clickBars = [
      ClickBarOption(
        beats: 5, groups: nil,
        presets: [ClickPresetOption(preset: .everyBeat, sounding: 0b11111)]),
      ClickBarOption(
        beats: 5, groups: [3, 2],
        presets: [ClickPresetOption(preset: .groupStarts, sounding: 0b01001)]),
      ClickBarOption(beats: 5, groups: [2, 3], presets: []),
      ClickBarOption(beats: 6, groups: [3, 3], presets: []),
    ]
    return limits
  }()

  @Test func theBandIsTheOneForTheBarsBeatValue() {
    #expect(limits.clickBand(unit: 4) == 50...150)
    #expect(limits.clickBand(unit: 8) == 100...300)
  }

  @Test func aBeatValueWithNoBandHoldsTheCoresDefault() {
    #expect(limits.clickBand(unit: 2) == 100...100)
  }

  @Test func aTempoOutsideTheBandArrivesClamped() {
    #expect(limits.clampClickTempo(30, unit: 4) == 50)
    #expect(limits.clampClickTempo(240, unit: 4) == 150)
    #expect(limits.clampClickTempo(240, unit: 8) == 240)
  }

  @Test func theStepIsTheCores() {
    #expect(limits.clickStep == 5)
  }

  @Test func aBarsPatternsMatchItsGroupingNotJustItsBeats() {
    #expect(
      limits.clickPresets(for: Metre(beats: 5, unit: 8, groups: [3, 2])).map(\.preset) == [
        .groupStarts
      ])
    #expect(
      limits.clickPresets(for: Metre(beats: 5, unit: 4, groups: nil)).map(\.preset) == [.everyBeat])
    #expect(limits.clickPresets(for: Metre(beats: 7, unit: 4, groups: nil)).isEmpty)
  }

  @Test func theGroupingsAreTheGroupedRowsForThatBeatCount() {
    #expect(limits.clickGroupings(beats: 5) == [[3, 2], [2, 3]])
    #expect(limits.clickGroupings(beats: 4).isEmpty)
  }
}

/// The drag-to-BPM maths (#1823).
@MainActor
struct ClickDragTests {
  private let points = ClickControl.dragPointsPerStep

  private func bpm(_ translation: CGFloat, anchor: Int = 96) -> Int {
    ClickControl.bpm(fromDragTranslation: translation, anchor: anchor, step: 2, band: 40...208)
  }

  @Test func aDragShorterThanOneStepChangesNothing() {
    #expect(bpm(-3) == 96)
    #expect(bpm(3) == 96)
  }

  @Test func draggingUpOneStepIsFaster() {
    #expect(bpm(-points) == 98)
  }

  @Test func draggingDownOneStepIsSlower() {
    #expect(bpm(points) == 94)
  }

  @Test func aLongDragStopsAtTheEndsOfTheBand() {
    #expect(bpm(-2000) == 208)
    #expect(bpm(2000) == 40)
    #expect(bpm(-points, anchor: 208) == 208)
    #expect(bpm(points, anchor: 40) == 40)
  }
}

/// The shell half of the tempo evidence contract (#1420): if `userSet` ever
/// stops tracking the stepper, every tempo becomes unevidenced and the trend
/// goes silently empty: the #846 failure mode with no other guard on it.
@MainActor
final class TrackedTempoTests: XCTestCase {
  func testAPreFillIsNotUserSet() {
    XCTAssertFalse(
      TrackedTempo(startingBpm: 96, band: 40...208).userSet,
      "the sheet opening at a pre-filled number is not the user setting one")
  }

  func testSettingMarksItUserSet() {
    var tempo = TrackedTempo(startingBpm: 96, band: 40...208)
    tempo.set(120)
    XCTAssertEqual(tempo.bpm, 120)
    XCTAssertTrue(tempo.userSet)
  }

  func testSettingBackToTheStartingValueStillCounts() {
    var tempo = TrackedTempo(startingBpm: 96, band: 40...208)
    tempo.set(98)
    tempo.set(96)
    XCTAssertTrue(
      tempo.userSet, "stepping up and back is still the user considering the number")
  }

  func testAnOutOfRangeStartClampsWithoutCountingAsUserSet() {
    let tempo = TrackedTempo(startingBpm: 400, band: 40...208)
    XCTAssertEqual(tempo.bpm, 208)
    XCTAssertFalse(tempo.userSet, "clamping is the app tidying up, not the user choosing")
  }
}
