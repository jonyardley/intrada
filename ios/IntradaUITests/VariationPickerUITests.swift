import XCTest

/// Drives the player's variation picker against the real bridge (#1739): the
/// item-complete sheet offers a mark per variation, proved through
/// `ScoreSelector`'s coordinate-based pill taps, which only a real rendered
/// view can exercise. "Major Scales" is the seeded exercise whose
/// variations are deterministic (`C`, `G`, `D`, `A`, `E`; see `app.rs`'s
/// `LoadSampleData` seed).
///
/// #1825 moved the chip/rep-count switch assertion to `VariationPlayBridgeTests`.
@MainActor
final class VariationPickerUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  private func startScalesSession() -> XCUIApplication {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()

    app.tabBars.buttons["Practice"].tap()
    app.openEmptyBuilder()

    app.control("builder.addItems", spoken: "Add piece or exercise", timeout: 10).tap()

    let scalesCard = app.descendants(matching: .any).matching(
      NSPredicate(
        format: "identifier == %@ AND label CONTAINS %@ AND value == %@", "libraryPicker.row",
        "Major Scales", "Not added")
    ).firstMatch
    XCTAssertTrue(scalesCard.waitForExistence(timeout: 10), "Major Scales card in the add sheet")
    scalesCard.tap()
    app.control("sheet.done", spoken: "Done").tap()

    app.control("builder.start", spoken: "Start session").tap()
    return app
  }

  private func pick(_ app: XCUIApplication, _ label: String) {
    app.control("player.variation", spoken: "Variation", timeout: 10).tap()

    let row = app.descendants(matching: .any).matching(
      NSPredicate(
        format: "identifier == %@ AND label BEGINSWITH %@", "variationPicker.row", "\(label),")
    ).firstMatch
    XCTAssertTrue(row.waitForExistence(timeout: 5), "\(label) row in the picker")
    row.tap()
  }

  private func tapPill(_ selector: XCUIElement, _ value: Int) {
    selector.coordinate(
      withNormalizedOffset: CGVector(dx: (Double(value) - 0.5) / 10, dy: 0.5)
    ).tap()
  }

  func testTheItemCompleteSheetOffersAMarkPerVariation() {
    let app = startScalesSession()

    pick(app, "C")
    app.control("player.gotIt", spoken: "Got it").tap()
    pick(app, "G")
    app.control("player.gotIt", spoken: "Got it").tap()

    app.control("player.advance", spoken: "Finish session").tap()

    let markC = app.mark("reflection.mark", "Mark for C")
    XCTAssertTrue(
      markC.waitForExistence(timeout: 10), "the stretch played in C has its own mark")
    let markG = app.mark("reflection.mark", "Mark for G")
    XCTAssertTrue(markG.exists, "and so does the stretch played in G")

    // Two different marks, so one write cannot satisfy both, then save: the
    // sheet's marks only reach the core once the item is completed, which is
    // the path a "Skip rating" test never covers. ScoreSelector hides its ten
    // pills behind one accessibility element, so the taps go by position:
    // pill N's centre sits at (N - 0.5) / 10 across the row.
    tapPill(markC, 7)
    tapPill(markG, 3)

    app.control("reflection.save", spoken: "Save & continue").tap()

    // The summary is the core's answer: each variation kept the mark it was
    // given, rather than one overwriting the other.
    let summaryC = app.mark("summary.mark", "Mark for Major Scales, C")
    XCTAssertTrue(
      summaryC.waitForExistence(timeout: 10), "the summary marks each variation separately")
    XCTAssertEqual(summaryC.value as? String, "7 of 10", "C kept the seven it was given")
    XCTAssertEqual(
      app.mark("summary.mark", "Mark for Major Scales, G").value as? String, "3 of 10",
      "and G kept its three rather than C's seven")

    app.discardSummary()
  }
}

extension XCUIApplication {
  /// Each play's mark shares one identifier, so its spoken label is the whole match.
  fileprivate func mark(_ identifier: String, _ label: String) -> XCUIElement {
    otherElements.matching(
      NSPredicate(format: "identifier == %@ AND label == %@", identifier, label)
    ).firstMatch
  }
}
