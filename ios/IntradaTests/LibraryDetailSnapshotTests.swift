import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class LibraryDetailSnapshotTests: SnapshotTestCase {
  func testRecentSessions() {
    let block = ZStack {
      PaperBackground()
      RecentSessions(sessions: [
        RecentSession(id: "1", score: 7, dateText: "Tue · Jun 24"),
        RecentSession(id: "2", score: 6, dateText: "Sat · Jun 21"),
        RecentSession(id: "3", score: 5, dateText: "Wed · Jun 18"),
      ])
      .padding(16)
    }
    assertSnapshot(of: host(block), as: config)
  }

  func testRecentSessionsDeclining() {
    let block = ZStack {
      PaperBackground()
      RecentSessions(sessions: [
        RecentSession(id: "1", score: 5, dateText: "Tue · Jun 24"),
        RecentSession(id: "2", score: 6, dateText: "Sat · Jun 21"),
        RecentSession(id: "3", score: 8, dateText: "Wed · Jun 18"),
      ])
      .padding(16)
    }
    assertSnapshot(of: host(block), as: config)
  }

  func testLibraryDetailScreen() {
    // Preset path so the snapshot covers the pushed detail, not just the body.
    let store = Store(bridge: PreviewBridge(items: [.previewDetail]))
    let pushed = NavigationStack(path: .constant([LibraryItemView.previewDetail.id])) {
      LibraryScreen().navigationBarHiddenAtRoot()
    }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  /// The chord-chart card: parsed bar grid + "See the curriculum" (Phase A).
  func testLibraryDetailChordChartCard() {
    let store = Store(bridge: PreviewBridge(items: [.previewCharted]))
    let pushed = NavigationStack(path: .constant([LibraryItemView.previewCharted.id])) {
      LibraryScreen()
    }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  /// The photo card in both of its load-bearing states (#1355). Component-level
  /// rather than another whole-screen reference: the card is what changes, and
  /// the two states are the only thing that can independently regress.
  func testPhotoCardStates() {
    let cards = ZStack {
      PaperBackground()
      VStack(spacing: 16) {
        PhotoCard(itemId: "p1", photoId: nil)
        PhotoCard(itemId: "p1", photoId: "01ARZ3NDEKTSV4RRFFQ69G5FAV") { _ in Self.page }
      }
      .padding(16)
    }
    assertSnapshot(of: host(cards), as: config)
  }

  /// The selectable derived-curriculum commit sheet, with already-linked (not
  /// selectable) + fallback flags and per-row selection controls.
  func testScaffoldPreviewSheet() {
    assertSnapshot(
      of: host(ScaffoldPreviewSheet(preview: .preview, onCommit: { _ in })), as: config)
  }

  func testScaffoldPreviewSheetAccessibilitySize() {
    assertSnapshot(
      of: host(ScaffoldPreviewSheet(preview: .preview, onCommit: { _ in })), as: axConfig)
  }

  func testPieceDetailLinkedPopulated() {
    let store = Store(bridge: PreviewBridge(items: [.previewDetailWithLinkedExercises]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewDetailWithLinkedExercises.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  func testPieceDetailLinkedEmpty() {
    let store = Store(bridge: PreviewBridge(items: [.previewDetailLinkedEmpty]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewDetailLinkedEmpty.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  func testPieceDetailLinkedEditing() {
    let store = Store(bridge: PreviewBridge(items: [.previewDetailWithLinkedExercises]))
    // editingLinks is @State: seed via EditingLinkedExercisesWrapper with startEditingLinks=true.
    let editing = EditingLinkedExercisesWrapper(item: .previewDetailWithLinkedExercises)
    assertSnapshot(of: host(editing, store: store), as: config)
  }

  // #1363: linked to pieces it has never been practised with: every row shows
  // the ring's unrated rest, so a fresh link never reads as a bad score.
  func testExerciseDetailUsedInLinkedOnly() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseLinkedOnly]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseLinkedOnly.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  // #1087 B2 / #1363: overall-ring caption + "Used in" rows: linked and
  // practised, practised only, linked only, removed, and on its own.
  func testExerciseDetailUsedIn() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseUsedIn]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseUsedIn.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  // #1783: Variations empty state: the two key presets, and Add variations opening Edit.
  func testExerciseDetailVariationsEmptyState() {
    let store = Store(bridge: PreviewBridge(items: [.previewExercise]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExercise.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  // #1083 C2: Variations section: solid / current / unrated ring states, horizontal
  // scroller, "N of M solid" header; Key/Tempo rows hidden for laddered exercises.
  func testExerciseDetailWithVariations() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithVariations]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseWithVariations.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  /// Largest accessibility text size: proves the Variations scroller reflows
  /// rather than clipping or wrapping (#1083 C2).
  func testExerciseDetailWithVariationsAccessibilitySize() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithVariations]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseWithVariations.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: axConfig)
  }

  // #1083 C2: 12-variation ladder: survives max realistic length without wrapping.
  func testExerciseDetailWith12Variations() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithTwelveVariations]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseWithTwelveVariations.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  func testExerciseDetailWithNamedVariations() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithNamedVariations]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseWithNamedVariations.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  // #1786: the list-row layout for non-key variations, replacing the ring.
  func testExerciseDetailWithLongVariationName() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithLongVariationName]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseWithLongVariationName.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  /// Largest accessibility text size: the label wraps within its own line
  /// rather than clipping or breaking mid-word (#1786).
  func testExerciseDetailWithLongVariationNameAccessibilitySize() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithLongVariationName]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseWithLongVariationName.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: axConfig)
  }

  func testAddRelatedExerciseSheet() {
    assertSnapshot(
      of: host(
        AddRelatedExerciseSheet(groupId: "g1"), store: .previewBuildingGroupedRelatedSheet),
      as: config)
  }

  func testAddRelatedExerciseSheetAccessibilitySize() {
    assertSnapshot(
      of: host(
        AddRelatedExerciseSheet(groupId: "g1"), store: .previewBuildingGroupedRelatedSheet),
      as: axConfig)
  }

  func testAddRelatedExerciseSheetAdded() {
    assertSnapshot(
      of: host(
        AddRelatedExerciseSheet(groupId: "g1"), store: .previewBuildingGroupedAdded),
      as: config)
  }

  // #1363: the card framed on its own (the screen snapshots above cut off
  // before it), with every row state in one reference.
  func testUsedInCardRowStates() {
    assertSnapshot(
      of: usedInCard(LibraryItemView.previewExerciseUsedIn.usedIn), as: config)
  }

  // #1363: at the largest text size the Link row has to reflow, not clip: it
  // carries a ring, two lines of text, a button and a chevron across one width.
  func testUsedInCardRowStatesAccessibilityText() {
    assertSnapshot(
      of: usedInCard(LibraryItemView.previewExerciseUsedIn.usedIn), as: axConfig)
  }

  func testUsedInCardOnItsOwn() {
    assertSnapshot(of: usedInCard([]), as: config)
  }

  private func usedInCard(_ usage: [ExerciseUsageView]) -> UIViewController {
    // Hosted in a NavigationStack: outside one the rows' NavigationLinks render
    // disabled, which greys the whole row and makes the reference a lie.
    host(
      NavigationStack {
        ZStack {
          PaperBackground()
          ScrollView {
            UsedInCard(
              usage: usage, locale: Locale(identifier: "en_US"),
              calendar: PreviewCalendar.utc, onLink: { _ in }, onLinkAPiece: {}
            )
            .padding(IntradaSpacing.card)
          }
        }
      })
  }

  /// Both tempo-trend states in one frame: the plot with two breaks in the line
  /// where sessions measured nothing, and the too-little-to-plot line beneath.
  func testTempoTrend() {
    let trends = ZStack {
      PaperBackground()
      VStack(spacing: 14) {
        TempoTrend(display: .previewWithGaps)
        TempoTrend(display: .previewSingleMeasurement)
      }
      .padding(16)
    }
    assertSnapshot(of: host(trends), as: config)
  }

  /// At accessibility sizes the footer keeps the measured count and drops the
  /// end dates, rather than crushing three labels into one row.
  func testTempoTrendAccessibilityText() {
    let trend = ZStack {
      PaperBackground()
      TempoTrend(display: .previewWithGaps).padding(16)
    }
    assertSnapshot(of: host(trend), as: axConfig)
  }
}
