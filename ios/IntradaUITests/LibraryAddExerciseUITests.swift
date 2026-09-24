import XCTest

/// Exercises #1616: the Add a piece screen's single "Add exercise" button,
/// where creating and selecting both land in the same staged list.
@MainActor
final class LibraryAddExerciseUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  func testCreatingAndSelectingInThePickerBothLandInTheStagedList() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()

    app.tabBars.buttons["Library"].tap()
    app.control("library.add", spoken: "Add item", timeout: 10).tap()

    let title = app.element("itemForm.title")
    XCTAssertTrue(title.waitForExistence(timeout: 5), "the title field")
    title.tap()
    title.typeText("Nocturne in E flat")

    app.control("itemForm.relatedExercises", spoken: "Related exercises").tap()
    app.control("itemForm.addExercise", spoken: "Add an exercise for this piece").tap()

    app.control("linkedPicker.create", spoken: "Create an exercise").tap()

    let draftTitle = app.element("draftExercise.title")
    XCTAssertTrue(draftTitle.waitForExistence(timeout: 5), "the draft exercise's title field")
    draftTitle.tap()
    draftTitle.typeText("Chromatic run")
    app.control("draftExercise.done", spoken: "Done").tap()

    XCTAssertTrue(
      app.staticTexts["Chromatic run"].waitForExistence(timeout: 5),
      "the fresh draft joins the picker's own list")

    let hanon = app.row("linkedPicker.row", spokenContaining: "Hanon No. 1")
    XCTAssertTrue(hanon.waitForExistence(timeout: 5), "Hanon in the picker's existing list")
    hanon.tap()

    app.control("sheet.done", spoken: "Done").tap()

    XCTAssertTrue(
      app.staticTexts["Chromatic run"].waitForExistence(timeout: 5),
      "the drafted exercise survives the picker closing")
    XCTAssertTrue(
      app.staticTexts["Hanon No. 1"].waitForExistence(timeout: 5),
      "the selected exercise survives the picker closing")
  }
}
