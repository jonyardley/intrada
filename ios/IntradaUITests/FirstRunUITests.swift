import XCTest

/// A new install meets the welcome once (#2117): through either way out, a
/// relaunch goes straight to the app. Launched without seed data, since a
/// seeded run never counts the store as loaded and so never greets.
@MainActor
final class FirstRunUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  func testSkippingTheProfileIsNotAskedAgain() {
    let app = launchFresh()
    app.control("firstRun.setUpProfile", spoken: "Set up profile", timeout: contendedTimeout).tap()
    app.control("firstRun.skipProfile", spoken: "Skip").tap()
    app.control("firstRun.skipFirstPiece", spoken: "Skip").tap()
    XCTAssertTrue(app.tabBars.buttons["Library"].waitForExistence(timeout: 5), "the app")

    assertRelaunchSkipsTheWelcome(app)
  }

  func testSkippingTheWelcomeIsNotAskedAgain() {
    let app = launchFresh()
    app.control("firstRun.skipWelcome", spoken: "Skip", timeout: contendedTimeout).tap()
    XCTAssertTrue(app.tabBars.buttons["Library"].waitForExistence(timeout: 5), "the app")

    assertRelaunchSkipsTheWelcome(app)
  }

  func testSavingTheProfileStillOffersTheFirstPiece() {
    let app = launchFresh()
    app.control("firstRun.setUpProfile", spoken: "Set up profile", timeout: contendedTimeout).tap()
    // The swatches first: the keyboard pushes the lazy grid off screen.
    app.control("profileEdit.highlighter.mint", spoken: "Mint").tap()
    let name = app.element("profileEdit.name")
    XCTAssertTrue(name.waitForExistence(timeout: 5), "the name field")
    name.tap()
    name.typeText("Jon")
    app.control("firstRun.saveProfile", spoken: "Save profile").tap()
    app.control("firstRun.skipFirstPiece", spoken: "Skip").tap()

    assertRelaunchSkipsTheWelcome(app)
  }

  /// The reset clears the welcome flag with the profile, so the welcome is up
  /// on an empty store whatever an earlier run left.
  private func launchFresh() -> XCUIApplication {
    let app = XCUIApplication()
    app.launchArguments = ["--disable-animations", "--reset-profile"]
    app.launch()
    return app
  }

  private func assertRelaunchSkipsTheWelcome(_ app: XCUIApplication) {
    app.terminate()
    app.launchArguments = ["--disable-animations"]
    app.launch()
    XCTAssertTrue(
      app.tabBars.buttons["Practice"].waitForExistence(timeout: contendedTimeout), "the app")
    app.tabBars.buttons["Practice"].tap()
    XCTAssertTrue(
      app.element("practice.profile").waitForExistence(timeout: 5),
      "Practice is reachable, not under the welcome")
    // The loads land after the tab bar draws, so give the welcome its chance.
    XCTAssertFalse(
      app.element("firstRun.setUpProfile").waitForExistence(timeout: 3),
      "the welcome stays dismissed")
  }
}
