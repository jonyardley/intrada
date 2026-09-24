import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class PracticeSplitSnapshotTests: SnapshotTestCase {
  func testPracticeSplitViewEmptyDetail() {
    assertSnapshot(
      of: host(
        PracticeSplitView(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPractice),
      as: splitConfig)
  }

  func testPracticeSplitViewWithSelection() {
    assertSnapshot(
      of: host(
        PracticeSplitView(
          referenceDate: PracticeSessionView.previewReferenceDate, previewSelection: "session-1"),
        store: .previewPractice),
      as: splitConfig)
  }
}
