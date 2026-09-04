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

    app.startOneItemSession()

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

    // Leave the container clean for the next test.
    relaunch.abandonSession()
  }
}
