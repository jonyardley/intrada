import SwiftUI
import Charts

/// Analytics dashboard with practice stats, charts, and insights.
struct AnalyticsDashboardView: View {
    @Environment(IntradaCore.self) private var core

    private var analytics: AnalyticsData? { core.viewModel.analytics }

    var body: some View {
        ScrollView {
            if let analytics {
                VStack(spacing: 20) {
                    // Weekly summary cards
                    WeeklySummarySection(summary: analytics.weeklySummary, streak: analytics.streak)

                    // Daily practice chart
                    if !analytics.dailyTotals.isEmpty {
                        DailyPracticeChart(totals: analytics.dailyTotals)
                    }

                    // Top items
                    if !analytics.topItems.isEmpty {
                        TopItemsSection(items: analytics.topItems)
                    }

                    // Score trends
                    if !analytics.scoreTrends.isEmpty {
                        ScoreTrendsSection(trends: analytics.scoreTrends)
                    }

                    // Score changes
                    if !analytics.scoreChanges.isEmpty {
                        ScoreChangesSection(changes: analytics.scoreChanges)
                    }

                    // Neglected items
                    if !analytics.neglectedItems.isEmpty {
                        NeglectedItemsSection(items: analytics.neglectedItems)
                    }
                }
                .padding()
            } else {
                EmptyStateView(
                    icon: "chart.xyaxis.line",
                    title: "No analytics yet",
                    message: "Complete some practice sessions to see your stats."
                )
            }
        }
        .navigationTitle("Analytics")
    }
}

// MARK: - Weekly Summary

private struct WeeklySummarySection: View {
    let summary: WeeklySummary
    let streak: PracticeStreak

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("This Week")
                .font(.headline)

            LazyVGrid(columns: [
                GridItem(.flexible()),
                GridItem(.flexible()),
            ], spacing: 12) {
                StatCard(
                    title: "Practice Time",
                    value: formatMinutes(summary.totalMinutes),
                    subtitle: summary.hasPrevWeekData ? trendText(summary.timeDirection, prev: formatMinutes(summary.prevTotalMinutes)) : nil,
                    icon: "clock"
                )

                StatCard(
                    title: "Sessions",
                    value: "\(summary.sessionCount)",
                    subtitle: summary.hasPrevWeekData ? trendText(summary.sessionsDirection, prev: "\(summary.prevSessionCount)") : nil,
                    icon: "play.circle"
                )

                StatCard(
                    title: "Items Covered",
                    value: "\(summary.itemsCovered)",
                    subtitle: summary.hasPrevWeekData ? trendText(summary.itemsDirection, prev: "\(summary.prevItemsCovered)") : nil,
                    icon: "music.note.list"
                )

                StatCard(
                    title: "Streak",
                    value: "\(streak.currentDays) day\(streak.currentDays == 1 ? "" : "s")",
                    icon: "flame"
                )
            }
        }
    }

    private func formatMinutes(_ minutes: UInt32) -> String {
        if minutes < 60 {
            return "\(minutes)m"
        }
        let h = minutes / 60
        let m = minutes % 60
        if m == 0 {
            return "\(h)h"
        }
        return "\(h)h \(m)m"
    }

    private func trendText(_ direction: Direction, prev: String) -> String {
        let arrow: String
        switch direction {
        case .up: arrow = "↑"
        case .down: arrow = "↓"
        case .same: arrow = "→"
        }
        return "\(arrow) prev: \(prev)"
    }
}

// MARK: - Daily Practice Chart

private struct DailyPracticeChart: View {
    let totals: [DailyPracticeTotal]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Daily Practice")
                .font(.headline)

            Chart(totals, id: \.date) { total in
                BarMark(
                    x: .value("Date", shortDate(total.date)),
                    y: .value("Minutes", total.minutes)
                )
                .foregroundStyle(.indigo.gradient)
                .cornerRadius(4)
            }
            .chartYAxisLabel("Minutes")
            .frame(height: 200)
            .padding()
            .background(.ultraThinMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 12))
        }
    }

    private func shortDate(_ isoDate: String) -> String {
        // Expect "yyyy-MM-dd" format
        let parts = isoDate.split(separator: "-")
        guard parts.count == 3 else { return isoDate }
        let day = parts[2]
        return String(day)
    }
}

// MARK: - Top Items

private struct TopItemsSection: View {
    let items: [ItemRanking]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Most Practised")
                .font(.headline)

