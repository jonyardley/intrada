import XCTest

/// Controls are found by `accessibilityIdentifier`, so a copy change cannot break
/// the merge gate, and every control a test drives has its spoken label asserted,
/// so VoiceOver reading the wrong words (or nothing) fails it instead (#1950).
///
/// Identifiers read `screen.control`: `builder.addItems`, `player.skip`. Rows that
/// repeat share one identifier, and their label picks one out. Tab bar items,
/// alert buttons and menu items are drawn by the system and keep their words.
extension XCUIApplication {
  @MainActor
  @discardableResult
  func control(
    _ identifier: String, spoken label: String, timeout: TimeInterval = 5,
    file: StaticString = #filePath, line: UInt = #line
  ) -> XCUIElement {
    let any = descendants(matching: .any).matching(identifier: identifier).firstMatch
    XCTAssertTrue(
      any.waitForExistence(timeout: timeout), "\(identifier) is on screen", file: file,
      line: line)
    // A toolbar item reports a wrapper around its button under the same
    // identifier; the tap goes to the button.
    let matches = buttons.matching(identifier: identifier).allElementsBoundByIndex
    let element = matches.first(where: \.isHittable) ?? matches.first ?? any
    XCTAssertEqual(
      element.label, label, "what VoiceOver reads for \(identifier)", file: file, line: line)
    return element
  }

  func row(_ identifier: String, spokenContaining text: String) -> XCUIElement {
    rows(identifier, spokenContaining: text).firstMatch
  }

  func rows(_ identifier: String, spokenContaining text: String) -> XCUIElementQuery {
    descendants(matching: .any).matching(
      NSPredicate(format: "identifier == %@ AND label CONTAINS %@", identifier, text))
  }

  func element(_ identifier: String) -> XCUIElement {
    descendants(matching: .any).matching(identifier: identifier).firstMatch
  }
}
