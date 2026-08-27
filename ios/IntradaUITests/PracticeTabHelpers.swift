import XCTest

extension XCUIApplication {
  /// The seeded library has a piece with a related exercise, so the Practice
  /// hero is the Up next suggestion (#1082) and its CTA seeds a block — the
  /// blank builder is behind "Build my own instead".
  @MainActor
  func openEmptyBuilder(file: StaticString = #filePath, line: UInt = #line) {
    let dismiss = buttons["Build my own instead"]
    if dismiss.waitForExistence(timeout: 10) { dismiss.tap() }

    let build = buttons["Start practising"]
    XCTAssertTrue(
      build.waitForExistence(timeout: 10), "Start practising hero button", file: file, line: line)
    build.tap()
  }
}
