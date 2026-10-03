import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class ComponentSnapshotTests: SnapshotTestCase {
  func testScoreRing() {
    let rings = ZStack {
      PaperBackground()
      HStack(spacing: 18) {
        ScoreRing(score: nil)
        ForEach([1, 4, 7, 10], id: \.self) { ScoreRing(score: $0) }
      }
      .padding(16)
    }
    assertSnapshot(of: host(rings), as: config)
  }

  func testScoreRingHero() {
    let hero = ZStack {
      PaperBackground()
      HStack(spacing: 24) {
        ScoreRing(score: 7, size: 132, showsScale: true)
        ScoreRing(score: nil, size: 132, showsScale: true)
      }
      .padding(16)
    }
    assertSnapshot(of: host(hero), as: config)
  }

  func testScoreRingFollowsTheCoreLimits() {
    let limits = LimitsView(
      metreBeatsMin: 1, metreBeatsMax: 12, metreUnits: [4],
      repTargetMin: 1, repTargetMax: 10, repTargetDefault: 3,
      plannedDurationMinSecs: 60, plannedDurationMaxSecs: 3600, plannedDurationDefaultSecs: 300,
      sessionLengthMinMins: 5, sessionLengthMaxMins: 120, sessionLengthStepMins: 5,
      sessionLengthDefaultMins: 30,
      scoreMin: 1, scoreMax: 5,
      clickTempoStep: 2, clickTempoDefault: 96,
      clickTempoBands: [TempoBand(unit: 4, min: 40, max: 208)],
      clickMetrePresets: LimitsView.preview.clickMetrePresets,
      clickBars: LimitsView.preview.clickBars,
      sectionNameMax: LimitsView.preview.sectionNameMax, barMax: LimitsView.preview.barMax)
    let rings = ZStack {
      PaperBackground()
      HStack(spacing: 24) {
        ScoreRing(score: 4, size: 132, showsScale: true)
        ScoreRing(score: 9, size: 132, showsScale: true)
      }
      .environment(\.scoreRange, limits.scoreRange)
      .padding(16)
    }
    assertSnapshot(of: host(rings), as: config)
  }

  func testAddRowButtonVariants() {
    let buttons = ZStack {
      PaperBackground()
      VStack(spacing: 16) {
        AddRowButton(title: "Add a related exercise") {}
        AddRowButton(title: "Add a related exercise", style: .plain) {}
      }
      .padding(16)
    }
    assertSnapshot(of: host(buttons), as: config)
  }

  // The preview week the screens read ends on today, so this one is read mid-week (#2144).
  func testWeekStripDimsDaysStillToCome() {
    let week = PracticeWeekView.previewMidWeek
    let strip = ZStack {
      PaperBackground()
      WeekStrip(days: week.days, selected: .constant(week.days[Int(week.openingDay)].date))
        .padding(IntradaSpacing.card)
    }
    assertSnapshot(of: host(strip), as: config)
  }

  // The eyebrow must wrap between words beside the Edit button, never inside one (#1781).
  func testSectionHeaderWithActionAccessibilitySize() {
    let headers = ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        SectionHeader(
          title: "Chord chart",
          action: .init(title: "Edit", accessibilityLabel: "Edit chord chart", perform: {}))
        SectionHeader(
          title: "Related exercises", caption: "3", captionAccessibilityHidden: true,
          action: .init(title: "Edit", accessibilityLabel: "Edit related exercises", perform: {}))
      }
      .padding(IntradaSpacing.card)
    }
    .dynamicTypeSize(.accessibility5)
    assertSnapshot(of: host(headers), as: config)
  }
}
