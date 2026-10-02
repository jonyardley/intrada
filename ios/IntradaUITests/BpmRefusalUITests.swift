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

  func testARefusedNewExerciseKeepsTheLinkSheetOpenWithOnlyThatRow() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()

    app.tabBars.buttons["Library"].tap()
    let clairRow = app.row("library.row", spokenContaining: "Clair de Lune")
    XCTAssertTrue(clairRow.waitForExistence(timeout: 10), "Clair library row")
    clairRow.tap()
    app.control(
      "libraryDetail.addExercise", spoken: "Add an exercise for this piece", timeout: 10
    ).tap()

    // The unreadable one first, so an accepted send after it is what would
    // have cleared the core's sentence.
    draft("Bad tempo", bpm: "12a", in: app)
    draft("Good tempo", bpm: "90", in: app)
    app.control("sheet.done", spoken: "Done").tap()

    XCTAssertTrue(
      app.descendants(matching: .any)["Error: BPM must be a whole number between 1 and 400"]
        .waitForExistence(timeout: 5),
      "the core's own sentence, inside the sheet")
    XCTAssertTrue(app.element("sheet.done").exists, "a refused Done leaves the sheet open")
    XCTAssertTrue(
      app.buttons["Remove Bad tempo"].exists, "the refused exercise stays under New")
    XCTAssertFalse(
      app.buttons["Remove Good tempo"].exists, "the accepted one is no longer a new row")
    let good = app.row("linkedPicker.row", spokenContaining: "Good tempo")
    XCTAssertTrue(good.waitForExistence(timeout: 5), "the accepted exercise in the list")
    XCTAssertTrue(good.isSelected, "and ticked, so a second Done keeps it linked")
  }

  private func draft(_ title: String, bpm: String, in app: XCUIApplication) {
    app.control("linkedPicker.create", spoken: "Create an exercise").tap()
    let bpmField = app.element("draftExercise.bpm")
    XCTAssertTrue(bpmField.waitForExistence(timeout: 5), "the new exercise's BPM field")
    bpmField.tap()
    bpmField.typeText(bpm)
    let titleField = app.element("draftExercise.title")
    titleField.tap()
    titleField.typeText(title)
    app.control("draftExercise.done", spoken: "Done").tap()
    XCTAssertTrue(
      app.buttons["Remove \(title)"].waitForExistence(timeout: 5), "\(title) joins under New")
  }
}
