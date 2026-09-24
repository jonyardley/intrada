import XCTest

/// Finds a control by its `screen.control` identifier and asserts what VoiceOver
/// reads for it (#1950). Repeated rows share an identifier; their label picks one.
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
