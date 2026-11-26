import XCTest

/// The plan from the time the musician has (#3007): Start plays it in one tap,
/// and Change opens it in the builder, filled.
@MainActor
final class PracticeSuggestionUITests: XCTestCase {
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

  func testStartPlaysThePlanInOneTap() {
    let app = launchSeeded()
    app.tabBars.buttons["Practice"].tap()

    app.control("practice.plan.start", spoken: "Start session", timeout: 10).tap()

    XCTAssertTrue(
      app.element("player.skip").waitForExistence(timeout: 10),
      "one tap lands in the player, with no builder in between")
    // An in-progress session outlives the app in UserDefaults; a later
    // unseeded launch would offer to resume it.
    app.abandonSession()
  }

  func testChangeOpensThePlanInTheBuilder() {
    let app = launchSeeded()
    app.tabBars.buttons["Practice"].tap()

    app.control("practice.plan.change", spoken: "Change", timeout: 10).tap()

    XCTAssertTrue(
      app.element("builder.row").waitForExistence(timeout: 10),
      "the builder opens holding the plan, not empty")
  }
}
