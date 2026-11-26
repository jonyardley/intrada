import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

/// Today's keys on the plan row and in their sheet (#3008).
final class TodaysKeysSnapshotTests: SnapshotTestCase {
  private var tall: Snapshotting<UIViewController, UIImage> {
    .image(
      on: ViewImageConfig(
        safeArea: .zero, size: CGSize(width: 390, height: 1000),
        traits: .init(displayScale: 1)),
      perceptualPrecision: 0.98, traits: .init(displayScale: 1))
  }

  func testPlanRowWithKeys() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPractice(plan: .previewWithKeys)), as: config)
  }

  func testSheet() {
    assertSnapshot(
      of: host(
        TodaysKeysSheet(itemId: PlanRowView.previewScalesWithKeys.itemId),
        store: .previewPractice(plan: .previewWithKeys)), as: tall)
  }

  func testCaptionReadsScoreThenWhen() {
    let scored = PlanKeyView.preview(.g, label: "G major", score: 3, lastPlayed: "2 weeks ago")
    let unplayed = PlanKeyView.preview(.f, .sharp, label: "F\u{266F} major")
    XCTAssertEqual(TodaysKeysSheet.caption(scored), "Scored 3, 2 weeks ago")
    XCTAssertEqual(TodaysKeysSheet.caption(unplayed), "Not practised yet")
  }
}
