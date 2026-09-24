import XCTest

/// The bar line appears only while the click sounds, the sheet it opens changes
/// which beats sound, and the metre chosen there reaches the hand-off. The
/// engine tests cover the grid; this covers the wiring from the row to the
/// sheet and on to the reflection.
///
/// One launch, not one per assertion: the UI tier is a merge gate, and each
/// pass through the builder costs minutes on CI (#1207's UI half caps at 20).
@MainActor
final class ClickBarUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  func testTheBarFollowsTheSheetAndTheMetreReachesTheReflection() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()
    app.startOneItemSession()

    let start = app.control("click.toggle", spoken: "Start the metronome", timeout: 10)
    XCTAssertFalse(app.element("click.bar").exists, "no bar line while silent")
    start.tap()

    let bar = app.control("click.bar", spoken: "Bar")
    XCTAssertEqual(bar.value as? String, "4 crotchet beats, metronome on every beat")

    // A pattern changes which beats sound, and nothing else.
    bar.tap()
    let done = app.control("sheet.done", spoken: "Done")
    // BUG: a pause between choosing a pattern and Done leaves the bar line
    // unable to reopen the sheet, so Done follows the pattern at once (#1950).
    app.control("clickSheet.pattern.downbeat", spoken: "Downbeat").tap()
    done.tap()
    XCTAssertTrue(bar.waitForExistence(timeout: 5))
    XCTAssertEqual(bar.value as? String, "4 crotchet beats, metronome on beat 1")

    // A new metre reads in its own unit and starts sounding every beat again.
    bar.tap()
    app.control("clickSheet.metre.6-8", spoken: "6/8").tap()
    app.control("sheet.done", spoken: "Done").tap()
    XCTAssertTrue(bar.waitForExistence(timeout: 5))
    XCTAssertEqual(bar.value as? String, "6 quaver beats, metronome on every beat")

    app.control("click.toggle", spoken: "Stop the metronome").tap()
    XCTAssertFalse(
      app.element("click.bar").waitForExistence(timeout: 2), "the bar line leaves with the click")

    // The quaver metre has to survive the hand-off, or the reflection asks for
    // a crotchet tempo the player never played (#1499).
    app.control("player.advance", spoken: "Finish session").tap()
    let achieved = app.control("reflection.tempo", spoken: "Achieved tempo")
    XCTAssertTrue(
      (achieved.value as? String)?.hasSuffix("quaver beats per minute") == true,
      "the stepper counts in the unit the click counted, not crotchets: "
        + "\(achieved.value as? String ?? "no value")")

    // A swipe must not throw the marks and the note away (#1934).
    app.swipeDown()
    app.swipeDown()
    XCTAssertTrue(app.element("reflection.skip").exists, "a swipe leaves the reflection sheet up")

    app.control("reflection.skip", spoken: "Skip rating").tap()
    app.discardSummary()
  }
}
