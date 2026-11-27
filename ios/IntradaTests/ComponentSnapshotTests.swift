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
      plannedDurationMinSecs: 60, plannedDurationMaxSecs: 3600, plannedDurationDefaultSecs: 300,
      sessionLengthMinMins: 5, sessionLengthMaxMins: 120, sessionLengthStepMins: 5,
      sessionLengthDefaultMins: 30,
      scoreMin: 1, scoreMax: 5,
      clickTempoStep: 2, clickTempoDefault: 96,
      clickTempoBands: [TempoBand(unit: 4, min: 40, max: 208)],
      clickMetrePresets: LimitsView.preview.clickMetrePresets,
      clickBars: LimitsView.preview.clickBars,
      plannedDurationMinMins: 1, plannedDurationMaxMins: 60, plannedDurationDefaultMins: 5,
      playNoteMaxChars: 500)
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

  private var bottomSheet: some View {
    BottomSheet(
      title: "Keys",
      leadingAction: { Button("Cancel") {} },
      content: {
        Text("C major, G major, D major")
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
          .init(title: "Add piece or exercise", identifier: "library.empty.add") {}
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

  private var sectionHeaders: some View {
    ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        SectionHeader(title: "Recently practised")
        SectionHeader(title: "Used in", caption: "3 pieces")
        SectionHeader(title: "Last five weeks", trailing: "best week · 95m")
        SectionHeader(title: "Chord chart", action: .init(title: "Edit", perform: {}))
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

  func testSegmentedProgress() {
    let strips = ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.section) {
        SegmentedProgress(types: [.piece, .exercise, .exercise, .piece, .piece], filled: 2)
        SegmentedProgress(count: 12, filled: 7, label: "7 of 12 solid")
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(strips), as: config)
  }

  func testSectionTitle() {
    let titles = ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        SectionTitle("Recent sessions")
        HeroPanel {
          SectionTitle("Then", tint: IntradaColor.onAccent.opacity(IntradaOpacity.secondary))
        }
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(titles), as: config)
  }

  func testFieldCard() {
    let cards = ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.cardCompact) {
        FieldCard("Beats in the bar") {
          Text("4").font(IntradaFont.body).foregroundStyle(IntradaColor.ink)
        }
        FieldCard("Beat value") {
          SegmentedPills(
            options: [2, 4, 8], selection: .constant(4), label: { "\($0)" },
            layout: .fullWidthTrack)
        }
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(cards), as: config)
  }

  func testFieldLabel() {
    let labels = ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        FieldLabel("Key")
        FieldLabel("Title", tint: IntradaColor.danger)
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(labels), as: config)
  }

  func testSwitchRow() {
    let rows = ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.cardCompact) {
        SwitchRow(title: "Include in warm-up?", isOn: .constant(true), identifier: "on")
          .padding(IntradaSpacing.card)
          .cardSurface()
        SwitchRow(title: "Planned duration", isOn: .constant(false), identifier: "off")
          .padding(IntradaSpacing.card)
          .cardSurface()
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(rows), as: config)
  }

  func testFormField() {
    let fields = ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.cardCompact) {
        VStack(spacing: 0) {
          FormField(label: "Title", text: .constant("Clair de Lune"), placeholder: "Required")
          HairlineDivider()
          FormField(label: "Tempo marking", text: .constant(""), placeholder: "e.g. Allegro")
          HairlineDivider()
          FormField(
            label: "Bars", text: .constant(""), placeholder: "1 to 16",
            note: "Optional. Bar 12, or 12 to 14")
        }
        .cardSurface()
        FormField(label: "Title", text: .constant(""), placeholder: "Required", faulted: true)
          .cardSurface()
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(fields), as: config)
  }

  func testProfileBadge() {
    let badges = ZStack {
      PaperBackground()
      HStack(spacing: IntradaSpacing.card) {
        ProfileBadge(icon: .cello, size: IntradaGlyph.bar)
        ProfileBadge(icon: .cello).marker(.mint)
        ProfileBadge(icon: .other, size: IntradaGlyph.hero).marker(.lavender)
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(badges), as: config)
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

  func testScoreSelector() {
    let selectors = ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.section) {
        ForEach([0, 4, 10], id: \.self) { score in
          ScoreSelector(
            score: score, range: 1...10, accessibilityLabel: "Confidence for Clair de Lune"
          ) { _ in }
        }
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(selectors), as: config)
  }

  func testScoreStepper() {
    let states: [(Int, ScoreStepper.Origin)] = [
      (4, .lastTime), (5, .today), (6, .suggested), (10, .today),
    ]
    let steppers = ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.section) {
        ForEach(Array(states.enumerated()), id: \.offset) { _, state in
          ScoreStepper(
            score: state.0, origin: state.1, range: LimitsView.preview.scoreRange,
            accessibilityLabel: "Score for D major"
          ) { _ in }
        }
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(steppers), as: config)
  }

  func testScoreStepperSessionLength() {
    let steppers = ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.section) {
        ForEach([20, 5], id: \.self) { minutes in
          ScoreStepper(
            score: minutes, caption: .unit("minutes"), range: 5...120, step: 5,
            figure: IntradaFont.timer(32), typing: true,
            accessibilityLabel: "Session length", accessibilityValue: "\(minutes) minutes"
          ) { _ in }
        }
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(steppers), as: config)
  }

  func testOutlineButton() {
    let buttons = ZStack {
      PaperBackground()
      HStack(spacing: IntradaSpacing.controlGap) {
        OutlineButton(title: "Fill with suggestions") {}
        OutlineButton(title: "Stretch to fit") {}
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(buttons), as: config)
  }

  func testMinutesUsedMeter() {
    let meters = ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.section) {
        MinutesUsedMeter(
          summary: "72 of 60 min", stateLine: "12 min over", usedMins: 72, lengthMins: 60,
          state: .over)
        MinutesUsedMeter(
          summary: "15 of 60 min, plus 2 untimed", stateLine: "45 min left", usedMins: 15,
          lengthMins: 60, state: .under)
        MinutesUsedMeter(
          summary: "60 of 60 min", stateLine: "Fits exactly", usedMins: 60, lengthMins: 60,
          state: .exact)
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(meters), as: config)
  }

  func testTempoDrag() {
    let limits = LimitsView.preview
    let band = limits.clickBand(unit: 4)
    let steppers = ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.section) {
        TempoDrag(value: .constant(88), step: limits.clickStep, band: band)
        TempoDrag(
          value: .constant(168), unit: 8, step: limits.clickStep,
          band: limits.clickBand(unit: 8))
        TempoDrag(value: .constant(band.upperBound), step: limits.clickStep, band: band)
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(steppers), as: config)
  }

  func testLibraryItemCard() {
    var priority = LibraryItemView.previewPiece
    priority.priority = true
    priority.tags = ["recital", "memorised"]
    var majorScales = LibraryItemView.previewExercise
    majorScales.id = "scales-twelve"
    majorScales.title = "Major scales"
    majorScales.subtitle = ""
    majorScales.keys = KeyHelper.circle(.major).map { key in
      ItemKeyView(
        key: key, label: KeyHelper.display(key) ?? "", latestScore: nil, caption: "Not yet played")
    }
    var arpeggios = LibraryItemView.previewScales
    arpeggios.id = "arpeggios"
    arpeggios.title = "Arpeggios"
    let rows = ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.cardCompact) {
        LibraryItemCard(item: priority, showsMastery: true, showsMissingDetailsPrompt: true)
        LibraryItemCard(item: majorScales, showsMastery: true, showsMissingDetailsPrompt: true)
        LibraryItemCard(
          item: .previewExerciseWithTwelveVariations, showsMastery: true,
          showsMissingDetailsPrompt: true)
        LibraryItemCard(item: .previewScales, showsMastery: true, showsMissingDetailsPrompt: true)
        SelectableLibraryRow(
          item: .previewExercise, added: true, addHint: "Adds it to the session",
          removeHint: "Removes it from the session", action: {})
        SelectableLibraryRow(
          item: arpeggios, added: false, addHint: "Adds it to the session",
          removeHint: "Removes it from the session", action: {})
      }
      .padding(IntradaSpacing.card)
    }
    assertSnapshot(of: host(rows), as: config)
  }

  func testMasteryDeltaToast() {
    let toasts = ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.section) {
        MasteryDeltaToast(title: "Clair de Lune moved up", subtitle: nil, was: 3, now: 4)
        MasteryDeltaToast(title: "Mastery up", subtitle: "Clair de Lune", was: 5, now: 7)
      }
      .padding(IntradaSpacing.section)
    }
    assertSnapshot(of: host(toasts), as: config)
  }
}

/// The dark hero surface a tinted section title sits on.
private struct HeroPanel<Content: View>: View {
  @Environment(\.heroGradient) private var heroGradient
  @ViewBuilder let content: Content

  var body: some View {
    content
      .frame(maxWidth: .infinity, alignment: .leading)
      .padding(IntradaSpacing.card)
      .background(heroGradient)
      .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
  }
}
