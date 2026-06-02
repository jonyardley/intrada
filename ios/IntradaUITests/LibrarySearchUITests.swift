import XCTest

/// Exercises the real running app — the search interactions snapshot tests can't
/// cover. Seeds demo data so the Library has rows to filter.
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
    XCTAssertTrue(app.staticTexts["Clair de Lune"].exists, "Full list before searching")

    let searchField = app.textFields["Search library"]
    XCTAssertFalse(searchField.exists, "Search field hidden until the button is tapped")

    app.buttons["Search"].tap()
    XCTAssertTrue(searchField.waitForExistence(timeout: 3), "Tapping Search reveals the field")

    searchField.typeText("hanon")
    XCTAssertTrue(
      app.staticTexts["Hanon No. 1"].waitForExistence(timeout: 3),
      "Matching item stays after filtering")
    XCTAssertFalse(app.staticTexts["Clair de Lune"].exists, "Non-matching item filtered out")

    app.buttons["Cancel"].tap()
    XCTAssertTrue(
      app.staticTexts["Clair de Lune"].waitForExistence(timeout: 3),
      "Cancel clears the query and restores the full list")
    XCTAssertFalse(searchField.exists, "Cancel hides the search field")
  }
}
