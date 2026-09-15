import XCTest

/// Real-device UITest for variation management (#1083, renamed in #1733): a
/// rename and a drag on the Edit screen (#1783), driven against "Major Scales", the seeded
/// exercise whose demo variations are deterministic (`C`, `G`, `D`, `A`, `E`,
/// in that order; see `app.rs`'s `LoadSampleData` seed).
///
/// Removing a variation moved to `StoreEffectLoopTests` (#1825): plain taps, no keyboard.
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
    let scalesRow = app.buttons.matching(
      NSPredicate(format: "label CONTAINS %@", "Major Scales")
    ).firstMatch
    XCTAssertTrue(scalesRow.waitForExistence(timeout: 10), "Major Scales library row")
    scalesRow.tap()

    let editButton = app.buttons["Edit"].firstMatch
    XCTAssertTrue(editButton.waitForExistence(timeout: 10), "Edit on the exercise's details")
    editButton.tap()
    return app
  }

  private func variationField(_ app: XCUIApplication, value: String) -> XCUIElement {
    app.textFields.matching(NSPredicate(format: "value == %@", value)).firstMatch
  }

  func testRenameVariationPersistsAndLeavesOthersUntouched() {
    let app = openScalesEditor()

    let eField = variationField(app, value: "E")
    XCTAssertTrue(eField.waitForExistence(timeout: 5), "E variation field")
    eField.tap()
    // One typeText call: eField is a live query on `value == "E"`, so a
    // second call after a separate delete step lands can no longer
    // re-resolve it, since the value has already changed (#1642).
    eField.typeText(XCUIKeyboardKey.delete.rawValue + "Fa\n")

    app.buttons["Save"].tap()

    // "Fa" isn't a key, so the details list the ladder as named rows.
    XCTAssertTrue(
      app.staticTexts["Fa"].waitForExistence(timeout: 5), "renamed variation reads back as Fa")
    XCTAssertFalse(app.staticTexts["E"].exists, "old label gone")
    XCTAssertTrue(app.staticTexts["C"].exists, "untouched variation still present")
    XCTAssertTrue(app.staticTexts["G"].exists, "untouched variation still present")
  }

  /// The handle's own gesture, which synthesised touches drive: the system drag
  /// and drop it replaced never landed a drop on a row on the simulator (#1783).
  func testDraggingAHandleMovesItsRow() {
    let app = openScalesEditor()

    let eHandle = app.descendants(matching: .any)["Reorder E"].firstMatch
    let cHandle = app.descendants(matching: .any)["Reorder C"].firstMatch
    XCTAssertTrue(eHandle.waitForExistence(timeout: 5), "E's reorder handle")
    XCTAssertGreaterThan(
      variationField(app, value: "E").frame.minY, variationField(app, value: "C").frame.minY,
      "E starts below C")

    eHandle.press(
      forDuration: 0.2, thenDragTo: cHandle, withVelocity: .slow, thenHoldForDuration: 0.3)

    XCTAssertLessThan(
      variationField(app, value: "E").frame.minY, variationField(app, value: "C").frame.minY,
      "E dragged above C")
  }
}
