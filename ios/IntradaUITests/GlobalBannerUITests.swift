import XCTest

/// Real-app UITest for #2687: the banner's dismiss cross takes a tap anywhere in
/// its 44 point square, though the drawn cross is about 15 points wide.
@MainActor
final class GlobalBannerUITests: XCTestCase {
  override func setUp() { continueAfterFailure = false }

  // The drawn cross is about 15 points wide and the square's half is 22. Above, 13
  // still lands inside the system's own touch tolerance, so it goes to 20.
  private let leftOfGlyph: CGFloat = 13
  private let aboveGlyph: CGFloat = 20

  /// One test: each dismiss uses up a saved session, and seeded data has none,
  /// so the setup plays two sessions through rather than four.
  func testTheDismissCrossTakesATapBesideAndAboveTheGlyph() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()

    app.startOneItemSession()
    app.endSessionEarly()
    app.saveSummary()
    playASecondSession(app)
    let cards = app.rows("practice.sessionCard", spokenContaining: "1 piece")
    expectCount(cards, 2, "two saved sessions in history")

    var banner = deleteASession(app)
    app.buttons["Undo"].tap()
    XCTAssertTrue(banner.waitForNonExistence(timeout: 5), "Undo clears the banner")
    expectCount(cards, 2, "Undo brings the session back")

    banner = deleteASession(app)
    tapBesideTheCross(app, dx: 0, dy: -aboveGlyph)
    XCTAssertTrue(
      banner.waitForNonExistence(timeout: 5), "a tap above the drawn cross dismisses")
    expectCount(cards, 1, "the dismiss did not undo")

    banner = deleteASession(app)
    tapBesideTheCross(app, dx: -leftOfGlyph, dy: 0)
    XCTAssertTrue(
      banner.waitForNonExistence(timeout: 5), "a tap left of the drawn cross dismisses")
    expectCount(cards, 0, "a tap left of the cross dismisses rather than undoes")
  }

  /// With a session in history the hero is the round Start button, which opens
  /// the empty builder, so "Build my own instead" is gone.
  private func playASecondSession(_ app: XCUIApplication) {
    app.control("practice.start", spoken: "Start practising", timeout: 10).tap()
    app.control("builder.addItems", spoken: "Add piece or exercise", timeout: 10).tap()
    let notAdded = app.descendants(matching: .any).matching(
      NSPredicate(format: "identifier == %@ AND value == %@", "libraryPicker.row", "Not added"))
    XCTAssertTrue(notAdded.firstMatch.waitForExistence(timeout: 5), "Library cards in sheet")
    notAdded.firstMatch.tap()
    app.control("sheet.done", spoken: "Done").tap()
    app.control("builder.start", spoken: "Start session").tap()
    app.endSessionEarly()
    app.saveSummary()
  }

  private func deleteASession(_ app: XCUIApplication) -> XCUIElement {
    let card = app.row("practice.sessionCard", spokenContaining: "1 piece")
    XCTAssertTrue(card.waitForExistence(timeout: 10), "a saved session card")
    card.tap()
    app.control("sessionDetail.delete", spoken: "Delete session", timeout: 10).tap()
    let banner = app.element("globalBanner")
    XCTAssertTrue(banner.waitForExistence(timeout: 10), "the undo banner is up")
    XCTAssertEqual(banner.label, "Note, Session deleted")
    let shot = XCTAttachment(screenshot: app.screenshot())
    shot.lifetime = .keepAlways
    add(shot)
    return banner
  }

  /// The cross's frame is centred on the glyph with or without the wider tap area.
  private func tapBesideTheCross(_ app: XCUIApplication, dx: CGFloat, dy: CGFloat) {
    app.buttons["xmark"].coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5))
      .withOffset(CGVector(dx: dx, dy: dy))
      .tap()
  }

  private func expectCount(_ query: XCUIElementQuery, _ count: Int, _ message: String) {
    let predicate = NSPredicate(format: "count == %d", count)
    let met = XCTNSPredicateExpectation(predicate: predicate, object: query)
    XCTAssertEqual(XCTWaiter().wait(for: [met], timeout: 10), .completed, message)
  }
}
