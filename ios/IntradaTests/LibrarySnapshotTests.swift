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

  func testLibrarySplitViewEmptyLibrary() {
    assertSnapshot(of: host(LibrarySplitView()), as: splitConfig)
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

  /// A search that matches nothing keeps sort, filter and search on screen, so
  /// it can be cleared; only an empty library hides them.
  func testLibraryScreenSearchMatchesNothing() {
    let store = Store(
      bridge: PreviewBridge(
        items: [.previewPiece, .previewExercise, .previewMinimal],
        activeQuery: ListQuery(
          text: "zzz", itemType: nil, key: nil, tags: [], priorityOnly: false),
        visibleIds: []))
    assertSnapshot(
      of: host(
        NavigationStack { LibraryScreen(previewSearch: "zzz").navigationBarHiddenAtRoot() },
        store: store), as: config)
  }

  /// A search left over from deleting the last match keeps its row, or it
  /// filters the next item added with nothing on screen to clear it.
  func testLibraryScreenEmptyWithSearch() {
    let store = Store(
      bridge: PreviewBridge(
        items: [],
        activeQuery: ListQuery(
          text: "zzz", itemType: nil, key: nil, tags: [], priorityOnly: false),
        visibleIds: []))
    assertSnapshot(
      of: host(
        NavigationStack { LibraryScreen(previewSearch: "zzz").navigationBarHiddenAtRoot() },
        store: store), as: config)
  }

  /// The search field just after reveal, before any text lands (#1825, moved from `LibrarySearchUITests`).
  func testLibraryScreenSearchRevealedEmpty() {
    assertSnapshot(
      of: host(
        NavigationStack { LibraryScreen(previewSearch: "").navigationBarHiddenAtRoot() },
        store: .previewLibrary), as: config)
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
