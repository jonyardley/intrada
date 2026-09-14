import XCTest

/// Real-device UITest for variation management (#1083, renamed in #1733):
/// renaming a live-queried field, driven against "Major Scales", the seeded
/// exercise whose demo variations are deterministic (`C`, `G`, `D`, `A`, `E`,
/// in that order; see `app.rs`'s `LoadSampleData` seed). Removing a variation
/// moved to `StoreEffectLoopTests` (#1825): plain taps, no keyboard.
///
/// Drag reorder is deliberately not covered here: `.draggable`/
/// `.dropDestination` ride the system Drag & Drop API, which XCUITest can't
/// reliably script a drop through in the Simulator (unlike a List's
/// `onMove` long-press-drag, which `SessionBuilderUITests` automates
/// successfully). Reorder-by-relabeling itself is fully covered by the
/// core's `set_variants_reorder_preserves_ids_by_label` test (id/history
/// preserved, position updated); only the gesture is untested here.
@MainActor
final class VariationManagementUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  private func openScalesVariations() -> XCUIApplication {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()

    app.tabBars.buttons["Library"].tap()
    let scalesRow = app.buttons.matching(
      NSPredicate(format: "label CONTAINS %@", "Major Scales")
    ).firstMatch
    XCTAssertTrue(scalesRow.waitForExistence(timeout: 10), "Major Scales library row")
    scalesRow.tap()

    // Major Scales is an all-key ladder, so #1464 reads it as "Keys".
    let editButton = app.buttons["Edit keys"]
    XCTAssertTrue(editButton.waitForExistence(timeout: 10), "Keys section with Edit button")
    editButton.tap()
    return app
  }

  private func variationField(_ app: XCUIApplication, value: String) -> XCUIElement {
    app.textFields.matching(NSPredicate(format: "value == %@", value)).firstMatch
  }

  func testRenameVariationPersistsAndLeavesOthersUntouched() {
    let app = openScalesVariations()

    let eField = variationField(app, value: "E")
    XCTAssertTrue(eField.waitForExistence(timeout: 5), "E variation field")
    eField.tap()
    // One typeText call: eField is a live query on `value == "E"`, so a
    // second call after a separate delete step lands can no longer
    // re-resolve it, since the value has already changed (#1642).
    eField.typeText(XCUIKeyboardKey.delete.rawValue + "Fa\n")

    // "Fa" isn't a key, so the ladder reads "Variations" again by Done.
    app.buttons["Done editing variations"].tap()

    // Back in read mode: the renamed variation shows, "E" is gone, others intact.
    XCTAssertTrue(
      app.staticTexts["Fa"].waitForExistence(timeout: 5), "renamed variation reads back as Fa")
    XCTAssertFalse(app.staticTexts["E"].exists, "old label gone")
    XCTAssertTrue(app.staticTexts["C"].exists, "untouched variation still present")
    XCTAssertTrue(app.staticTexts["G"].exists, "untouched variation still present")
  }
}
