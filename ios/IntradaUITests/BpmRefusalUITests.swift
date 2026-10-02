import XCTest

/// Real-app UITest for #2224: a BPM the core cannot read is refused on the
/// form, in the core's own sentence, rather than dropped on save.
@MainActor
final class BpmRefusalUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  func testAnUnreadableBpmKeepsTheAddFormOpenWithTheCoresSentence() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()

    app.tabBars.buttons["Library"].tap()
    app.control("library.add", spoken: "Add item", timeout: 10).tap()

    // BPM first, while the keyboard is down and the field is still on screen.
    let bpm = app.element("itemForm.bpm")
    XCTAssertTrue(bpm.waitForExistence(timeout: 5), "the BPM field")
    bpm.tap()
    bpm.typeText("12a")

    let title = app.element("itemForm.title")
    title.tap()
    title.typeText("Kettle of fish")
    app.control("itemForm.confirm", spoken: "Add").tap()

    XCTAssertTrue(
      app.staticTexts["BPM must be a whole number between 1 and 400"].waitForExistence(
        timeout: 5),
      "the core's own sentence, on the form")
    XCTAssertTrue(
      app.element("itemForm.confirm").exists, "a refused add leaves the form on screen")
    XCTAssertEqual(bpm.value as? String, "12a", "the typed BPM is still there to correct")
  }
}
