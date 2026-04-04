import SwiftUI
import Charts

/// Analytics tab root — practice insights dashboard.
///
/// Read-only display of data from `core.viewModel.analytics`.
/// No events dispatched — purely renders computed analytics.
struct AnalyticsDashboardView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.horizontalSizeClass) private var sizeClass

    private var analytics: AnalyticsView? {
        core.viewModel.analytics
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                if let analytics {
                    if sizeClass == .regular {
                        iPadDashboard(analytics)
                    } else {
                        iPhoneDashboard(analytics)
                    }
                } else {
                    EmptyStateView(
                        icon: "chart.xyaxis.line",
                        title: "No analytics yet",
                        message: "Complete a practice session to see your insights here"
                    )
                }
            }
            .background(Color.backgroundApp)
            .navigationTitle("Analytics")
        }
    }

    // MARK: - Streak

    // MARK: - iPhone Layout

    @ViewBuilder
    private func iPhoneDashboard(_ analytics: AnalyticsView) -> some View {
        VStack(spacing: Spacing.card) {
            consistencyCard(analytics.streak)
            weeklySummaryCard(analytics.weeklySummary)
            weeklyInsightsCard(
                neglected: analytics.neglectedItems,
                scoreChanges: analytics.scoreChanges
            )
            practiceChartCard(analytics.dailyTotals)
            topItemsCard(analytics.topItems)
            scoreTrendsCard(analytics.scoreTrends)
        }
        .padding(.horizontal, Spacing.card)
        .padding(.bottom, Spacing.cardComfortable)
    }

    // MARK: - iPad Layout

    @ViewBuilder
    private func iPadDashboard(_ analytics: AnalyticsView) -> some View {
        VStack(spacing: Spacing.card) {
            // Top row: consistency + weekly summary side by side
            HStack(spacing: Spacing.card) {
                consistencyCard(analytics.streak)
                weeklySummaryCard(analytics.weeklySummary)
            }

            // Insights + chart side by side
            HStack(alignment: .top, spacing: Spacing.card) {
                weeklyInsightsCard(
                    neglected: analytics.neglectedItems,
                    scoreChanges: analytics.scoreChanges
                )
                practiceChartCard(analytics.dailyTotals)
            }

            // Bottom row: top items + score trends side by side
            HStack(alignment: .top, spacing: Spacing.card) {
                topItemsCard(analytics.topItems)
                scoreTrendsCard(analytics.scoreTrends)
            }
        }
        .padding(.horizontal, Spacing.cardComfortable)
        .padding(.bottom, Spacing.cardComfortable)
    }

    // MARK: - Practice Consistency (comeback framing, not streak)

    @ViewBuilder
    private func consistencyCard(_ streak: PracticeStreak) -> some View {
        CardView {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    if streak.currentDays > 0 {
                        Text("\(streak.currentDays) days")
                            .font(.system(size: 36, weight: .bold))
                            .foregroundStyle(Color.warmAccentText)
                        Text("of practice this week")
                            .font(.system(size: 13, weight: .medium))
                            .foregroundStyle(Color.textMuted)
                    } else {
                        Text("Welcome back")
                            .font(.system(size: 24, weight: .bold))
                            .foregroundStyle(Color.warmAccentText)
                        Text("Ready to pick up where you left off?")
                            .font(.system(size: 13, weight: .medium))
                            .foregroundStyle(Color.textMuted)
                    }
                }
                Spacer()
                Image(systemName: streak.currentDays > 0 ? "music.note.list" : "music.note")
                    .font(.system(size: 28))
                    .foregroundStyle(Color.warmAccent)
            }
        }
    }

    // MARK: - Weekly Summary

    @ViewBuilder
    private func weeklySummaryCard(_ summary: WeeklySummary) -> some View {
        CardView {
            VStack(alignment: .leading, spacing: Spacing.cardCompact) {
                Text("This Week")
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundStyle(Color.textPrimary)

                HStack(spacing: Spacing.cardCompact) {
                    weekStat(
                        value: formatMinutes(summary.totalMinutes),
                        label: "Practice",
                        direction: summary.timeDirection,
                        prev: summary.hasPrevWeekData ? formatMinutes(summary.prevTotalMinutes) : nil
                    )
                    weekStat(
                        value: "\(summary.sessionCount)",
                        label: "Sessions",
                        direction: summary.sessionsDirection,
                        prev: summary.hasPrevWeekData ? "\(summary.prevSessionCount)" : nil
                    )
                    weekStat(
                        value: "\(summary.itemsCovered)",
                        label: "Items",
                        direction: summary.itemsDirection,
                        prev: summary.hasPrevWeekData ? "\(summary.prevItemsCovered)" : nil
                    )
                }
            }
        }
    }

    @ViewBuilder
    private func weekStat(value: String, label: String, direction: Direction, prev: String?) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 4) {
                Text(value)
                    .font(.system(size: 18, weight: .bold))
                    .foregroundStyle(Color.textPrimary)
                directionIcon(direction)
            }
            Text(label)
                .font(.system(size: 11, weight: .medium))
                .foregroundStyle(Color.textMuted)
            if let prev {
                Text("from \(prev)")
                    .font(.system(size: 10))
                    .foregroundStyle(Color.textFaint)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    @ViewBuilder
    private func directionIcon(_ direction: Direction) -> some View {
        switch direction {
        case .up:
            Image(systemName: "arrow.up")
                .font(.system(size: 10, weight: .bold))
                .foregroundStyle(Color.successText)
        case .down:
            Image(systemName: "arrow.down")
                .font(.system(size: 10, weight: .bold))
                .foregroundStyle(Color.textMuted)
        case .same:
            Image(systemName: "minus")
                .font(.system(size: 10, weight: .bold))
                .foregroundStyle(Color.textFaint)
        }
    }

    // MARK: - Weekly Insights (Neglected + Score Changes)

    @ViewBuilder
    private func weeklyInsightsCard(neglected: [NeglectedItem], scoreChanges: [ScoreChange]) -> some View {
        if !neglected.isEmpty || !scoreChanges.isEmpty {
            CardView {
                VStack(alignment: .leading, spacing: Spacing.card) {
                    if !neglected.isEmpty {
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Needs Attention")
                                .font(.system(size: 13, weight: .semibold))
                                .foregroundStyle(Color.warmAccentText)

                            ForEach(neglected.prefix(5), id: \.itemId) { (item: NeglectedItem) in
                                HStack {
                                    Text(item.itemTitle)
                                        .font(.system(size: 13))
                                        .foregroundStyle(Color.textSecondary)
                                        .lineLimit(1)
                                    Spacer()
                                    if let days = item.daysSincePractice {
                                        Text("\(days) days ago")
                                            .font(.system(size: 11))
                                            .foregroundStyle(Color.textFaint)
                                    } else {
                                        Text("never practised")
                                            .font(.system(size: 11))
                                            .foregroundStyle(Color.textFaint)
                                    }
                                }
                            }
                        }
                    }

                    if !neglected.isEmpty && !scoreChanges.isEmpty {
                        Divider().background(Color.borderDefault)
                    }

                    if !scoreChanges.isEmpty {
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Improvements")
                                .font(.system(size: 13, weight: .semibold))
                                .foregroundStyle(Color.successText)

                            ForEach(scoreChanges, id: \.itemId) { (change: ScoreChange) in
                                HStack {
                                    Text(change.itemTitle)
                                        .font(.system(size: 13))
                                        .foregroundStyle(Color.textSecondary)
                                        .lineLimit(1)
                                    Spacer()
                                    if change.isNew {
                                        Text("new")
                                            .font(.system(size: 11, weight: .medium))
                                            .foregroundStyle(Color.accentText)
                                    } else if let prev = change.previousScore {
                                        Text("\(prev) → \(change.currentScore)")
                                            .font(.system(size: 11, weight: .medium, design: .monospaced))
                                            .foregroundStyle(change.delta > 0 ? Color.successText : Color.textMuted)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // MARK: - Practice Chart

    @ViewBuilder
    private func practiceChartCard(_ dailyTotals: [DailyPracticeTotal]) -> some View {
        CardView {
            VStack(alignment: .leading, spacing: Spacing.cardCompact) {
                Text("Practice History (28 days)")
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(Color.textSecondary)

                if dailyTotals.isEmpty || dailyTotals.allSatisfy({ $0.minutes == 0 }) {
                    Text("No practice data yet. Start a session to see your history here.")
                        .font(.system(size: 12))
                        .foregroundStyle(Color.textFaint)
                        .padding(.vertical, Spacing.cardComfortable)
                } else {
                    Chart(dailyTotals, id: \.date) { (total: DailyPracticeTotal) in
                        BarMark(
                            x: .value("Date", formatChartDate(total.date)),
                            y: .value("Minutes", total.minutes)
                        )
                        .foregroundStyle(Color.accent.opacity(0.6))
                        .cornerRadius(3)
                    }
                    .chartXAxis {
                        AxisMarks(values: .automatic(desiredCount: 7)) { _ in
                            AxisValueLabel()
                                .font(.system(size: 8))
                                .foregroundStyle(Color.textFaint)
                        }
                    }
                    .chartYAxis {
                        AxisMarks { _ in
                            AxisGridLine()
                                .foregroundStyle(Color.borderDefault)
                            AxisValueLabel()
                                .font(.system(size: 9))
                                .foregroundStyle(Color.textFaint)
                        }
                    }
                    .frame(height: 160)
                }
            }
        }
    }

    // MARK: - Top Items

    @ViewBuilder
    private func topItemsCard(_ items: [ItemRanking]) -> some View {
        CardView {
            VStack(alignment: .leading, spacing: Spacing.cardCompact) {
                Text("Most Practised")
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(Color.textSecondary)

                if items.isEmpty {
                    Text("No session data yet.")
                        .font(.system(size: 12))
                        .foregroundStyle(Color.textFaint)
                        .padding(.vertical, Spacing.cardCompact)
                } else {
                    ForEach(Array(items.prefix(10).enumerated()), id: \.element.itemId) { (index: Int, item: ItemRanking) in
                        HStack(spacing: Spacing.cardCompact) {
                            Text("\(index + 1).")
                                .font(.system(size: 13, weight: .medium, design: .monospaced))
                                .foregroundStyle(Color.textFaint)
                                .frame(width: 24)

                            Text(item.itemTitle)
                                .font(.system(size: 14, weight: .medium))
                                .foregroundStyle(Color.textPrimary)
                                .lineLimit(1)

                            Spacer()

                            VStack(alignment: .trailing, spacing: 2) {
                                Text(formatMinutes(item.totalMinutes))
                                    .font(.system(size: 12, weight: .medium))
                                    .foregroundStyle(Color.textSecondary)
                                Text("\(item.sessionCount) sessions")
                                    .font(.system(size: 10))
                                    .foregroundStyle(Color.textFaint)
                            }
                        }
                        .padding(.vertical, 2)
                    }
                }
            }
        }
    }

    // MARK: - Score Trends

    @ViewBuilder
    private func scoreTrendsCard(_ trends: [ItemScoreTrend]) -> some View {
        CardView {
            VStack(alignment: .leading, spacing: Spacing.cardCompact) {
                Text("Score Trends")
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(Color.textSecondary)

                if trends.isEmpty {
                    Text("Rate your confidence (1–5) during practice to see trends here.")
                        .font(.system(size: 12))
                        .foregroundStyle(Color.textFaint)
                        .padding(.vertical, Spacing.cardCompact)
                } else {
                    ForEach(trends, id: \.itemId) { (trend: ItemScoreTrend) in
                        VStack(alignment: .leading, spacing: 6) {
                            HStack {
                                Text(trend.itemTitle)
                                    .font(.system(size: 14, weight: .medium))
                                    .foregroundStyle(Color.textPrimary)
                                    .lineLimit(1)
                                Spacer()
                                Text("\(trend.latestScore)/5")
                                    .font(.system(size: 12, weight: .semibold))
                                    .foregroundStyle(scoreColor(trend.latestScore))
                            }

                            // Score dots
                            HStack(spacing: 4) {
                                ForEach(trend.scores.suffix(15), id: \.date) { (point: ScorePoint) in
                                    Circle()
                                        .fill(scoreColor(point.score))
                                        .frame(width: scoreDotSize(point.score), height: scoreDotSize(point.score))
                                }
                            }
                        }
                        .padding(.vertical, 4)
                    }
                }
            }
        }
    }

    // MARK: - Helpers

    private static let chartDateParser: DateFormatter = {
        let f = DateFormatter()
        f.dateFormat = "yyyy-MM-dd"
        return f
    }()

    private static let chartDateFormatter: DateFormatter = {
        let f = DateFormatter()
        f.dateFormat = "d MMM"
        return f
    }()

    private func formatChartDate(_ isoDate: String) -> String {
        guard let date = Self.chartDateParser.date(from: isoDate) else {
            return String(isoDate.suffix(5))
        }
        return Self.chartDateFormatter.string(from: date)
    }

    private func formatMinutes(_ minutes: UInt32) -> String {
        if minutes >= 60 {
            let h = minutes / 60
            let m = minutes % 60
            return m > 0 ? "\(h)h \(m)m" : "\(h)h"
        }
        return "\(minutes)m"
    }

    private func scoreColor(_ score: UInt8) -> Color {
        switch score {
        case 1: Color.dangerText.opacity(0.6)
        case 2: Color.warningText.opacity(0.4)
        case 3: Color.warningText.opacity(0.6)
        case 4: Color.successText.opacity(0.6)
        case 5: Color.successText.opacity(0.8)
        default: Color.textFaint
        }
    }

    private func scoreDotSize(_ score: UInt8) -> CGFloat {
        CGFloat(4 + score * 2)
    }
}

#Preview("AnalyticsDashboardView") {
    AnalyticsDashboardView()
        .environment(IntradaCore())
        .preferredColorScheme(.dark)
}
