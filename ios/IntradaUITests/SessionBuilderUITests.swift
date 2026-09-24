import XCTest

/// Real-bridge UITest for the "Build session" builder (#935). `--disable-animations`
/// stops the Practice week-strip's paging TabView defeating XCUITest idle (#941).
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
    app.openEmptyBuilder()

    app.control("builder.addItems", spoken: "Add piece or exercise").tap()

    // In the sheet, add the top two cards ("Not added" / "Added" a11y value).
    let notAdded = app.descendants(matching: .any).matching(
      NSPredicate(format: "identifier == %@ AND value == %@", "libraryPicker.row", "Not added"))
    XCTAssertTrue(notAdded.firstMatch.waitForExistence(timeout: 5), "Library cards in sheet")
    notAdded.firstMatch.tap()
    XCTAssertTrue(notAdded.firstMatch.waitForExistence(timeout: 5), "A second card to add")
    notAdded.firstMatch.tap()
    app.control("sheet.done", spoken: "Done").tap()

    // Both land as standalone rows, each with a remove button spoken "Remove <title>".
    let removes = app.rows("builder.remove", spokenContaining: "Remove ")
    XCTAssertTrue(removes.firstMatch.waitForExistence(timeout: 5), "queued items")
    XCTAssertEqual(removes.count, 2, "two items queued")

    removes.firstMatch.tap()
    XCTAssertEqual(
      app.rows("builder.remove", spokenContaining: "Remove ").count, 1,
      "one item remains after removing from the queue")

    let cancel = app.control("builder.cancel", spoken: "Cancel")
    cancel.tap()
    XCTAssertTrue(app.alerts.buttons["Discard"].waitForExistence(timeout: 5), "Cancel confirms")
    app.alerts.buttons["Keep editing"].tap()
    XCTAssertTrue(cancel.waitForExistence(timeout: 5), "Keep editing stays in the builder")

    cancel.tap()
    app.alerts.buttons["Discard"].tap()
    app.control("practice.start", spoken: "Start practising")
  }

  /// Direct manipulation: top-level units reorder by long-press drag with NO
  /// Edit-mode round trip (the design's always-available reorder).
  func testTopLevelDragReorderWithoutEditMode() {
    let app = launchSeeded()

    app.tabBars.buttons["Practice"].tap()
    app.openEmptyBuilder()

    app.control("builder.addItems", spoken: "Add piece or exercise").tap()

    let hanonCard = app.row("libraryPicker.row", spokenContaining: "Hanon No. 1")
    XCTAssertTrue(hanonCard.waitForExistence(timeout: 5), "Hanon card in sheet")
    hanonCard.tap()
    app.row("libraryPicker.row", spokenContaining: "Major Scales").tap()
    app.control("sheet.done", spoken: "Done").tap()

    // Builder rows are combined a11y elements labelled "<title>, Standalone …".
    let hanonRow = builderRow(app, titled: "Hanon No. 1", meta: "Standalone")
    let scalesRow = builderRow(app, titled: "Major Scales", meta: "Standalone")
    XCTAssertTrue(hanonRow.waitForExistence(timeout: 5), "Hanon queued")
    XCTAssertTrue(scalesRow.exists, "Scales queued")
    XCTAssertLessThan(hanonRow.frame.minY, scalesRow.frame.minY, "Hanon starts above Scales")

    // No Edit tap. The drop must land INSIDE the target row — past its bottom
    // edge sits the move-disabled Add row, and a drop there cancels the move.
    let flipped = dragUntilSettled(hanonRow, onto: scalesRow, targetDy: 0.75) {
      scalesRow.exists && hanonRow.exists && scalesRow.frame.minY < hanonRow.frame.minY
    }
    XCTAssertTrue(flipped, "long-press drag reorders without entering Edit mode")
  }

  /// Long-press-drags `mover` onto `target` until `settled` holds, retrying up
  /// to 3 times: the shared CI runners are slow enough that the List's lift
  /// occasionally misses the press window, so a single-shot drag flakes.
  private func dragUntilSettled(
    _ mover: XCUIElement, onto target: XCUIElement, targetDy: CGFloat,
    settled: () -> Bool
  ) -> Bool {
    for _ in 0..<3 {
      let from = mover.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5))
      let to = target.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: targetDy))
      from.press(forDuration: 1.2, thenDragTo: to, withVelocity: .slow, thenHoldForDuration: 0.4)
      for _ in 0..<12 {
        if settled() { return true }
        usleep(250_000)
      }
    }
    return settled()
  }

  /// Builder rows combine their title + meta into one labelled element, spoken
  /// "<title>, Standalone …" or "<title>, Related …".
  private func builderRow(_ app: XCUIApplication, titled title: String, meta: String)
    -> XCUIElement
  {
    app.descendants(matching: .any).matching(
      NSPredicate(
        format: "identifier == %@ AND label BEGINSWITH %@ AND label CONTAINS %@", "builder.row",
        title, meta)
    ).firstMatch
  }

  /// Grouped block, end to end: link exercises to a piece, add the piece (block
  /// forms), then tap a nested row to open its settings. The core's side of a
  /// nested reorder is pinned in `SessionBridgeTests` (#2003).
  func testNestedRowTapOpensEntrySettings() {
    let app = launchSeeded()

    app.tabBars.buttons["Library"].tap()
    let clairRow = app.row("library.row", spokenContaining: "Clair de Lune")
    XCTAssertTrue(clairRow.waitForExistence(timeout: 10), "Clair library row")
    clairRow.tap()
    // The empty state's one action opens the picker (#1616).
    app.control(
      "libraryDetail.addExercise", spoken: "Add an exercise for this piece", timeout: 10
    ).tap()
    let hanonPick = app.row("linkedPicker.row", spokenContaining: "Hanon No. 1")
    XCTAssertTrue(hanonPick.waitForExistence(timeout: 10), "Hanon in picker")
    hanonPick.tap()
    app.row("linkedPicker.row", spokenContaining: "Major Scales").tap()
    app.control("sheet.done", spoken: "Done").tap()

    app.tabBars.buttons["Practice"].tap()
    app.openEmptyBuilder()
    app.control("builder.addItems", spoken: "Add piece or exercise").tap()
    let clairCard = app.row("libraryPicker.row", spokenContaining: "Clair de Lune")
    XCTAssertTrue(clairCard.waitForExistence(timeout: 10), "Clair card in sheet")
    clairCard.tap()
    app.control("sheet.done", spoken: "Done").tap()

    let hanonRow = builderRow(app, titled: "Hanon No. 1", meta: "Related")
    let scalesRow = builderRow(app, titled: "Major Scales", meta: "Related")
    XCTAssertTrue(hanonRow.waitForExistence(timeout: 10), "nested Hanon row")
    XCTAssertTrue(scalesRow.exists, "nested Scales row")

    // Row tap opens the entry settings sheet (Toggle labels surface as switches).
    hanonRow.tap()
    app.control("entrySettings.trackReps", spoken: "Track repetitions")
  }
}
