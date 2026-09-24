import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class ProgressSnapshotTests: SnapshotTestCase {
  func testRoutinesScreen() {
    assertSnapshot(of: host(RoutinesScreen()), as: config)
  }

  func testRoutinesScreenAccessibilitySize() {
    assertSnapshot(of: host(RoutinesScreen()), as: axConfig)
  }

  func testAnalyticsScreen() {
    assertSnapshot(of: host(AnalyticsScreen()), as: config)
  }

  func testProgressScreenPopulated() {
    assertSnapshot(of: host(AnalyticsScreen(), store: .previewProgress), as: config)
  }

  /// The top of the Progress screen at the largest text size: the mover toast
  /// and the hero card stack rather than squash. The coverage rows sit below
  /// this frame; they get their own snapshot.
  func testProgressScreenPopulatedAccessibilitySize() {
    assertSnapshot(of: host(AnalyticsScreen(), store: .previewProgress), as: axConfig)
  }

  /// The variation coverage rows at the largest text size: the exercise title
  /// and its solid count share a row, so they have to reflow rather than
  /// squash (#1739, #1762).
  func testProgressVariationRowsAccessibilitySize() {
    let rows = ZStack {
      PaperBackground()
      VStack {
        VariationCoverageSection(rows: AnalyticsView.previewAnalytics.variationCoverage)
        Spacer()
      }
      .padding(IntradaSpacing.card)
    }
    .dynamicTypeSize(.accessibility5)
    assertSnapshot(of: host(rows), as: config)
  }

  func testMasteryDial() {
    let dial = ZStack {
      PaperBackground()
      MasteryDial(value: 3.4)
    }
    assertSnapshot(of: host(dial), as: config)
  }

  func testMasteryDeltaRows() {
    let rows = ZStack {
      PaperBackground()
      VStack(spacing: 12) {
        MasteryDelta(
          title: "Clair de Lune", subtitle: "D♭ major · now", was: 3, now: 4, kind: .piece)
        MasteryDelta(
          title: "Hanon No. 1", subtitle: "first time marked", was: nil, now: 3, kind: .exercise)
        MasteryDeltaToast(
          title: "Clair de Lune moved up", subtitle: "D♭ major mastery", was: 3, now: 4)
      }
      .padding(16)
    }
    assertSnapshot(of: host(rows), as: config)
  }

  func testConsistencyBars() {
    let bars = ZStack {
      PaperBackground()
      ConsistencyBars(weeks: [
        ConsistencyWeek(label: "W1", minutes: 40),
        ConsistencyWeek(label: "W2", minutes: 75),
        ConsistencyWeek(label: "W3", minutes: 55),
        ConsistencyWeek(label: "W4", minutes: 95),
        ConsistencyWeek(label: "Now", minutes: 82, isCurrent: true),
      ])
      .padding(16)
    }
    assertSnapshot(of: host(bars), as: config)
  }

  /// Recorded at the size #1471 reports, which is also where the ring's clamp
  /// binds hardest: stacked layout and dial inset are both load-bearing here.
  func testMasteryHeroCardAccessibilitySize() {
    let hero = ZStack {
      PaperBackground()
      MasteryHeroCard(mastery: 6.4, change: "+1.2 this week", itemsCovered: 5)
        .padding(16)
    }
    .dynamicTypeSize(.accessibility5)
    assertSnapshot(of: host(hero), as: config)
  }
}
