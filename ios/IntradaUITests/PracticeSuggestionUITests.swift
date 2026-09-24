import XCTest

/// The Up next suggestion's own two escape hatches: opening the builder in one
/// tap when you say you'd rather build your own (#1617), and coming back to
/// the suggestion afterwards rather than losing it for the app run (#1618).
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

  func testBuildOwnInsteadOpensBuilderInOneTap() {
    let app = launchSeeded()
    app.tabBars.buttons["Practice"].tap()

    app.openEmptyBuilder()

    XCTAssertTrue(
      app.element("builder.addItems").waitForExistence(timeout: 10),
      "one tap lands in the builder, with no second tap on Start practising")
  }

  func testSuggestionCanBeRestoredWithoutRelaunching() {
    let app = launchSeeded()
    app.tabBars.buttons["Practice"].tap()

    app.openEmptyBuilder()
    app.control("builder.addItems", spoken: "Add piece or exercise", timeout: 10)

    app.control("builder.cancel", spoken: "Cancel", timeout: 10).tap()

    app.control("practice.showSuggestion", spoken: "Show suggestion", timeout: 10).tap()

    XCTAssertTrue(
      app.element("practice.buildOwn").waitForExistence(timeout: 10),
      "the suggestion is back without relaunching")
  }
}
