import XCTest

/// Real-bridge UITest for the "Build session" builder (#935). Adding moved into
/// the "Add to session" sheet. `--disable-animations` stops the Practice
/// week-strip's paging TabView defeating XCUITest idle (#941).
@MainActor
final class SessionBuilderUITests: XCTestCase {
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

  func testBuildSetlistAddRemoveThenCancel() {
    let app = launchSeeded()

    app.tabBars.buttons["Practice"].tap()
    let start = app.buttons["Start practising"]
    XCTAssertTrue(start.waitForExistence(timeout: 10), "Start practising")
    start.tap()

    // Adding moved to the "Add to session" sheet — open it from the dashed row.
    let addRow = app.buttons["Add piece or exercise"]
    XCTAssertTrue(addRow.waitForExistence(timeout: 5), "Add row")
    addRow.tap()

    // In the sheet, add the top two cards ("Not added" / "Added" a11y value).
    let notAdded = app.buttons.matching(NSPredicate(format: "value == %@", "Not added"))
    XCTAssertTrue(notAdded.firstMatch.waitForExistence(timeout: 5), "Library cards in sheet")
    notAdded.firstMatch.tap()
    XCTAssertTrue(notAdded.firstMatch.waitForExistence(timeout: 5), "A second card to add")
    notAdded.firstMatch.tap()
    app.buttons["Done"].tap()

    // Both land as standalone rows (their remove buttons are "Remove <title>").
    let removes = app.buttons.matching(NSPredicate(format: "label BEGINSWITH %@", "Remove"))
    XCTAssertTrue(removes.firstMatch.waitForExistence(timeout: 5), "queued items")
    XCTAssertEqual(removes.count, 2, "two items queued")

    removes.firstMatch.tap()
    XCTAssertEqual(
      app.buttons.matching(NSPredicate(format: "label BEGINSWITH %@", "Remove")).count, 1,
      "one item remains after removing from the queue")

    let cancel = app.buttons["Cancel"]
    cancel.tap()
    XCTAssertTrue(app.buttons["Discard"].waitForExistence(timeout: 5), "Cancel confirms")
    app.buttons["Keep editing"].tap()
    XCTAssertTrue(cancel.waitForExistence(timeout: 5), "Keep editing stays in the builder")

    app.buttons["Cancel"].tap()
    app.buttons["Discard"].tap()
    XCTAssertTrue(start.waitForExistence(timeout: 5), "Discard returns to Practice")
  }

