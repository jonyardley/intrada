import XCTest

/// Exercises #1616 on the piece detail screen: the same single "Add exercise"
/// action, where creating and selecting both land linked to the piece.
@MainActor
final class LibraryDetailAddExerciseUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  func testCreatingAndSelectingInThePickerBothLinkToThePiece() {
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

    // Links immediately on Done, since the piece already exists (#1431).
    XCTAssertTrue(
      app.staticTexts["Chromatic run"].waitForExistence(timeout: 5),
      "the created exercise survives the picker closing and reads as linked")
    XCTAssertTrue(
      app.staticTexts["Hanon No. 1"].waitForExistence(timeout: 5),
      "the selected exercise survives the picker closing and reads as linked")
  }
}
