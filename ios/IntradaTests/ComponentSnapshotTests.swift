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
      sectionNameMax: LimitsView.preview.sectionNameMax, barMax: LimitsView.preview.barMax,
      plannedDurationMinMins: 1, plannedDurationMaxMins: 60, plannedDurationDefaultMins: 5)
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

  func testScoreRingKeyLabels() {
    let rings = ZStack {
      PaperBackground()
      HStack(spacing: 18) {
        ScoreRing(score: nil, size: 44, labelOverride: "C")
        ScoreRing(score: 6, size: 44, labelOverride: "G")
        ScoreRing(score: 9, size: 44, labelOverride: "B\u{266D}")
      }
      .padding(16)
    }
    assertSnapshot(of: host(rings), as: config)
  }

  func testTagChip() {
    let chips = ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: 16) {
        HStack(spacing: 8) {
          TagChip("Baroque")
          TagChip("Sight-reading", onRemove: {})
          TagChip("+2")
        }
        .padding(IntradaSpacing.card)
        .frame(maxWidth: .infinity, alignment: .leading)
        .cardSurface()
        HStack(spacing: 8) {
          TagChip("Exam piece", style: .outlined)
          TagChip("Left hand", style: .outlined, onRemove: {})
        }
      }
      .padding(16)
    }
    assertSnapshot(of: host(chips), as: config)
  }

  func testBottomSheet() {
    assertSnapshot(of: host(bottomSheet), as: config)
  }

  func testBottomSheetAtAccessibilitySize() {
    assertSnapshot(of: host(bottomSheet.dynamicTypeSize(.accessibility3)), as: config)
  }

  private var bottomSheet: some View {
    BottomSheet(
      title: "Trouble spot", confirmationLabel: "Add spot",
      leadingAction: { Button("Cancel") {} },
      content: {
        Text("Bars 12 to 16")
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.ink)
          .frame(maxWidth: .infinity, alignment: .leading)
          .padding(IntradaSpacing.card)
          .cardSurface()
          .padding(IntradaSpacing.card)
        Spacer()
      }
    )
  }

  func testFormErrorBanner() {
    let banners = ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.controlGap) {
        FormErrorBanner(message: "Title must be between 1 and 500 characters")
        FormErrorBanner(message: "Composer must be between 1 and 200 characters")
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(banners), as: config)
  }

  func testPlaceholderContent() {
    let placeholder = ZStack {
      PaperBackground()
      PlaceholderContent(
        systemImage: "books.vertical",
        message: "Pieces and exercises will live here.",
        actions: [
          .init(title: "Add piece or exercise", identifier: "library.empty.add") {},
          .init(title: "Scan a page", identifier: "library.empty.scan") {},
        ])
    }
    assertSnapshot(of: host(placeholder), as: config)
  }

  func testScreenScaffold() {
    let screen = NavigationStack {
      ScreenScaffold(
        title: "Library", subtitle: "8 pieces · 4 exercises",
        trailing: .init(label: "Add item", action: {})
      ) {
        Text("Clair de Lune")
          .font(IntradaFont.cardTitle)
          .foregroundStyle(IntradaColor.ink)
          .frame(maxWidth: .infinity, alignment: .leading)
          .padding(IntradaSpacing.card)
          .cardSurface()
          .padding(IntradaSpacing.card)
          .frame(maxHeight: .infinity, alignment: .top)
      }
    }
    assertSnapshot(of: host(screen), as: config)
  }

  func testSectionHeader() {
    assertSnapshot(of: host(sectionHeaders), as: config)
  }

  func testSectionHeaderAtAccessibilitySize() {
    assertSnapshot(of: host(sectionHeaders.dynamicTypeSize(.accessibility3)), as: config)
  }

  private var sectionHeaders: some View {
    ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        SectionHeader(title: "Recently practised")
        SectionHeader(title: "Used in", caption: "3 pieces")
        SectionHeader(title: "Last five weeks", trailing: "best week · 95m")
        SectionHeader(title: "Sections", action: .init(title: "Reorder", perform: {}))
      }
      .padding(IntradaSpacing.card)
    }
  }

  func testHairlineDivider() {
    let dividers = ZStack {
      PaperBackground()
      VStack(spacing: 0) {
        Color.clear.frame(height: 40)
        HairlineDivider()
        HStack(spacing: 0) {
          Color.clear.frame(width: 80)
          HairlineDivider(axis: .vertical)
          Color.clear.frame(width: 80)
          HairlineDivider(axis: .vertical, colour: IntradaColor.divider)
          Spacer()
        }
        .frame(height: 40)
      }
      .cardSurface()
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(dividers), as: config)
  }

  func testCard() {
    let cards = ZStack {
      PaperBackground()
      VStack(spacing: 24) {
        Color.clear.frame(height: 64).cardSurface()
        Color.clear.frame(height: 64).cardSurface().cardShadow()
      }
      .padding(24)
    }
    assertSnapshot(of: host(cards), as: config)
  }

  func testSegmentedPills() {
    let pills = ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: 24) {
        SegmentedPills(
          options: LibraryFilter.allCases, selection: .constant(.pieces), label: \.label)
        SegmentedPills(
          options: [ItemKind.piece, .exercise], selection: .constant(.exercise), label: \.label,
          font: IntradaFont.segment, layout: .fullWidthTrack)
        SegmentedPills(
          options: ["2 + 2 + 3", "2 + 3 + 2", "3 + 2 + 2"], selection: .constant("2 + 3 + 2"),
          label: { $0 }, layout: .wrappingTrack)
      }
      .padding(16)
    }
    assertSnapshot(of: host(pills), as: config)
  }

  func testPrimaryButtons() {
    let buttons = ZStack {
      PaperBackground()
      VStack(spacing: 16) {
        MarkerButton("Save session") {}
        BrandBarButton(action: {}) {
          Image(systemName: "play.fill")
          Text("Start session · 10 min")
        }
      }
      .padding(16)
    }
    assertSnapshot(of: host(buttons), as: config)
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
}