  /// Direct manipulation: top-level units reorder by long-press drag with NO
  /// Edit-mode round trip (the design's always-available reorder).
  func testTopLevelDragReorderWithoutEditMode() {
    let app = launchSeeded()

    app.tabBars.buttons["Practice"].tap()
    let start = app.buttons["Start practising"]
    XCTAssertTrue(start.waitForExistence(timeout: 10), "Start practising")
    start.tap()

    let addRow = app.buttons["Add piece or exercise"]
    XCTAssertTrue(addRow.waitForExistence(timeout: 5), "Add row")
    addRow.tap()

    let hanonCard = app.buttons.matching(
      NSPredicate(format: "label CONTAINS %@", "Hanon No. 1")
    ).firstMatch
    XCTAssertTrue(hanonCard.waitForExistence(timeout: 5), "Hanon card in sheet")
    hanonCard.tap()
    app.buttons.matching(NSPredicate(format: "label CONTAINS %@", "Major Scales")).firstMatch.tap()
    app.buttons["Done"].tap()

    // Builder rows are combined a11y elements labelled "<title>, Standalone …".
    let hanonRow = builderRow(app, titled: "Hanon No. 1", meta: "Standalone")
    let scalesRow = builderRow(app, titled: "Major Scales", meta: "Standalone")
    XCTAssertTrue(hanonRow.waitForExistence(timeout: 5), "Hanon queued")
    XCTAssertTrue(scalesRow.exists, "Scales queued")
    XCTAssertLessThan(hanonRow.frame.minY, scalesRow.frame.minY, "Hanon starts above Scales")

    // No Edit tap. Long-press the first row, drag it below the second. The
    // drop must land INSIDE the target row — past its bottom edge sits the
    // move-disabled Add row, and a drop there cancels the whole move.
    let from = hanonRow.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5))
    let to = scalesRow.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.75))
    from.press(forDuration: 1.2, thenDragTo: to, withVelocity: .slow, thenHoldForDuration: 0.4)

    XCTAssertTrue(scalesRow.waitForExistence(timeout: 5), "Scales still visible after drag")
    XCTAssertLessThan(
      scalesRow.frame.minY, hanonRow.frame.minY,
      "long-press drag reorders without entering Edit mode")
  }

  /// Builder rows combine their title + meta into one labelled element. Match
  /// on title AND meta ("Standalone" / "Related"): the add-sheet cards prefix
  /// their labels with the item type, and the Library tab's hierarchy stays
  /// queryable while hidden, so a bare title BEGINSWITH can resolve to an
  /// offscreen library row and derive coordinates from the wrong screen.
  private func builderRow(_ app: XCUIApplication, titled title: String, meta: String)
    -> XCUIElement
  {
    app.buttons.matching(
      NSPredicate(format: "label BEGINSWITH %@ AND label CONTAINS %@", title, meta)
    ).firstMatch
  }

  /// Grouped block, end to end: link exercises to a piece, add the piece (block
  /// forms), long-press-drag a nested exercise to reorder it with NO Edit mode
  /// (each flattened row lifts natively), then tap the row to open its settings.
  func testNestedGripDragReordersAndRowTapOpensSettings() {
    let app = launchSeeded()

    app.tabBars.buttons["Library"].tap()
    let clairRow = app.buttons.matching(
      NSPredicate(format: "label CONTAINS %@", "Clair de Lune")
    ).firstMatch
    XCTAssertTrue(clairRow.waitForExistence(timeout: 10), "Clair library row")
    clairRow.tap()
    let addRelated = app.buttons["Add a related exercise to this piece"]
    XCTAssertTrue(addRelated.waitForExistence(timeout: 5), "related empty-state CTA")
    addRelated.tap()
    let hanonPick = app.buttons.matching(
      NSPredicate(format: "label CONTAINS %@", "Hanon No. 1")
    ).firstMatch
    XCTAssertTrue(hanonPick.waitForExistence(timeout: 5), "Hanon in picker")
    hanonPick.tap()
    app.buttons.matching(NSPredicate(format: "label CONTAINS %@", "Major Scales")).firstMatch
      .tap()
    app.buttons["Done"].tap()

    app.tabBars.buttons["Practice"].tap()
    let start = app.buttons["Start practising"]
    XCTAssertTrue(start.waitForExistence(timeout: 10), "Start practising")
    start.tap()
    let addRow = app.buttons["Add piece or exercise"]
    XCTAssertTrue(addRow.waitForExistence(timeout: 5), "Add row")
    addRow.tap()
    let clairCard = app.buttons.matching(
      NSPredicate(format: "label CONTAINS %@", "Clair de Lune")
    ).firstMatch
    XCTAssertTrue(clairCard.waitForExistence(timeout: 5), "Clair card in sheet")
    clairCard.tap()
    app.buttons["Done"].tap()

    let hanonRow = builderRow(app, titled: "Hanon No. 1", meta: "Related")
    let scalesRow = builderRow(app, titled: "Major Scales", meta: "Related")
    XCTAssertTrue(hanonRow.waitForExistence(timeout: 5), "nested Hanon row")
    XCTAssertTrue(scalesRow.exists, "nested Scales row")

    // The linked order isn't deterministic (the picker applies a Set), so drag
    // whichever nested row is lower above the other and assert the flip. The
    // drop lands INSIDE the target row's upper half — each flattened row lifts
    // natively, exactly like the top-level test.
    let hanonFirst = hanonRow.frame.minY < scalesRow.frame.minY
    let upper = hanonFirst ? hanonRow : scalesRow
    let lower = hanonFirst ? scalesRow : hanonRow
    let from = lower.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5))
    let to = upper.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.25))
    from.press(forDuration: 1.2, thenDragTo: to, withVelocity: .slow, thenHoldForDuration: 0.4)

    XCTAssertTrue(lower.waitForExistence(timeout: 5), "dragged row visible after drag")
    XCTAssertLessThan(
      lower.frame.minY, upper.frame.minY, "long-press drag reorders nested rows without Edit")

    // Row tap opens the entry settings sheet (Toggle labels surface as switches).
    hanonRow.tap()
    XCTAssertTrue(
      app.descendants(matching: .any)["Track reps"].firstMatch.waitForExistence(timeout: 5),
      "settings sheet opens")
  }
}
