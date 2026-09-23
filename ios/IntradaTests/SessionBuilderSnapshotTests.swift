import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class SessionBuilderSnapshotTests: SnapshotTestCase {
  func testSessionBuilderEmpty() {
    assertSnapshot(of: host(NavigationStack { SessionBuilderScreen() }), as: config)
  }

  func testSessionBuilderPopulated() {
    assertSnapshot(
      of: host(NavigationStack { SessionBuilderScreen() }, store: .previewBuilding), as: config)
  }

  func testSessionBuilderGrouped() {
    assertSnapshot(
      of: host(NavigationStack { SessionBuilderScreen() }, store: .previewBuildingGrouped),
      as: config)
  }

  func testAddToSessionSheet() {
    assertSnapshot(of: host(AddToSessionSheet(), store: .previewBuilding), as: config)
  }

  func testAddToSessionSheetRecentlyPractised() {
    assertSnapshot(
      of: host(AddToSessionSheet(), store: .previewBuildingRecentlyPractised), as: config)
  }

  func testAddToSessionSheetRecentlyPractisedHiddenWhileFiltered() {
    assertSnapshot(
      of: host(AddToSessionSheet(), store: .previewBuildingRecentlyPractisedFiltered), as: config)
  }

  func testSessionBuilderGroupedEditing() {
    // editMode is @State: seed via the startInEditMode init to capture the
    // nested-row reorder/remove/settings controls without UI interaction.
    assertSnapshot(
      of: host(
        NavigationStack { SessionBuilderScreen(startInEditMode: true) },
        store: .previewBuildingGrouped), as: config)
  }

  func testEntrySettingsSheetEmpty() throws {
    let store = Store.previewBuildingGrouped
    let limits = try XCTUnwrap(store.viewModel?.limits)
    assertSnapshot(
      of: host(EntrySettingsSheet(entry: .previewGroupedScales, limits: limits), store: store),
      as: config)
  }

  func testEntrySettingsSheetPopulated() throws {
    let store = Store.previewBuildingGrouped
    let limits = try XCTUnwrap(store.viewModel?.limits)
    assertSnapshot(
      of: host(
        EntrySettingsSheet(entry: .previewGroupedScalesConfigured, limits: limits), store: store
      ), as: config)
  }
}
