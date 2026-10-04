import XCTest

/// Real-device UITest for variation management (#1083, renamed in #1733): a
/// drag on the Edit screen (#1783), driven against "Major Scales", the seeded
/// exercise whose one variation is "Hands separately" (`sample.rs`).
///
/// Removing a variation moved to `LibraryBridgeTests` (#1825): plain taps, no keyboard.
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
}
