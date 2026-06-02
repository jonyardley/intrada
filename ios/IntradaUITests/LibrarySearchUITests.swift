import XCTest

/// Exercises the real running app — the search interactions snapshot tests
/// can't cover.
@MainActor
final class LibrarySearchUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  private func launchSeeded() -> XCUIApplication {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data"]
    app.launch()
    return app
  }

  func testSearchButtonRevealsFiltersAndCancels() {
    let app = launchSeeded()
    XCTAssertTrue(app.staticTexts["Library"].waitForExistence(timeout: 10), "Library header")
    XCTAssertTrue(
      app.staticTexts["Clair de Lune"].waitForExistence(timeout: 5), "Full list before searching")

    let searchField = app.textFields["Search library"]
    XCTAssertFalse(searchField.exists, "Search field hidden until the button is tapped")

    app.buttons["Search"].tap()
    XCTAssertTrue(searchField.waitForExistence(timeout: 5), "Tapping Search reveals the field")
    // Tap to ensure focus before typing — the reveal animates in and typeText drops keys otherwise.
    searchField.tap()
    searchField.typeText("hanon")

    XCTAssertTrue(
      app.staticTexts["Hanon No. 1"].waitForExistence(timeout: 5),
      "Matching item stays after filtering")
    XCTAssertTrue(
      app.staticTexts["Clair de Lune"].waitForNonExistence(timeout: 5),
      "Non-matching item filtered out")

    app.buttons["Cancel"].tap()
    XCTAssertTrue(
      app.staticTexts["Clair de Lune"].waitForExistence(timeout: 5),
      "Cancel clears the query and restores the full list")
    XCTAssertTrue(searchField.waitForNonExistence(timeout: 5), "Cancel hides the search field")
  }
}
