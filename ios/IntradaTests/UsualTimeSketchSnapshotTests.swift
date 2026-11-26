import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class UsualTimeSketchSnapshotTests: SnapshotTestCase {
  func testALearned() { capture(.underBadge, UsualTimeSketch.learned) }
  func testANotYetLearned() { capture(.underBadge, UsualTimeSketch.notYetLearned) }
  func testALongTitle() { capture(.underBadge, UsualTimeSketch.longTitle) }
  func testALargestText() { capture(.underBadge, UsualTimeSketch.learned, largest: true) }

  func testBLearned() { capture(.withSessions, UsualTimeSketch.learned) }
  func testBNotYetLearned() { capture(.withSessions, UsualTimeSketch.notYetLearned) }
  func testBLongTitle() { capture(.withSessions, UsualTimeSketch.longTitle) }
  func testBLargestText() { capture(.withSessions, UsualTimeSketch.learned, largest: true) }

  func testCLearned() { capture(.sameStyle, UsualTimeSketch.learned) }
  func testCNotYetLearned() { capture(.sameStyle, UsualTimeSketch.notYetLearned) }
  func testCLongTitle() { capture(.sameStyle, UsualTimeSketch.longTitle) }
  func testCLargestText() { capture(.sameStyle, UsualTimeSketch.notYetLearned, largest: true) }

  private func capture(
    _ variation: UsualTimeSketch.Variation, _ item: LibraryItemView, largest: Bool = false,
    testName: String = #function
  ) {
    let sketch = UsualTimeSketch(item: item, variation: variation)
      .dynamicTypeSize(largest ? .accessibility5 : .large)
    assertSnapshot(of: host(sketch), as: crop(largest ? 1100 : 560), testName: testName)
  }

  private func crop(_ height: CGFloat) -> Snapshotting<UIViewController, UIImage> {
    .image(
      on: ViewImageConfig(
        safeArea: UIEdgeInsets(top: 47, left: 0, bottom: 0, right: 0),
        size: CGSize(width: 390, height: height), traits: .init(displayScale: 2)),
      perceptualPrecision: 0.98, traits: .init(displayScale: 2))
  }
}
