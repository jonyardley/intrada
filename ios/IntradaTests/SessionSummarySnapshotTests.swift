import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class SessionSummarySnapshotTests: SnapshotTestCase {
  func testSessionSummaryCompleted() {
    assertSnapshot(of: host(SessionSummaryScreen(), store: .previewSummary), as: config)
  }

  /// A mark per variation, beside a piece with one play (#1739 decision 10).
  func testSessionSummaryWithVariations() {
    assertSnapshot(
      of: host(SessionSummaryScreen(), store: .previewSummaryVariations), as: config)
  }

  func testSessionSummaryEndedEarly() {
    assertSnapshot(
      of: host(SessionSummaryScreen(), store: .previewSummaryEndedEarly), as: config)
  }

  func testScoreSelectorPills() {
    let selectors = ZStack {
      PaperBackground()
      VStack(spacing: 20) {
        ScoreSelector(score: 0, range: 1...10, accessibilityLabel: "Score") { _ in }
        ScoreSelector(score: 4, range: 1...10, accessibilityLabel: "Score") { _ in }
        ScoreSelector(score: 10, range: 1...10, accessibilityLabel: "Score") { _ in }
      }
      .padding(16)
    }
    assertSnapshot(of: host(selectors), as: config)
  }

  /// One play scores with the same stepper as several, here on last time's score (#3004).
  func testReflectionSheet() {
    assertSnapshot(of: host(onePlaySheet(dragScore: false)), as: config)
  }

  // FIXME(#3004): the two option snapshots and the sheet's dragScore parameter
  // go once the owner picks the score's control.
  func testReflectionSheetOptionDragScore() {
    assertSnapshot(of: host(onePlaySheet(dragScore: true)), as: config)
  }

  private func onePlaySheet(dragScore: Bool) -> some View {
    ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Scales · D♭", elapsedDisplay: "7:00", tempoTarget: nil,
        tempoRows: [.preview("p1", lastScore: 7, startScore: 7)],
        plays: [.preview("p1", nil, "7:00")],
        limits: .preview,
        onSave: { _ in }, onSkip: {}, dragScore: dragScore)
    }
  }

  func testReflectionSheetWithTempoTarget() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Scales · D♭", elapsedDisplay: "7:00",
        tempoTarget: clickTempoWords(bpm: 96, unit: 4).text,
        tempoRows: [.preview("p1")],
        plays: [.preview("p1", nil, "7:00")],
        limits: .preview,
        onSave: { _ in }, onSkip: {})
    }
    assertSnapshot(of: host(sheet), as: config)
  }

  /// Each key starts on last time's score or a changed one, with a note under D major (#3004).
  func testReflectionSheetWithThreeVariations() {
    assertSnapshot(of: host(threeVariationsSheet(dragScore: false)), as: config)
  }

  func testReflectionSheetWithThreeVariationsOptionDragScore() {
    assertSnapshot(of: host(threeVariationsSheet(dragScore: true)), as: config)
  }

  private func threeVariationsSheet(dragScore: Bool) -> some View {
    ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Major scales", elapsedDisplay: "11:20", tempoTarget: nil,
        tempoRows: [
          .preview("p1", tempo: 96, lastScore: 4, startScore: 4),
          .preview("p2", tempo: 92, lastScore: 5, startScore: 5),
          .preview("p3", tempo: 88, lastScore: 3, startScore: 3),
        ],
        plays: [
          .preview("p1", "C major", "3:05"),
          .preview("p2", "G major", "4:10"),
          .preview("p3", "D major", "4:05"),
        ],
        limits: .preview,
        seed: ReflectionAnswers.preview(
          marks: [DraftMark(playId: "p1", score: 5)],
          notes: [DraftNote(playId: "p3", note: "Stumbles at the top on the way down.")]),
        onSave: { _ in }, onSkip: {}, dragScore: dragScore)
    }
  }

  /// A key never scored starts on the core's suggestion, beside one scored before (#3004).
  func testReflectionSheetWithANeverScoredKey() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Major scales", elapsedDisplay: "8:00", tempoTarget: nil,
        tempoRows: [
          .preview("p1", lastScore: 6, startScore: 6),
          .preview("p2", lastScore: nil, startScore: 6),
        ],
        plays: [
          .preview("p1", "C major", "4:00"),
          .preview("p2", "F\u{266f} major", "4:00"),
        ],
        limits: .preview,
        onSave: { _ in }, onSkip: {})
    }
    assertSnapshot(of: host(sheet), as: config)
  }

  /// A key's tempo and note, opened from its name (#3004).
  func testReflectionKeySheet() {
    let row = ReflectionTempoView.preview("p3", tempo: 88)
    let sheet = ReflectionKeySheet(
      title: "D major", tempoRow: row, limits: .preview, tempo: TrackedTempo(row: row),
      note: "Stumbles at the top on the way down.",
      wayRow: nil, wayChoices: PlayWayChoices(keys: [], variations: [], planned: nil),
      onWay: { _ in }, onDone: { _, _ in nil })
    assertSnapshot(of: host(sheet), as: config)
  }

  /// The core's refusal stays on the key's own sheet, naming the key (#3004).
  func testReflectionKeySheetRefusingANote() {
    let row = ReflectionTempoView.preview("p3", tempo: 88)
    let sheet = ReflectionKeySheet(
      title: "D major", tempoRow: row, limits: .preview, tempo: TrackedTempo(row: row),
      note: "Stumbles at the top on the way down.",
      wayRow: nil, wayChoices: PlayWayChoices(keys: [], variations: [], planned: nil),
      refusal: "Keep the note under 500 characters.",
      onWay: { _ in }, onDone: { _, _ in nil })
    assertSnapshot(of: host(sheet), as: config)
  }

  func testReflectionSheetWithADroppedPlay() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Major Scales", elapsedDisplay: "4:12", tempoTarget: nil,
        tempoRows: [.preview("p1")],
        plays: [
          .preview("p1", "C major", "4:10"),
          .preview("p2", "G major", "0:02", isMarkable: false),
        ],
        limits: .preview,
        onSave: { _ in }, onSkip: {})
    }
    assertSnapshot(of: host(sheet), as: config)
  }

  func testReflectionSheetWithARefusedSave() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Scales · D♭", elapsedDisplay: "7:00", tempoTarget: nil,
        tempoRows: [.preview("p1")],
        plays: [.preview("p1", nil, "7:00")],
        limits: .preview,
        refusal: "Notes must not exceed 5000 characters",
        onSave: { _ in }, onSkip: {})
    }
    assertSnapshot(of: host(sheet), as: config)
  }

  func testReflectionSheetWithThreeVariationsAccessibilitySize() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Major Scales", elapsedDisplay: "12:40", tempoTarget: nil,
        tempoRows: [.preview("p1"), .preview("p2")],
        plays: [
          .preview("p1", "C major", "4:10"),
          .preview("p2", "G major", "3:20"),
        ],
        limits: .preview,
        onSave: { _ in }, onSkip: {})
    }
    .dynamicTypeSize(.accessibility1)
    assertSnapshot(of: host(sheet), as: config)
  }

  /// A stamped row counts in its own metre (7/8, quavers); an unstamped one counts in crotchets (#1761 rule 6).
  func testReflectionSheetWithTempoPerRow() {
    let sevenEight = Metre(beats: 7, unit: 8, groups: [3, 2, 2])
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Major Scales", elapsedDisplay: "12:40", tempoTarget: nil,
        tempoRows: [
          .preview("p1", tempo: 168, click: ClickState(metre: sevenEight, sounding: 0b0101001)),
          .preview("p2"),
        ],
        plays: [
          .preview("p1", "C major", "4:10"),
          .preview("p2", "G major", "3:20"),
        ],
        limits: .preview,
        onSave: { _ in }, onSkip: {})
    }
    assertSnapshot(of: host(sheet), as: config)
  }

  func testReflectionSheetWithTempoPerRowAccessibilitySize() {
    let sevenEight = Metre(beats: 7, unit: 8, groups: [3, 2, 2])
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Major Scales", elapsedDisplay: "12:40", tempoTarget: nil,
        tempoRows: [
          .preview("p1", tempo: 168, click: ClickState(metre: sevenEight, sounding: 0b0101001)),
          .preview("p2"),
        ],
        plays: [
          .preview("p1", "C major", "4:10"),
          .preview("p2", "G major", "3:20"),
        ],
        limits: .preview,
        onSave: { _ in }, onSkip: {})
    }
    .dynamicTypeSize(.accessibility1)
    assertSnapshot(of: host(sheet), as: config)
  }

  // A key's sheet, its key changed from D major to G major (#2249).
  func testReflectionSheetChangingWhatWasPlayed() {
    let finish = FinishSheetView.previewWayChoices
    let row = ReflectionTempoView.preview("p1")
    let sheet = ReflectionKeySheet(
      title: "G major", tempoRow: row, limits: .preview, tempo: TrackedTempo(row: row), note: "",
      wayRow: finish.rows.first,
      wayChoices: PlayWayChoices(
        keys: finish.keys,
        variations: [
          PickerVariationView(id: "v-hs", label: "Hands separately", caption: ""),
          PickerVariationView(id: "v-dot", label: "Dotted rhythms", caption: ""),
        ],
        planned: "D major"),
      onWay: { _ in }, onDone: { _, _ in nil })
    assertSnapshot(of: host(sheet), as: config)
  }
}
