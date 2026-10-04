import XCTest

/// A section added on the item screen is still there after a relaunch, and
/// one removed in Reorder is gone once Done saves (#2247).
/// Launched without seed data on purpose: seed mode skips persistence, and the
/// relaunch is the point.
@MainActor
final class SectionsUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  func testAnAddedSectionSurvivesARelaunch() {
    let title = "Sections \(UUID().uuidString.prefix(6))"
    let app = XCUIApplication()
    app.launchArguments = ["--disable-animations", "--skip-welcome"]
    app.launch()

    app.tabBars.buttons["Library"].tap()
    app.control("library.add", spoken: "Add item", timeout: 10).tap()
    let titleField = app.element("itemForm.title")
    XCTAssertTrue(titleField.waitForExistence(timeout: 5), "the title field")
    titleField.tap()
    titleField.typeText(title)
    app.control("itemForm.confirm", spoken: "Add").tap()

    openItem(app, title: title)
    app.control("sections.add", spoken: "Add sections to this piece").tap()
    let name = app.element("sectionSheet.name")
    XCTAssertTrue(name.waitForExistence(timeout: 5), "the section's name field")
    name.tap()
    name.typeText("A1")
    let bars = app.element("sectionSheet.bars")
    bars.tap()
    bars.typeText("1 to 16")
    app.control("sheet.done", spoken: "Save").tap()

    let row = app.row("sections.row", spokenContaining: "A1, bars 1 to 16")
    XCTAssertTrue(row.waitForExistence(timeout: 5), "the new section is in the list")

    app.terminate()
    app.launchArguments = ["--disable-animations", "--skip-welcome"]
    app.launch()
    app.tabBars.buttons["Library"].tap()
    openItem(app, title: title)
    XCTAssertTrue(
      app.row("sections.row", spokenContaining: "A1, bars 1 to 16").waitForExistence(
        timeout: contendedTimeout),
      "the section survived the relaunch")

    // Removed in Reorder and saved on Done: the empty slot comes back.
    app.buttons["Reorder or remove sections"].tap()
    app.buttons["Remove A1"].tap()
    app.buttons["Done reordering sections"].tap()
    XCTAssertTrue(
      app.control("sections.add", spoken: "Add sections to this piece").waitForExistence(
        timeout: 5),
      "the piece has no sections once Done saves the removal")
  }

  private func openItem(_ app: XCUIApplication, title: String) {
    let row = app.row("library.row", spokenContaining: title)
    XCTAssertTrue(row.waitForExistence(timeout: 10), "the piece's library row")
    row.tap()
  }
}
