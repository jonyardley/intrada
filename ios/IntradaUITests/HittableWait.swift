import XCTest

extension XCUIElementQuery {
  /// A covered stacked sheet stays in the tree but not hittable, so a shared
  /// label (two "Required" fields, two "Done" buttons) needs this to disambiguate.
  @MainActor
  func firstHittable(timeout: TimeInterval = 5) -> XCUIElement? {
    let deadline = Date().addingTimeInterval(timeout)
    repeat {
      if let match = allElementsBoundByIndex.first(where: \.isHittable) {
        return match
      }
      usleep(100_000)
    } while Date() < deadline
    return nil
  }
}
