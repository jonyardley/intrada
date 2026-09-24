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
        ScoreSelector(score: 0, accessibilityLabel: "Score") { _ in }
        ScoreSelector(score: 4, accessibilityLabel: "Score") { _ in }
        ScoreSelector(score: 10, accessibilityLabel: "Score") { _ in }
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
        onSave: { _ in }, onSkip: {})
    }
    .dynamicTypeSize(.accessibility1)
    assertSnapshot(of: host(sheet), as: config)
  }
}
