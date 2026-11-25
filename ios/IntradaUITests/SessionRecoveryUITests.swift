import XCTest

/// Kill-and-relaunch drive of the #962 crash-recovery flow: a session started,
/// the process terminated, and the relaunch offering Resume — the one seam
/// (RootView launch wiring + real UserDefaults surviving the kill) that unit
/// and live-bridge tests cannot cover.
@MainActor
final class SessionRecoveryUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  func testKilledSessionOffersResumeOnRelaunch() {
    // Seeded first launch: build a one-item session and start practising.
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()

    app.startOneItemSession()

    app.control("player.skip", spoken: "Skip this item", timeout: 10)

    // Kill mid-session — the crash this feature exists for.
    app.terminate()

    // Relaunch WITHOUT seeding (a seeded launch skips recovery on purpose).
    let relaunch = XCUIApplication()
    relaunch.launchArguments = ["--disable-animations", "--skip-welcome"]
    relaunch.launch()

    relaunch.tabBars.buttons["Practice"].tap()
    relaunch.control("practice.resume", spoken: "Resume the interrupted session", timeout: 10)
      .tap()

    XCTAssertTrue(
      relaunch.element("player.skip").waitForExistence(timeout: 10),
      "Resume reopens the focus player on the interrupted session")

    // Leave the container clean for the next test.
    relaunch.abandonSession()
  }

  /// The item-complete sheet's answers are in the saved copy, so a kill
  /// before Next brings the sheet back with them (#2137). Animations stay on:
  /// resume presents the player and the sheet in one pass.
  func testKilledSheetComesBackWithItsAnswers() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data"]
    app.launch()

    app.startOneItemSession()
    app.control("player.advance", spoken: "Finish session", timeout: 10).tap()

    let mark = app.element("reflection.mark")
    XCTAssertTrue(mark.waitForExistence(timeout: 10), "the item-complete sheet is up")
    // The score stepper is one element; plus is its trailing ring, minus its leading one.
    mark.coordinate(withNormalizedOffset: CGVector(dx: 0.9, dy: 0.5)).tap()
    if (mark.value as? String)?.hasSuffix(", today") != true {
      mark.coordinate(withNormalizedOffset: CGVector(dx: 0.1, dy: 0.5)).tap()
    }
    let setMark = mark.value as? String
    XCTAssertTrue(setMark?.hasSuffix(", today") == true, "the nudge set today's score")

    // The tempo is one adjustable element; a drag up raises it.
    let tempo = app.element("reflection.tempo")
    let prefilled = tempo.value as? String
    let start = tempo.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5))
    start.press(forDuration: 0.05, thenDragTo: start.withOffset(CGVector(dx: 0, dy: -40)))
    let setTempo = tempo.value as? String
    XCTAssertNotEqual(setTempo, prefilled, "a drag up moved the tempo")

    // Killed with no note typed, so only the mark's and the tempo's own sends can have saved them.
    app.terminate()
    let resumed = relaunchAndResume()

    let resumedMark = resumed.element("reflection.mark")
    XCTAssertTrue(resumedMark.waitForExistence(timeout: 10), "resume reopens the sheet")
    XCTAssertEqual(resumedMark.value as? String, setMark, "the mark survived the kill")
    XCTAssertEqual(
      resumed.element("reflection.tempo").value as? String, setTempo,
      "the tempo set by hand survived the kill")

    let note = resumed.element("reflection.note")
    note.tap()
    note.typeText("Pedal clearer in bar 12")
    XCUIDevice.shared.press(.home)
    resumed.terminate()
    let again = relaunchAndResume()

    XCTAssertTrue(again.element("reflection.mark").waitForExistence(timeout: 10))
    XCTAssertEqual(
      again.element("reflection.note").value as? String, "Pedal clearer in bar 12",
      "the note survived the kill")
    XCTAssertEqual(again.element("reflection.mark").value as? String, setMark)

    again.control("reflection.skip", spoken: "Skip").tap()
    again.discardSummary()
  }

  /// Relaunch without seeding, since a seeded launch skips recovery on purpose.
  private func relaunchAndResume() -> XCUIApplication {
    let relaunch = XCUIApplication()
    relaunch.launchArguments = ["--skip-welcome"]
    relaunch.launch()
    relaunch.tabBars.buttons["Practice"].tap()
    relaunch.control("practice.resume", spoken: "Resume the interrupted session", timeout: 10)
      .tap()
    return relaunch
  }
}
