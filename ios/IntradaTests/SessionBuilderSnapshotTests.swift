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

  func testSessionBuilderEmptyWithLength() {
    assertSnapshot(
      of: host(NavigationStack { SessionBuilderScreen() }, store: .previewBuildingEmpty),
      as: config)
  }

  func testSessionBuilderPopulated() {
    assertSnapshot(
      of: host(NavigationStack { SessionBuilderScreen() }, store: .previewBuilding), as: config)
  }

  func testSessionBuilderAccessibilitySize() {
    assertSnapshot(
      of: host(NavigationStack { SessionBuilderScreen() }, store: .previewBuilding), as: axConfig)
  }

  func testSessionBuilderGrouped() {
    assertSnapshot(
      of: host(NavigationStack { SessionBuilderScreen() }, store: .previewBuildingGrouped),
      as: config)
  }

  func testSessionBuilderGroupedAccessibilitySize() {
    assertSnapshot(
      of: host(NavigationStack { SessionBuilderScreen() }, store: .previewBuildingGrouped),
      as: tallAxConfig(height: 1800))
  }

  func testSessionBuilderGroupedCollapsedAccessibilitySize() {
    assertSnapshot(
      of: host(
        NavigationStack { SessionBuilderScreen(startCollapsedGroups: ["g1"]) },
        store: .previewBuildingGrouped), as: tallAxConfig(height: 1800))
  }

  func testAddToSessionSheet() {
    assertSnapshot(of: host(AddToSessionSheet(), store: .previewBuilding), as: config)
  }

  func testAddToSessionSheetAccessibilitySize() {
    assertSnapshot(of: host(AddToSessionSheet(), store: .previewBuilding), as: axConfig)
  }

  func testAddToSessionSheetLargestStandardSizeLongKeyAndTempo() {
    var item = LibraryItemView.previewPiece
    item.key = Key(letter: .c, accidental: .sharp, mode: .minor)
    item.keyLabel = "C\u{266F} minor"
    item.tempoMarking = "Allegro ma non troppo e molto espressivo"
    item.tempoBpm = 132
    let store = Store(
      bridge: PreviewBridge(
        items: [item, .previewExercise],
        buildingSetlist: BuildingSetlistView(
          entries: [.previewExercise], itemCount: 1, blocks: [],
          totalDurationDisplay: nil, totalDurationSummary: nil, lengthMins: nil,
          lengthSummary: nil, entryVariations: [], lastTimes: [], focusChoices: [], entryKeys: [],
          drillOffers: [])))
    assertSnapshot(
      of: host(AddToSessionSheet(), store: store),
      as: .image(
        on: .iPhone13, perceptualPrecision: 0.98,
        traits: UITraitCollection { traits in
          traits.displayScale = 2
          traits.preferredContentSizeCategory = .extraExtraExtraLarge
        }))
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

  func testEntrySettingsSheetAccessibilitySize() throws {
    let store = Store.previewBuildingGrouped
    let limits = try XCTUnwrap(store.viewModel?.limits)
    var entry = SetlistEntryView.previewGroupedScalesConfigured
    entry.itemTitle = "Piano Concerto No. 2 in C minor"
    assertSnapshot(
      of: host(EntrySettingsSheet(entry: entry, limits: limits), store: store), as: axConfig)
  }

  func testSessionBuilderPlanned() {
    assertSnapshot(
      of: host(NavigationStack { SessionBuilderScreen() }, store: .previewBuildingPlanned()),
      as: config)
  }

  func testEntrySettingsSheetSplitWithFocus() throws {
    let store = Store.previewBuildingPlanned()
    let limits = try XCTUnwrap(store.viewModel?.limits)
    assertSnapshot(
      of: host(EntrySettingsSheet(entry: .previewPlannedPiece, limits: limits), store: store),
      as: config)
  }

  func testEntrySettingsSheetSplitWithFocusAccessibilitySize() throws {
    let store = Store.previewBuildingPlanned()
    let limits = try XCTUnwrap(store.viewModel?.limits)
    assertSnapshot(
      of: host(EntrySettingsSheet(entry: .previewPlannedPiece, limits: limits), store: store),
      as: axConfig)
  }

  func testEntrySettingsSheetSuggestsAFocus() throws {
    let store = Store.previewBuildingPlanned(.previewSuggestingPiece)
    let limits = try XCTUnwrap(store.viewModel?.limits)
    assertSnapshot(
      of: host(EntrySettingsSheet(entry: .previewSuggestingPiece, limits: limits), store: store),
      as: config)
  }

  private static let clairDrills = [
    DrillOfferView(
      entryId: "plan-p", sectionId: "sec-a2", exerciseId: "ex-arp", title: "Left hand arpeggios",
      addedEntryId: "drill-arp"),
    DrillOfferView(
      entryId: "plan-p", sectionId: "sec-a2", exerciseId: "ex-pedal", title: "Pedal changes",
      addedEntryId: nil),
  ]

  private static let clairKeys = EntryKeysView(
    entryId: "plan-p",
    keys: [
      KeyChoiceView(key: nil, label: "Written key", caption: "D\u{266D} major"),
      KeyChoiceView(
        key: Key(letter: .d, accidental: .natural, mode: .major), label: "D major", caption: nil),
      KeyChoiceView(
        key: Key(letter: .c, accidental: .natural, mode: .major), label: "C major", caption: nil),
    ], current: Key(letter: .d, accidental: .natural, mode: .major), currentLabel: "D major")

  private static var plannedInD: SetlistEntryView {
    var entry = SetlistEntryView.previewPlannedPiece
    entry.plannedKey = Key(letter: .d, accidental: .natural, mode: .major)
    return entry
  }

  func testEntrySettingsSheetOffersDrillsOneTicked() throws {
    let store = Store.previewBuildingPlanned(drillOffers: Self.clairDrills)
    let limits = try XCTUnwrap(store.viewModel?.limits)
    assertSnapshot(
      of: host(EntrySettingsSheet(entry: .previewPlannedPiece, limits: limits), store: store),
      as: tallFormConfig)
  }

  func testEntrySettingsSheetKeyRow() throws {
    let store = Store.previewBuildingPlanned(Self.plannedInD, entryKeys: [Self.clairKeys])
    let limits = try XCTUnwrap(store.viewModel?.limits)
    assertSnapshot(
      of: host(EntrySettingsSheet(entry: Self.plannedInD, limits: limits), store: store),
      as: tallFormConfig)
  }

  func testEntryKeyList() {
    let store = Store.previewBuildingPlanned(Self.plannedInD, entryKeys: [Self.clairKeys])
    assertSnapshot(
      of: host(NavigationStack { EntryKeyList(entryId: "plan-p") }, store: store), as: config)
  }

  func testEntryKeyListAccessibilitySize() {
    let store = Store.previewBuildingPlanned(Self.plannedInD, entryKeys: [Self.clairKeys])
    assertSnapshot(
      of: host(NavigationStack { EntryKeyList(entryId: "plan-p") }, store: store), as: axConfig)
  }
}
