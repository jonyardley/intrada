import XCTest

extension XCUIApplication {
  /// The seeded library has a piece with a related exercise, so the Practice
  /// hero is the Up next suggestion (#1082); "Build my own instead" opens the
  /// blank builder directly in one tap (#1617).
  @MainActor
  func openEmptyBuilder(file: StaticString = #filePath, line: UInt = #line) {
    control(
      "practice.buildOwn", spoken: "Build my own instead", timeout: 10, file: file, line: line
    )
    .tap()
  }

  /// The shortest route to the focus player: one seeded library item in a
  /// session, started. Two suites drive it, so it lives here.
  @MainActor
  func startOneItemSession(file: StaticString = #filePath, line: UInt = #line) {
    tabBars.buttons["Practice"].tap()
    openEmptyBuilder(file: file, line: line)

    control("builder.addItems", spoken: "Add piece or exercise", file: file, line: line).tap()
    let notAdded = descendants(matching: .any).matching(
      NSPredicate(format: "identifier == %@ AND value == %@", "libraryPicker.row", "Not added"))
    XCTAssertTrue(
      notAdded.firstMatch.waitForExistence(timeout: 5), "Library cards in sheet", file: file,
      line: line)
    notAdded.firstMatch.tap()
    control("sheet.done", spoken: "Done", file: file, line: line).tap()

    control("builder.start", spoken: "Start session", file: file, line: line).tap()
  }

  /// Abandons the running session so the next test starts from a clean
  /// container: an in-progress session outlives the app in UserDefaults.
  @MainActor
  func abandonSession(file: StaticString = #filePath, line: UInt = #line) {
    endSessionEarly(file: file, line: line)
    discardSummary(file: file, line: line)
  }

  /// Same job for a session that reached the summary: only `SaveSession` and
  /// `DiscardSession` clear the in-progress blob, and reaching the summary
  /// clears nothing.
  @MainActor
  func discardSummary(file: StaticString = #filePath, line: UInt = #line) {
    control("summary.discard", spoken: "Discard", timeout: 10, file: file, line: line).tap()
    let confirm = alerts.buttons["Discard"]
    XCTAssertTrue(
      confirm.waitForExistence(timeout: 5), "the discard confirmation", file: file, line: line)
    confirm.tap()
  }

  /// Saves a session that reached the summary, which is the only way to leave a
  /// card in Practice history for a test to open (#1371).
  @MainActor
  func saveSummary(file: StaticString = #filePath, line: UInt = #line) {
    control("summary.save", spoken: "Save session", timeout: 10, file: file, line: line).tap()
  }

  /// Ends a running session early, which lands on the summary.
  @MainActor
  func endSessionEarly(file: StaticString = #filePath, line: UInt = #line) {
    control("player.options", spoken: "Session options", file: file, line: line).tap()
    let end = buttons["End session early"]
    XCTAssertTrue(
      end.waitForExistence(timeout: 5), "End session early", file: file, line: line)
    end.tap()
  }
}
