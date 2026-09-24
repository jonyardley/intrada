import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class LibrarySnapshotTests: SnapshotTestCase {
  func testLibraryScreen() {
    assertSnapshot(
      of: host(NavigationStack { LibraryScreen().navigationBarHiddenAtRoot() }), as: config)
  }

  func testLibrarySplitViewEmptyDetail() {
    assertSnapshot(
      of: host(LibrarySplitView(), store: .previewLibrary), as: splitConfig)
  }

  /// The column line must stop below the top strip, not run past the tabs
  /// (#1682); see `LibrarySplitAlignmentTests` for header alignment.
  func testLibrarySplitViewWithSelection() {
    assertSnapshot(
      of: host(LibrarySplitView(previewSelection: "piece-1"), store: .previewLibrary),
      as: splitConfig)
  }

  func testLibraryScreenPopulated() {
    assertSnapshot(
      of: host(
        NavigationStack { LibraryScreen().navigationBarHiddenAtRoot() }, store: .previewLibrary),
      as: config)
  }

  func testLibraryScreenPriorities() {
    assertSnapshot(
      of: host(
        NavigationStack { LibraryScreen().navigationBarHiddenAtRoot() },
        store: .previewLibraryPriorities), as: config)
  }

  func testLibraryScreenFiltered() {
    assertSnapshot(
      of: host(
        NavigationStack { LibraryScreen().navigationBarHiddenAtRoot() },
        store: .previewLibraryFiltered), as: config)
  }

  func testLibraryScreenSearching() {
    assertSnapshot(
      of: host(
        NavigationStack { LibraryScreen(previewSearch: "clair").navigationBarHiddenAtRoot() },
        store: .previewLibrarySearching), as: config)
  }

  /// The search field just after reveal, before any text lands (#1825, moved from `LibrarySearchUITests`).
  func testLibraryScreenSearchRevealedEmpty() {
    assertSnapshot(
      of: host(
        NavigationStack { LibraryScreen(previewSearch: "").navigationBarHiddenAtRoot() },
        store: .previewLibrary), as: config)
  }

  /// The browse controls have to give way at accessibility sizes, or the screen
  /// lays out wider than the device and shifts off its leading edge (#1470).
  func testLibraryScreenAccessibilityText() {
    assertSnapshot(
      of: host(
        NavigationStack { LibraryScreen().navigationBarHiddenAtRoot() }, store: .previewLibrary),
      as: axConfig)
  }

  func testLibraryScreenMastery() {
    assertSnapshot(
      of: host(
        NavigationStack { LibraryScreen().navigationBarHiddenAtRoot() },
        store: .previewLibraryMastery), as: config)
  }

  func testTypeBadges() {
    let badges = ZStack {
      PaperBackground()
      HStack(spacing: 12) {
        TypeBadge(kind: .piece)
        TypeBadge(kind: .exercise)
      }
    }
    assertSnapshot(of: host(badges), as: config)
  }

  func testTagFilterSheet() {
    let sheet = TagFilterSheet(
      available: ["classical", "jazz", "recital", "technique", "warm-up"],
      selected: ["jazz", "recital"],
      onChange: { _ in })
    assertSnapshot(of: host(sheet), as: config)
  }

  func testTagFilterSheetAccessibilitySize() {
    let sheet = TagFilterSheet(
      available: ["classical", "jazz", "recital", "technique", "warm-up"],
      selected: ["jazz", "recital"],
      onChange: { _ in })
    assertSnapshot(of: host(sheet), as: axConfig)
  }

  func testTagFilterSheetEmpty() {
    let sheet = TagFilterSheet(available: [], selected: [], onChange: { _ in })
    assertSnapshot(of: host(sheet), as: config)
  }

  func testLibraryItemCards() {
    var manyTags = LibraryItemView.previewDetail
    manyTags.tags = ["jazz", "improv", "bebop", "ii-V-I", "comping"]
    // Starred: pins the accent star to the left of the tags + the trailing meter.
    var starred = LibraryItemView.previewDetail
    starred.priority = true
    let cards = ZStack {
      PaperBackground()
      VStack(spacing: 14) {
        LibraryItemCard(item: .previewPiece)
        LibraryItemCard(item: .previewDetail)
        LibraryItemCard(item: manyTags)  // 5 tags → +2 overflow pill
        LibraryItemCard(item: starred, showsMastery: true)
        LibraryItemCard(item: .previewExerciseWithTwelveVariations)
        LibraryItemCard(item: .previewExerciseWithNamedVariations)
        LibraryItemCard(item: .previewMinimal, showsMastery: true, showsMissingDetailsPrompt: true)
      }
      .padding(16)
    }
    assertSnapshot(of: host(cards), as: tallFormConfig)
  }
}
