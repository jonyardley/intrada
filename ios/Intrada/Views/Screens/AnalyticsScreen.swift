import SharedTypes
import SwiftUI

struct AnalyticsScreen: View {
  @Environment(Store.self) private var store

  private var analytics: AnalyticsView? { store.viewModel?.analytics }

  var body: some View {
    ScreenScaffold(title: "Progress", subtitle: subtitle) {
      content
    }
  }

  @ViewBuilder private var content: some View {
    if let analytics {
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.section) {
          if let mover = analytics.topMover {
            MasteryDeltaToast(
              title: "Mastery up", subtitle: mover.itemTitle,
              was: Int(mover.previousScore ?? 0), now: Int(mover.currentScore)
            )
            .fadeUp(0)
          }
          heroCard(analytics)
            .fadeUp(1)
          if !variationCoverage.isEmpty {
            VariationCoverageSection(rows: variationCoverage)
              .fadeUp(2)
          }
          consistencySection(analytics)
            .fadeUp(3)
          recentMasterySection(analytics)
            .fadeUp(4)
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
        .padding(.bottom, IntradaSpacing.card)
      }
      .scrollEdgeShadow()
    } else {
      PlaceholderContent(
        systemImage: "chart.line.uptrend.xyaxis",
        message: "Progress will appear here once you start practising.")
    }
  }

  // ── Hero mastery ──

  private func heroCard(_ analytics: AnalyticsView) -> some View {
    MasteryHeroCard(
      mastery: analytics.overallMastery,
      change: analytics.masteryChange,
      itemsCovered: Int(analytics.weeklySummary.itemsCovered))
  }

  // ── Consistency ──

  private func consistencySection(_ analytics: AnalyticsView) -> some View {
    let weeks = weeklyBuckets(analytics)
    let maxMinutes = weeks.map(\.minutes).max() ?? 0
    return VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(title: "Last five weeks", trailing: "best week · \(maxMinutes)m")
      ConsistencyBars(weeks: weeks)
    }
  }

  // ── Variations ──

  private var variationCoverage: [VariationCoverageView] {
    analytics?.variationCoverage ?? []
  }

  // ── Recent mastery ──

  private func recentMasterySection(_ analytics: AnalyticsView) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Eyebrow("Recent mastery")
      VStack(spacing: IntradaSpacing.cardCompact) {
        ForEach(Array(analytics.scoreChanges.enumerated()), id: \.offset) { idx, change in
          MasteryDelta(
            title: change.itemTitle,
            subtitle: change.isNew ? "first time marked" : nil,
            was: change.previousScore.map(Int.init),
            now: Int(change.currentScore)
          )
          .fadeUp(5 + idx)
        }
      }
    }
  }

  // ── Derivations ──

  private var subtitle: String {
    guard let summary = analytics?.weeklySummary else { return "No sessions yet" }
    let h = summary.totalMinutes / 60
    let m = summary.totalMinutes % 60
    let duration = h == 0 ? "\(m)m" : "\(h)h \(m)m"
    return "\(summary.sessionCount) sessions · \(duration) this week"
  }

  private func weeklyBuckets(_ analytics: AnalyticsView) -> [ConsistencyWeek] {
    let lastIndex = analytics.weeklyMinutes.count - 1
    return analytics.weeklyMinutes.enumerated().map { idx, minutes in
      let isCurrent = idx == lastIndex
      return ConsistencyWeek(
        label: isCurrent ? "Now" : "W\(idx + 1)",
        minutes: Int(minutes),
        isCurrent: isCurrent)
    }
  }
}

#if DEBUG
  #Preview {
    AnalyticsScreen()
      .environment(Store.previewProgress)
  }

  #Preview("Empty") {
    AnalyticsScreen()
      .environment(Store.preview)
  }
#endif
