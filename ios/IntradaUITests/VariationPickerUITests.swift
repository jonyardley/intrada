import XCTest

/// Drives the player's variation picker against the real bridge (#1739): the
/// item-complete sheet offers a score per variation, each a `ScoreStepper`
/// whose rings sit behind one accessibility element, so only a real rendered
/// view can tap them. "Major Scales" is the seeded exercise whose one
/// variation is "Hands separately" (`sample.rs`); its keys start at C major,
/// so a play opens in C major with no variation (#2788).
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

  /// A play under five seconds with no mark is a stray tap the core drops,
  /// so each stretch runs past it to be offered a mark (#2791).
  private func playLongEnoughToMark() {
    usleep(5_500_000)
  }

  /// Minus and plus are 44 points each side of a 64 point number.
  private func tapRing(_ stepper: XCUIElement, raise: Bool) {
    stepper.coordinate(
      withNormalizedOffset: CGVector(dx: raise ? 130.0 / 152 : 22.0 / 152, dy: 0.5)
    )
    .tap()
  }

  /// The plain stretch's name, which opens its sheet; the hands stretch reads "C major · …".
  private func keyRow(_ app: XCUIApplication) -> XCUIElement {
    app.buttons.matching(
      NSPredicate(format: "identifier == %@ AND label BEGINSWITH %@", "reflection.key", "C major,")
    ).firstMatch
  }

  /// The stepper reads "5, last time": the score comes first.
  private func score(_ stepper: XCUIElement) -> Int {
    Int((stepper.value as? String)?.split(separator: ",").first ?? "") ?? 0
  }

  func testTheItemCompleteSheetOffersAMarkPerVariation() {
    let app = startScalesSession()

    XCTAssertTrue(
      app.buttons["player.advance"].waitForExistence(timeout: 10), "the player is up")
    playLongEnoughToMark()
    pick(app, "Hands separately")
    playLongEnoughToMark()

    app.control("player.advance", spoken: "Finish session").tap()

    let markPlain = app.mark("reflection.mark", "Score for C major")
    XCTAssertTrue(
      markPlain.waitForExistence(timeout: 10), "the plain stretch has its own mark")
    let markHands = app.mark("reflection.mark", "Score for C major \u{00B7} Hands separately")
    XCTAssertTrue(markHands.exists, "and so does the stretch hands separately")

    // One raised and one lowered from the same start, so one write cannot
    // satisfy both, then save: the sheet's marks only reach the core once the
    // item is completed, which is the path a "Skip" test never covers.
    tapRing(markPlain, raise: true)
    tapRing(markHands, raise: false)
    let plain = score(markPlain)
    let hands = score(markHands)
    XCTAssertNotEqual(plain, hands, "the two stretches hold different scores")

    // The key's sheet applies a typed tempo and a note on Done, even with
    // the number pad still up (#3004).
    keyRow(app).tap()
    let note = app.element("reflection.keyNote")
    XCTAssertTrue(note.waitForExistence(timeout: 5), "the key's sheet is up")
    note.tap()
    app.typeText("Even at the top")
    app.element("reflection.tempo").tap()
    app.typeText(String(repeating: XCUIKeyboardKey.delete.rawValue, count: 3) + "120")
    app.control("sheet.done", spoken: "Done").tap()
    let row = keyRow(app)
    XCTAssertTrue(row.waitForExistence(timeout: 5), "back on the item-complete sheet")
    XCTAssertTrue(row.label.contains("120"), "the typed tempo reached the row: \(row.label)")
    XCTAssertTrue(
      row.label.hasSuffix("note: Even at the top"), "the note reached the row: \(row.label)")

    app.control("reflection.save", spoken: "Save & continue").tap()

    // The summary is the core's answer: each variation kept the mark it was
    // given, rather than one overwriting the other.
    let summaryPlain = app.mark("summary.mark", "Confidence for Major Scales, C major")
    XCTAssertTrue(
      summaryPlain.waitForExistence(timeout: 10), "the summary marks each variation separately")
    XCTAssertEqual(
      summaryPlain.value as? String, "\(plain) of 10", "the plain play kept the score it was given")
    XCTAssertEqual(
      app.mark("summary.mark", "Confidence for Major Scales, C major \u{00B7} Hands separately")
        .value as? String,
      "\(hands) of 10",
      "and hands separately kept its own rather than the plain one's")

    app.discardSummary()
  }
}

extension XCUIApplication {
  /// Each play's mark shares one identifier, so its spoken label is the whole match.
  fileprivate func mark(_ identifier: String, _ label: String) -> XCUIElement {
    descendants(matching: .any).matching(
      NSPredicate(format: "identifier == %@ AND label == %@", identifier, label)
    ).firstMatch
  }
}
