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

  func testAnalyticsScreen() {
    assertSnapshot(of: host(AnalyticsScreen()), as: config)
  }

  func testProgressScreenPopulated() {
    assertSnapshot(of: host(AnalyticsScreen(), store: .previewProgress), as: config)
  }

  /// Monday morning: nothing played this week and nothing marked, so no
  /// "0 sessions" line, no empty Recent mastery heading (#2374) and no
  /// climbing line (#2399).
  func testProgressScreenNothingThisWeek() {
    var analytics = AnalyticsView.previewAnalytics
    analytics.weeklySummary.totalMinutes = 0
    analytics.weeklySummary.sessionCount = 0
    analytics.weeklySummary.itemsCovered = 0
    analytics.scoreChanges = []
    analytics.topMover = nil
    analytics.masteryChange = nil
    analytics.climbing = nil
    analytics.weekLine = nil
    analytics.consistencyWeeks[4].minutes = 0
    analytics.consistencyWeeks[4].spoken = "This week: 0 minutes"
    let store = Store(bridge: PreviewBridge(analytics: analytics))
    assertSnapshot(of: host(AnalyticsScreen(), store: store), as: config)
  }

  func testMasteryDial() {
    let dial = ZStack {
      PaperBackground()
      MasteryDial(value: 3.4)
    }
    assertSnapshot(of: host(dial), as: config)
  }

  func testMasteryDialTakesTheTopMarkFromTheCore() {
    let dial = ZStack {
      PaperBackground()
      MasteryDial(value: 3.4)
    }
    .environment(\.scoreRange, 1...5)
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
      ConsistencyBars(weeks: AnalyticsView.previewAnalytics.consistencyWeeks)
        .padding(16)
    }
    assertSnapshot(of: host(bars), as: config)
  }
}