            VStack(spacing: 8) {
                ForEach(Array(items.prefix(5).enumerated()), id: \.element.itemId) { index, item in
                    HStack(spacing: 12) {
                        Text("#\(index + 1)")
                            .font(.caption)
                            .fontWeight(.bold)
                            .foregroundStyle(.indigo)
                            .frame(width: 28)

                        VStack(alignment: .leading, spacing: 2) {
                            Text(item.itemTitle)
                                .font(.subheadline)
                                .fontWeight(.medium)
                                .lineLimit(1)
                            HStack(spacing: 8) {
                                TypeBadge(itemType: item.itemType)
                                Text("\(item.sessionCount) session\(item.sessionCount == 1 ? "" : "s")")
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                            }
                        }

                        Spacer()

                        Text(formatMinutes(item.totalMinutes))
                            .font(.caption)
                            .fontWeight(.medium)
                            .foregroundStyle(.secondary)
                    }
                    .padding(.vertical, 4)
                }
            }
            .padding()
            .background(.ultraThinMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 12))
        }
    }

    private func formatMinutes(_ minutes: UInt32) -> String {
        if minutes < 60 { return "\(minutes)m" }
        let h = minutes / 60
        let m = minutes % 60
        return m == 0 ? "\(h)h" : "\(h)h \(m)m"
    }
}

// MARK: - Score Trends

private struct ScoreTrendsSection: View {
    let trends: [ItemScoreTrend]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Score Trends")
                .font(.headline)

            VStack(spacing: 12) {
                ForEach(trends.prefix(5), id: \.itemId) { trend in
                    VStack(alignment: .leading, spacing: 6) {
                        HStack {
                            Text(trend.itemTitle)
                                .font(.subheadline)
                                .fontWeight(.medium)
                                .lineLimit(1)
                            Spacer()
                            ScoreIndicator(score: trend.latestScore)
                        }

                        if trend.scores.count > 1 {
                            Chart(trend.scores, id: \.date) { point in
                                LineMark(
                                    x: .value("Date", point.date),
                                    y: .value("Score", point.score)
                                )
                                .interpolationMethod(.catmullRom)
                                .foregroundStyle(.indigo)

                                PointMark(
                                    x: .value("Date", point.date),
                                    y: .value("Score", point.score)
                                )
                                .foregroundStyle(.indigo)
                                .symbolSize(20)
                            }
                            .chartYScale(domain: 0...5)
                            .frame(height: 60)
                        }
                    }
                    .padding()
                    .background(.ultraThinMaterial)
                    .clipShape(RoundedRectangle(cornerRadius: 10))
                }
            }
        }
    }
}

// MARK: - Score Changes

private struct ScoreChangesSection: View {
    let changes: [ScoreChange]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Recent Score Changes")
                .font(.headline)

            VStack(spacing: 8) {
                ForEach(changes.prefix(5), id: \.itemId) { change in
                    HStack {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(change.itemTitle)
                                .font(.subheadline)
                                .lineLimit(1)
                            if change.isNew {
                                Text("New score")
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                            }
                        }

                        Spacer()

                        HStack(spacing: 6) {
                            if let prev = change.previousScore {
                                ScoreIndicator(score: prev)
                                Image(systemName: "arrow.right")
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                            }
                            ScoreIndicator(score: change.currentScore)

                            if change.delta != 0 {
                                Text(change.delta > 0 ? "+\(change.delta)" : "\(change.delta)")
                                    .font(.caption2)
                                    .fontWeight(.bold)
                                    .foregroundStyle(change.delta > 0 ? .green : .red)
                            }
                        }
                    }
                    .padding(.vertical, 4)
                }
            }
            .padding()
            .background(.ultraThinMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 12))
        }
    }
}

// MARK: - Neglected Items

private struct NeglectedItemsSection: View {
    let items: [NeglectedItem]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Needs Attention")
                .font(.headline)

            VStack(spacing: 8) {
                ForEach(items.prefix(5), id: \.itemId) { item in
                    HStack {
                        Image(systemName: "exclamationmark.triangle")
                            .font(.caption)
                            .foregroundStyle(.orange)

                        Text(item.itemTitle)
                            .font(.subheadline)
                            .lineLimit(1)

                        Spacer()

                        if let days = item.daysSincePractice {
                            Text("\(days)d ago")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        } else {
                            Text("Never practised")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                    .padding(.vertical, 4)
                }
            }
            .padding()
            .background(.ultraThinMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 12))
        }
    }
}
