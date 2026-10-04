import XCTest

/// Choosing every major key in the keys sheet lays the twelve rings out on the
/// item screen (#2247), against the seeded "Clair de Lune".
@MainActor
final class ItemKeysUITests: XCTestCase {
  override func setUp() {
    super.setUp()
    continueAfterFailure = false
  }

  func testAllMajorKeysShowAsRings() {
    let app = XCUIApplication()
    app.launchArguments = ["--seed-sample-data", "--disable-animations"]
    app.launch()
    app.tabBars.buttons["Library"].tap()
    let piece = app.row("library.row", spokenContaining: "Clair de Lune")
    XCTAssertTrue(piece.waitForExistence(timeout: 10), "Clair de Lune library row")
    piece.tap()

    app.control("keys.choose", spoken: "Practise in other keys").tap()
    app.control("keysSheet.allMajor", spoken: "All major").tap()
    app.control("sheet.done", spoken: "Done").tap()

    for key in ["C major", "G major", "F major"] {
      XCTAssertTrue(
        app.descendants(matching: .any).matching(
          NSPredicate(format: "label BEGINSWITH %@", "\(key),")
        ).firstMatch.waitForExistence(timeout: 5), "\(key) shows as a ring")
    }
  }
}
