import XCTest

/// Kill-and-relaunch drive of the #962 crash-recovery flow: a session started,
/// the process terminated, and the relaunch offering Resume — the one seam
/// (RootView launch wiring + real UserDefaults surviving the kill) that unit
/// and live-bridge tests cannot cover.
@MainActor
final class SessionRecoveryUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  func testKilledSessionOffersResumeOnRelaunch() {
    // Seeded first launch: build a one-item session and start practising.
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()

    app.tabBars.buttons["Practice"].tap()
    let start = app.buttons["Start practising"]
    XCTAssertTrue(start.waitForExistence(timeout: 10), "Start practising")
    start.tap()

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

    let skip = app.buttons["Skip this item"]
    XCTAssertTrue(skip.waitForExistence(timeout: 10), "the focus player is up")

    // Kill mid-session — the crash this feature exists for.
    app.terminate()

    // Relaunch WITHOUT seeding (a seeded launch skips recovery on purpose).
    let relaunch = XCUIApplication()
    relaunch.launchArguments = ["--disable-animations"]
    relaunch.launch()

    relaunch.tabBars.buttons["Practice"].tap()
    let resume = relaunch.buttons["Resume the interrupted session"]
    XCTAssertTrue(resume.waitForExistence(timeout: 10), "the recovery prompt offers Resume")
    resume.tap()

    XCTAssertTrue(
      relaunch.buttons["Skip this item"].waitForExistence(timeout: 10),
      "Resume reopens the focus player on the interrupted session")

    // Leave the container clean for the next test: abandon via Session options.
    relaunch.buttons["Session options"].tap()
    let end = relaunch.buttons["End session early"]
    if end.waitForExistence(timeout: 3) {
      end.tap()
      let discard = relaunch.buttons["Discard"]
      if discard.waitForExistence(timeout: 5) { discard.tap() }
    }
  }
}
