import XCTest

/// Real-bridge UITest for the "Build session" builder (#935). Adding moved into
/// the "Add to session" sheet. `--disable-animations` stops the Practice
/// week-strip's paging TabView defeating XCUITest idle (#941).
@MainActor
final class SessionBuilderUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  private func launchSeeded() -> XCUIApplication {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()
    return app
  }

  func testBuildSetlistAddRemoveThenCancel() {
    let app = launchSeeded()

    app.tabBars.buttons["Practice"].tap()
    let start = app.buttons["Start practising"]
    XCTAssertTrue(start.waitForExistence(timeout: 10), "Start practising")
    start.tap()

    // Adding moved to the "Add to session" sheet — open it from the dashed row.
    let addRow = app.buttons["Add piece or exercise"]
    XCTAssertTrue(addRow.waitForExistence(timeout: 5), "Add row")
    addRow.tap()

    // In the sheet, add the top two cards ("Not added" / "Added" a11y value).
    let notAdded = app.buttons.matching(NSPredicate(format: "value == %@", "Not added"))
    XCTAssertTrue(notAdded.firstMatch.waitForExistence(timeout: 5), "Library cards in sheet")
    notAdded.firstMatch.tap()
    XCTAssertTrue(notAdded.firstMatch.waitForExistence(timeout: 5), "A second card to add")
    notAdded.firstMatch.tap()
    app.buttons["Done"].tap()

    // Both land as standalone rows (their remove buttons are "Remove <title>").
    let removes = app.buttons.matching(NSPredicate(format: "label BEGINSWITH %@", "Remove"))
    XCTAssertTrue(removes.firstMatch.waitForExistence(timeout: 5), "queued items")
    XCTAssertEqual(removes.count, 2, "two items queued")

    removes.firstMatch.tap()
    XCTAssertEqual(
      app.buttons.matching(NSPredicate(format: "label BEGINSWITH %@", "Remove")).count, 1,
      "one item remains after removing from the queue")

    let cancel = app.buttons["Cancel"]
    cancel.tap()
    XCTAssertTrue(app.buttons["Discard"].waitForExistence(timeout: 5), "Cancel confirms")
    app.buttons["Keep editing"].tap()
    XCTAssertTrue(cancel.waitForExistence(timeout: 5), "Keep editing stays in the builder")

    app.buttons["Cancel"].tap()
    app.buttons["Discard"].tap()
    XCTAssertTrue(start.waitForExistence(timeout: 5), "Discard returns to Practice")
  }
}
