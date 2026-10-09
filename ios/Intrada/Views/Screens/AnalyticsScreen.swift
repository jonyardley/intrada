import SharedTypes
import SwiftUI

struct AnalyticsScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.openPractice) private var openPractice

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
            VariationCoverageSection(
              caption: analytics.variationCoverageCaption, rows: variationCoverage
            )
            .fadeUp(2)
          }
          if !analytics.pooledVariations.isEmpty {
            PooledMarksSection(
              title: "Variations across the library", rows: analytics.pooledVariations
            )
            .fadeUp(2)
          }
          if !analytics.pooledKeys.isEmpty {
            PooledMarksSection(title: "Keys across the library", rows: analytics.pooledKeys)
              .fadeUp(2)
          }
          consistencySection(analytics)
            .fadeUp(3)
          if !analytics.scoreChanges.isEmpty {
            recentMasterySection(analytics)
              .fadeUp(4)
          }
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
        .padding(.bottom, IntradaSpacing.card)
      }
      .scrollEdgeShadow()
    } else {
      PlaceholderContent(
        systemImage: "chart.line.uptrend.xyaxis",
        message: "Minutes and marks will show here, week by week.",
        actions: [
          .buildSession(
            identifier: "progress.empty.build", store: store, openPractice: openPractice)
        ])
    }
  }

  // ── Hero mastery ──

  private func heroCard(_ analytics: AnalyticsView) -> some View {
    MasteryHeroCard(
      mastery: analytics.overallMastery,
      change: analytics.masteryChange,
      climbing: analytics.climbing)
  }

  // ── Consistency ──

  private func consistencySection(_ analytics: AnalyticsView) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(title: "Last five weeks", trailing: analytics.bestWeek)
      ConsistencyBars(weeks: analytics.consistencyWeeks)
    }
  }

  // ── Variations ──

  private var variationCoverage: [VariationCoverageView] {
    analytics?.variationCoverage ?? []
  }

  // ── Recent mastery ──

  private func recentMasterySection(_ analytics: AnalyticsView) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionTitle("Recent mastery")
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

  private var subtitle: String? {
    guard let analytics else { return "No sessions yet" }
    return analytics.weekLine
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
