import XCTest

/// The native back chevron and its edge-swipe gesture, restored on a pushed
/// screen once #1822 stopped hiding the nav bar in favour of a blank inline
/// title (#1820, #1822). A snapshot can't prove a gesture fired.
@MainActor
final class NativeBackNavigationUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  func testEdgeSwipeBackReturnsFromLibraryDetail() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()

    app.tabBars.buttons["Library"].tap()
    let clairRow = app.row("library.row", spokenContaining: "Clair de Lune")
    XCTAssertTrue(clairRow.waitForExistence(timeout: 10), "Clair library row")
    clairRow.tap()

    XCTAssertTrue(
      app.staticTexts["Claude Debussy"].waitForExistence(timeout: 10), "on the detail screen")

    let start = app.windows.firstMatch.coordinate(
      withNormalizedOffset: CGVector(dx: 0.01, dy: 0.5))
    let end = app.windows.firstMatch.coordinate(withNormalizedOffset: CGVector(dx: 0.9, dy: 0.5))
    start.press(forDuration: 0.05, thenDragTo: end)

    XCTAssertTrue(
      app.staticTexts["3 pieces · 2 exercises"].waitForExistence(timeout: 10),
      "the edge swipe popped back to Library")
  }
}
