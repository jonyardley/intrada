import XCTest

/// The bar line appears only while the click sounds, and the sheet it opens
/// changes which beats sound. The engine tests cover the grid; this covers the
/// wiring from the row to the sheet and back, and on to the reflection.
@MainActor
final class ClickBarUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  func testTheBarAppearsWhileSoundingAndFollowsTheSheet() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()
    app.startOneItemSession()

    let start = app.buttons["Start the click"]
    XCTAssertTrue(start.waitForExistence(timeout: 10), "the click row is up")
    XCTAssertFalse(app.buttons["Bar"].exists, "no bar line while silent")
    start.tap()

    let bar = app.buttons["Bar"]
    XCTAssertTrue(bar.waitForExistence(timeout: 5), "the bar line appears while sounding")
    XCTAssertEqual(bar.value as? String, "4 crotchet beats, click on every beat")

    bar.tap()
    let downbeat = app.buttons["Downbeat"]
    XCTAssertTrue(downbeat.waitForExistence(timeout: 5), "the click sheet opens from the bar line")
    downbeat.tap()
    app.buttons["Done"].tap()

    XCTAssertTrue(bar.waitForExistence(timeout: 5))
    XCTAssertEqual(bar.value as? String, "4 crotchet beats, click on beat 1")

    app.buttons["Stop the click"].tap()
    XCTAssertFalse(
      app.buttons["Bar"].waitForExistence(timeout: 2), "the bar line leaves with the click")

    app.abandonSession()
  }

  /// A quaver metre chosen in the sheet has to reach the hand-off, or the
  /// reflection asks for a crotchet tempo the player never played (#1499).
  func testTheReflectionCountsInTheClicksUnit() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()
    app.startOneItemSession()

    let start = app.buttons["Start the click"]
    XCTAssertTrue(start.waitForExistence(timeout: 10), "the click row is up")
    start.tap()

    let bar = app.buttons["Bar"]
    XCTAssertTrue(bar.waitForExistence(timeout: 5), "the bar line appears while sounding")
    bar.tap()

    let compound = app.buttons["6/8"]
    XCTAssertTrue(compound.waitForExistence(timeout: 5), "the metre presets are offered")
    compound.tap()
    app.buttons["Done"].tap()

    XCTAssertTrue(bar.waitForExistence(timeout: 5))
    XCTAssertEqual(bar.value as? String, "6 quaver beats, click on every beat")

    app.buttons["Finish session"].tap()

    let achieved = app.descendants(matching: .any).matching(identifier: "Achieved tempo").firstMatch
    XCTAssertTrue(achieved.waitForExistence(timeout: 5), "the reflection sheet is up")
    XCTAssertTrue(
      (achieved.value as? String)?.hasSuffix("quaver beats per minute") == true,
      "the stepper counts in the unit the click counted, not crotchets: "
        + "\(achieved.value as? String ?? "no value")")

    app.buttons["Skip rating"].tap()
  }
}
