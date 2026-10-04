import XCTest

/// Real-device UITest for variation management (#1083, renamed in #1733): a
/// drag on the Edit screen (#1783), driven against "Major Scales", the seeded
/// exercise whose one variation is "Hands separately" (`sample.rs`).
///
/// Removing a variation moved to `LibraryBridgeTests` (#1825): plain taps, no keyboard.
/// Renaming one acts on the whole library, from the item screen's Variations
/// sheet (#2247).
@MainActor
final class VariationManagementUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  private func openScalesEditor() -> XCUIApplication {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()

    app.tabBars.buttons["Library"].tap()
    let scalesRow = app.row("library.row", spokenContaining: "Major Scales")
    XCTAssertTrue(scalesRow.waitForExistence(timeout: 10), "Major Scales library row")
    scalesRow.tap()

    app.control("libraryDetail.edit", spoken: "Edit", timeout: 10).tap()
    return app
  }

  private func variationField(_ app: XCUIApplication, value: String) -> XCUIElement {
    app.descendants(matching: .any).matching(
      NSPredicate(
        format: "identifier == %@ AND (value == %@ OR label == %@)", "variationRow.label", value,
        "Variation \(value)")
    ).firstMatch
  }

  /// The handle's own gesture, which synthesised touches drive: the system drag
  /// and drop it replaced never landed a drop on a row on the simulator (#1783).
  func testDraggingAHandleMovesItsRow() {
    let app = openScalesEditor()

    let add = app.buttons["Add a variation"]
    XCTAssertTrue(add.waitForExistence(timeout: 5), "the add row under the saved variation")
    add.tap()
    app.typeText("Slow")

    let slowHandle = app.row("variationRow.reorder", spokenContaining: "Reorder Slow")
    let savedHandle = app.row("variationRow.reorder", spokenContaining: "Reorder Hands separately")
    XCTAssertTrue(slowHandle.waitForExistence(timeout: 5), "the typed row's reorder handle")
    XCTAssertGreaterThan(
      variationField(app, value: "Slow").frame.minY,
      variationField(app, value: "Hands separately").frame.minY,
      "the typed row starts below the saved one")

    slowHandle.press(
      forDuration: 0.2, thenDragTo: savedHandle, withVelocity: .slow, thenHoldForDuration: 0.3)

    XCTAssertLessThan(
      variationField(app, value: "Slow").frame.minY,
      variationField(app, value: "Hands separately").frame.minY,
      "the typed row dragged above the saved one")
  }

  /// Renamed in the sheet, the saved variation reads back under its new name
  /// on the item screen, and the old name is gone.
  func testRenameVariationFromTheSheet() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()
    app.tabBars.buttons["Library"].tap()
    let scalesRow = app.row("library.row", spokenContaining: "Major Scales")
    XCTAssertTrue(scalesRow.waitForExistence(timeout: 10), "Major Scales library row")
    scalesRow.tap()

    app.buttons["Choose variations"].tap()
    let row = app.row("variationsSheet.row", spokenContaining: "Hands separately")
    XCTAssertTrue(row.waitForExistence(timeout: 5), "the saved variation in the sheet")
    row.press(forDuration: 1.0)
    app.buttons["Rename"].tap()

    let field = app.alerts.textFields.firstMatch
    XCTAssertTrue(field.waitForExistence(timeout: 5), "the rename field")
    field.tap()
    field.typeText(
      String(repeating: XCUIKeyboardKey.delete.rawValue, count: "Hands separately".count)
        + "Hands apart")
    app.alerts.buttons["Rename"].tap()
    app.control("sheet.done", spoken: "Done").tap()

    XCTAssertTrue(
      app.staticTexts["Hands apart"].waitForExistence(timeout: 5), "the new name on the item")
    XCTAssertFalse(app.staticTexts["Hands separately"].exists, "the old name is gone")
  }
}
