import XCTest

/// Real-app UITest for #1595: a create the core refuses keeps the form open
/// with the banner on it, which is the whole reason marking the fault is worth
/// anything. Driven through the chart, since `ChordChartEditSheet` hands its
/// text back unparsed on the create path and the refusal lands on Add.
@MainActor
final class CreateFaultUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  /// The add form runs past one screen since the type scale rose (#1723), so a
  /// row that used to be in view has to be scrolled to before it takes a tap.
  private func hittable(_ query: XCUIElementQuery, scrolling app: XCUIApplication) -> XCUIElement? {
    for _ in 0..<6 {
      if let match = query.allElementsBoundByIndex.first(where: \.isHittable) { return match }
      app.swipeUp()
    }
    return query.allElementsBoundByIndex.first(where: \.isHittable)
  }

  func testARefusedOnePassCreateKeepsTheFormOpenAndNamesTheFault() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()

    app.tabBars.buttons["Library"].tap()
    app.control("library.add", spoken: "Add item", timeout: 10).tap()

    // The chart first, while nothing has raised the keyboard: with it up, the
    // section rows sit under it and a tap never reaches them.
    app.control("itemForm.chordChart", spoken: "Chord chart", timeout: 10)
    let chartRow = app.descendants(matching: .any).matching(identifier: "itemForm.chordChart")
    let tappable = hittable(chartRow, scrolling: app)
    XCTAssertNotNil(tappable, "one of the chord chart rows takes a tap")
    tappable?.tap()

    let editor = app.control("chordChart.text", spoken: "Chord chart text")
    editor.tap()
    editor.typeText("| Dm7 | G7 |")
    app.control("chordChart.save", spoken: "Save").tap()

    let title = app.element("itemForm.title")
    XCTAssertTrue(title.waitForExistence(timeout: 5), "the title field")
    title.tap()
    title.typeText("Kettle of fish")

    // An over-long composer, so the piece itself is what the core refuses. The
    // create carries a chart, so it goes through the one-pass event, not a
    // plain add.
    let composer = app.element("itemForm.composer")
    XCTAssertTrue(composer.waitForExistence(timeout: 5), "the composer field")
    composer.tap()
    composer.typeText(String(repeating: "x", count: 201))
    app.control("itemForm.confirm", spoken: "Add").tap()

    XCTAssertTrue(
      app.staticTexts["Composer must be between 1 and 200 characters"].waitForExistence(
        timeout: 5),
      "the core's own sentence, on the form rather than behind it")
    XCTAssertTrue(
      app.element("itemForm.confirm").exists,
      "a refused create leaves everything staged on screen, chart included")

    app.control("itemForm.cancel", spoken: "Cancel").tap()
    XCTAssertFalse(
      app.row("library.row", spokenContaining: "Kettle of fish").waitForExistence(timeout: 3),
      "nothing was written, so the library is as it was")
  }
}
