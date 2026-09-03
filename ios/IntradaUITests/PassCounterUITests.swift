import XCTest

/// The pass counter end to end (#1367): the resident counter reads `0 of 10`
/// before anyone touches it, and each tap moves the count the way the core
/// rules. The real-bridge unit test covers the wire; this covers the buttons.
@MainActor
final class PassCounterUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  func testTapsMoveTheResidentCounter() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()

    app.tabBars.buttons["Practice"].tap()
    app.openEmptyBuilder()

    let addRow = app.buttons["Add piece or exercise"]
    XCTAssertTrue(addRow.waitForExistence(timeout: 5), "Add row")
    addRow.tap()
    let notAdded = app.buttons.matching(NSPredicate(format: "value == %@", "Not added"))
    XCTAssertTrue(notAdded.firstMatch.waitForExistence(timeout: 5), "Library cards in sheet")
    notAdded.firstMatch.tap()
    app.buttons["Done"].tap()

    let startSession = app.buttons["Start session"]
    XCTAssertTrue(startSession.waitForExistence(timeout: 5), "Start session bar")
    startSession.tap()

    let passes = app.otherElements["Passes"]
    XCTAssertTrue(passes.waitForExistence(timeout: 10), "the counter is resident from the start")
    XCTAssertEqual(passes.value as? String, "0 of 10", "untouched reads 0 of 10 with no distance")

    app.buttons["Got it"].tap()
    XCTAssertEqual(
      passes.value as? String, "1 of 10, 9 to go", "a pass banks and the distance appears")

    app.buttons["Not quite"].tap()
    XCTAssertEqual(passes.value as? String, "0 of 10, 10 to go", "a miss steps back, floor zero")

    app.buttons["Not quite"].tap()
    XCTAssertEqual(passes.value as? String, "0 of 10, 10 to go", "a miss at zero is still tappable")

    // Leave the container clean for the next test: abandon via Session options.
    app.buttons["Session options"].tap()
    let end = app.buttons["End session early"]
    if end.waitForExistence(timeout: 3) {
      end.tap()
      let discard = app.buttons["Discard"]
      if discard.waitForExistence(timeout: 5) { discard.tap() }
    }
  }
}
