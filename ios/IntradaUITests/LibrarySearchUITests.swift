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

  func testSearchFiltersAndCancelRestores() {
    let app = launchSeeded()
    XCTAssertTrue(
      app.navigationBars["Library"].waitForExistence(timeout: 10),
      "Library nav bar should appear")

    let searchField = app.searchFields["Search library"]
    XCTAssertTrue(searchField.waitForExistence(timeout: 3), "Search field is present")

    XCTAssertTrue(app.staticTexts["Clair de Lune"].exists, "Full list shows before searching")

    searchField.tap()
    searchField.typeText("hanon")
    XCTAssertTrue(
      app.staticTexts["Hanon No. 1"].waitForExistence(timeout: 3),
      "Matching item stays after filtering")
    XCTAssertFalse(app.staticTexts["Clair de Lune"].exists, "Non-matching item filtered out")

    searchField.typeText(String(repeating: XCUIKeyboardKey.delete.rawValue, count: 5))
    XCTAssertTrue(
      app.staticTexts["Clair de Lune"].waitForExistence(timeout: 3),
      "Clearing the query restores the full list")
  }
}
