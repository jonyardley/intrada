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

  func testSessionSummaryAccessibilitySize() {
    assertSnapshot(of: host(SessionSummaryScreen(), store: .previewSummary), as: axConfig)
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

  func testReflectionSheet() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Scales · D♭", elapsedDisplay: "7:00", tempoTarget: nil,
        plays: [.preview("p1", nil, "7:00")],
        limits: .preview,
        onSave: { _ in }, onSkip: {})
    }
    assertSnapshot(of: host(sheet), as: config)
  }

  func testReflectionSheetWithTempoTarget() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Scales · D♭", elapsedDisplay: "7:00", tempoTarget: 96,
        plays: [.preview("p1", nil, "7:00")],
        limits: .preview,
        onSave: { _ in }, onSkip: {})
    }
    assertSnapshot(of: host(sheet), as: config)
  }

  func testReflectionSheetWithThreeVariations() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Major Scales", elapsedDisplay: "12:40", tempoTarget: nil,
        plays: [
          .preview("p1", "C major", "4:10", 8, 10),
          .preview("p2", "G major", "3:20", 10, 10),
          .preview("p3", "D major", "5:10", 4, 10),
        ],
        limits: .preview,
        onSave: { _ in }, onSkip: {})
    }
    assertSnapshot(of: host(sheet), as: config)
  }

  func testReflectionSheetWithADroppedPlay() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Major Scales", elapsedDisplay: "4:12", tempoTarget: nil,
        plays: [
          .preview("p1", "C major", "4:10", 8, 10),
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
        plays: [
          .preview("p1", "C major", "4:10", 8, 10),
          .preview("p2", "G major", "3:20", 10, 10),
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
        plays: [
          .preview(
            "p1", "C major", "4:10", 8, 10,
            tempoDisplay: 168, clickPattern: ClickState(metre: sevenEight, sounding: 0b0101001)),
          .preview("p2", "G major", "3:20", 10, 10),
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
        plays: [
          .preview(
            "p1", "C major", "4:10", 8, 10,
            tempoDisplay: 168, clickPattern: ClickState(metre: sevenEight, sounding: 0b0101001)),
          .preview("p2", "G major", "3:20", 10, 10),
        ],
        limits: .preview,
        onSave: { _ in }, onSkip: {})
    }
    .dynamicTypeSize(.accessibility1)
    assertSnapshot(of: host(sheet), as: config)
  }

  // The aim asked, two points read from the note (one kept), detail open (#2303, #2307, #2308).
  func testReflectionSheetWithFinishAnswers() {
    assertSnapshot(of: host(finishSheet), as: config)
  }

  func testReflectionSheetWithFinishAnswersAccessibilitySize() {
    assertSnapshot(of: host(finishSheet), as: axConfig)
  }

  func testReflectionSheetWithTheAimReadFromThePlays() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Clair de Lune", elapsedDisplay: "12:00", tempoTarget: nil,
        plays: [.preview("p1", "A1", "4:10"), .preview("p2", "B", "4:00")],
        limits: .preview, finish: .preview(asksIntention: false, read: .yes), aim: "A1 at 84",
        seed: ReflectionResult(
          marks: [:], note: "A1: left hand rushed in bar 12, got it at 84", tempos: []),
        onSave: { _ in }, onSkip: {})
    }
    assertSnapshot(of: host(sheet), as: config)
  }

  private var finishSheet: some View {
    ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Clair de Lune", elapsedDisplay: "12:00", tempoTarget: nil,
        plays: [.preview("p1", nil, "12:00")],
        limits: .preview, finish: .preview(asksIntention: true), aim: "A1 from memory",
        seed: ReflectionResult(
          marks: [:], note: "A1: left hand rushed in bar 12, got it at 84", tempos: [],
          felt: .strained, obstacles: [.rhythm, .tension],
          notePoints: [NoteSpan(start: 38, end: 40)], intentionMet: .partly),
        onSave: { _ in }, onSkip: {})
    }
  }
}
