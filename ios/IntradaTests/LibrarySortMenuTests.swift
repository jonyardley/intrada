import SharedTypes
import XCTest

@testable import Intrada

final class LibrarySortMenuTests: XCTestCase {

  func testTappingActiveFieldFlipsDirection() {
    let descending = LibrarySort(field: .dateAdded, direction: .descending)
    let flipped = LibrarySort.selecting(.dateAdded, from: descending)
    XCTAssertEqual(flipped, LibrarySort(field: .dateAdded, direction: .ascending))

    let backAgain = LibrarySort.selecting(.dateAdded, from: flipped)
    XCTAssertEqual(backAgain, descending, "flipping twice returns to the original")
  }

  func testSwitchingFieldUsesNaturalDefault() {
    let current = LibrarySort(field: .dateAdded, direction: .ascending)

    XCTAssertEqual(
      LibrarySort.selecting(.lastPracticed, from: current),
      LibrarySort(field: .lastPracticed, direction: .descending),
      "Last Practiced defaults to most-recent-first regardless of the prior direction")

    XCTAssertEqual(
      LibrarySort.selecting(.title, from: current),
      LibrarySort(field: .title, direction: .ascending),
      "Title defaults to A→Z")

    XCTAssertEqual(
      LibrarySort.selecting(.dateAdded, from: LibrarySort(field: .title, direction: .ascending)),
      LibrarySort(field: .dateAdded, direction: .descending),
      "Date Added defaults to newest-first")
  }

  func testNaturalDefaultsMatchSpec() {
    XCTAssertEqual(LibrarySortField.dateAdded.naturalDefault, .descending)
    XCTAssertEqual(LibrarySortField.lastPracticed.naturalDefault, .descending)
    XCTAssertEqual(LibrarySortField.title.naturalDefault, .ascending)
  }
}
