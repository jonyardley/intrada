import XCTest

/// Real-bridge UITest for the builder (#935). `--disable-animations` stops the
/// Practice week-strip's paging TabView defeating XCUITest idle (pulled from #933).
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

  func testBuildSetlistAddThenRemove() {
    let app = launchSeeded()

    let practiceTab = app.tabBars.buttons["Practice"]
    XCTAssertTrue(practiceTab.waitForExistence(timeout: 10), "Practice tab")
    practiceTab.tap()

    let startButton = app.buttons["Start practising"]
    XCTAssertTrue(startButton.waitForExistence(timeout: 5), "Start practising button")
    startButton.tap()

    let addItems = app.buttons["Add items"]
    XCTAssertTrue(addItems.waitForExistence(timeout: 5), "Builder's Add items action")
    addItems.tap()

    XCTAssertTrue(
      app.staticTexts["Add to session"].waitForExistence(timeout: 5), "Picker sheet title")
    tapButton(in: app, containing: "Clair de Lune")
    tapButton(in: app, containing: "Hanon No. 1")
    // Match by id — Done's label carries a running count ("Done · N").
    let done = app.buttons["sessionPickerDone"]
    XCTAssertTrue(done.waitForExistence(timeout: 5), "Picker Done button")
    done.tap()

    XCTAssertTrue(
      row(in: app, containing: "Clair de Lune").waitForExistence(timeout: 5),
      "Clair de Lune added to setlist")
    XCTAssertTrue(
      row(in: app, containing: "Hanon No. 1").waitForExistence(timeout: 5),
      "Hanon No. 1 added to setlist")

    revealSwipeActions(on: row(in: app, containing: "Clair de Lune"))
    let remove = app.buttons["Remove"]
    XCTAssertTrue(remove.waitForExistence(timeout: 5), "Swipe reveals Remove")
    remove.tap()

    XCTAssertTrue(
      row(in: app, containing: "Clair de Lune").waitForNonExistence(timeout: 5),
      "Removed entry leaves the setlist")
    XCTAssertTrue(row(in: app, containing: "Hanon No. 1").exists, "Untouched entry remains")
  }

  // Cards/rows combine into one a11y element ("<type>, <title>"); match by CONTAINS.
  private func tapButton(in app: XCUIApplication, containing label: String) {
    let button = app.buttons.matching(NSPredicate(format: "label CONTAINS %@", label)).firstMatch
    XCTAssertTrue(button.waitForExistence(timeout: 5), "Picker button for \(label)")
    button.tap()
  }

  private func row(in app: XCUIApplication, containing label: String) -> XCUIElement {
    app.staticTexts.matching(NSPredicate(format: "label CONTAINS %@", label)).firstMatch
  }

  // Partial drag reveals the actions without a full swipe committing the delete.
  private func revealSwipeActions(on element: XCUIElement) {
    let start = element.coordinate(withNormalizedOffset: CGVector(dx: 0.9, dy: 0.5))
    let end = element.coordinate(withNormalizedOffset: CGVector(dx: 0.2, dy: 0.5))
    start.press(forDuration: 0.05, thenDragTo: end)
  }
}
