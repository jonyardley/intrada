import SwiftUI

/// Full item detail view with practice history, score/tempo charts.
struct ItemDetailView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.dismiss) private var dismiss

    let itemId: String

    @State private var showingEdit = false
    @State private var showingDeleteConfirm = false

    private var item: LibraryItemView? {
        core.viewModel.items.first(where: { $0.id == itemId })
    }

    var body: some View {
        Group {
            if let item {
                ScrollView {
                    VStack(alignment: .leading, spacing: 20) {
                        headerSection(item)
                        metadataSection(item)
                        if let practice = item.practice, practice.sessionCount > 0 {
                            practiceSection(practice, achievedTempo: item.latestAchievedTempo)
                        }
                    }
                    .padding()
                }
                .navigationTitle(item.title)
                .navigationBarTitleDisplayMode(.large)
                .toolbar {
                    ToolbarItem(placement: .primaryAction) {
                        Menu {
                            Button { showingEdit = true } label: {
                                Label("Edit", systemImage: "pencil")
                            }
                            Button(role: .destructive) {
                                showingDeleteConfirm = true
                            } label: {
                                Label("Delete", systemImage: "trash")
                            }
                        } label: {
                            Image(systemName: "ellipsis.circle")
                        }
                    }
                }
                .sheet(isPresented: $showingEdit) {
                    NavigationStack {
                        EditItemView(item: item)
                    }
                }
                .deleteConfirmation(
                    "Delete \(item.title)?",
                    message: "This will permanently remove this item and its practice history.",
                    isPresented: $showingDeleteConfirm
                ) {
                    core.update(.item(.delete(id: item.id)))
                    dismiss()
                }
            } else {
                ContentUnavailableView("Item not found", systemImage: "music.note")
            }
        }
    }

    // MARK: - Sections

    @ViewBuilder
    private func headerSection(_ item: LibraryItemView) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                TypeBadge(itemType: item.itemType)
                Spacer()
            }

            if !item.subtitle.isEmpty {
                Text(item.subtitle)
                    .font(.title3)
                    .foregroundStyle(.secondary)
            }

            if !item.tags.isEmpty {
                FlowLayout(spacing: 6) {
                    ForEach(item.tags, id: \.self) { tag in
                        Text(tag)
                            .font(.caption)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 4)
                            .background(.quaternary)
                            .clipShape(Capsule())
                    }
                }
            }
        }
    }

    @ViewBuilder
    private func metadataSection(_ item: LibraryItemView) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Details")
                .font(.headline)

            LazyVGrid(columns: [
                GridItem(.flexible()),
                GridItem(.flexible()),
            ], alignment: .leading, spacing: 12) {
                if let category = item.category, !category.isEmpty {
                    MetadataField(label: "Category", value: category)
                }
                if let key = item.key, !key.isEmpty {
                    MetadataField(label: "Key", value: key)
                }
                if let tempo = item.tempo, !tempo.isEmpty {
                    MetadataField(label: "Tempo", value: tempo)
                }
            }

            if let notes = item.notes, !notes.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Notes")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Text(notes)
                        .font(.subheadline)
                }
            }
        }
        .padding()
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }

    @ViewBuilder
    private func practiceSection(_ practice: ItemPracticeSummary, achievedTempo: UInt16?) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Practice Summary")
                .font(.headline)

            HStack(spacing: 16) {
                StatCard(
                    title: "Sessions",
                    value: "\(practice.sessionCount)",
                    icon: "music.note"
                )

                StatCard(
                    title: "Total Time",
                    value: formatMinutes(practice.totalMinutes),
                    icon: "clock"
                )
            }

            if let score = practice.latestScore {
                HStack {
                    Text("Latest Score")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                    Spacer()
                    ScoreIndicator(score: score)
                }
            }

            if let tempo = achievedTempo {
                HStack {
                    Text("Latest Tempo")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                    Spacer()
                    Text("\(tempo) BPM")
                        .font(.subheadline)
                        .fontWeight(.medium)
                }
            }

            // Score history chart
            if practice.scoreHistory.count >= 2 {
                ScoreHistoryChart(entries: practice.scoreHistory)
            }

            // Tempo history chart
            if practice.tempoHistory.count >= 2 {
                TempoHistoryChart(entries: practice.tempoHistory)
            }
        }
        .padding()
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }

    private func formatMinutes(_ minutes: UInt32) -> String {
        if minutes >= 60 {
            let h = minutes / 60
            let m = minutes % 60
            return m > 0 ? "\(h)h \(m)m" : "\(h)h"
        }
        return "\(minutes)m"
    }
}

// MARK: - Metadata Field

private struct MetadataField: View {
    let label: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label)
                .font(.caption)
                .foregroundStyle(.secondary)
            Text(value)
                .font(.subheadline)
        }
    }
}

// MARK: - Score History Chart (Swift Charts)

import Charts

struct ScoreHistoryChart: View {
    let entries: [ScoreHistoryEntry]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Score History")
                .font(.caption)
                .foregroundStyle(.secondary)

            Chart(entries, id: \.sessionId) { entry in
                LineMark(
                    x: .value("Date", entry.sessionDate),
                    y: .value("Score", entry.score)
                )
                .foregroundStyle(.indigo)

                PointMark(
                    x: .value("Date", entry.sessionDate),
                    y: .value("Score", entry.score)
                )
                .foregroundStyle(.indigo)
            }
            .chartYScale(domain: 1...5)
            .chartYAxis {
                AxisMarks(values: [1, 2, 3, 4, 5])
            }
            .frame(height: 120)
        }
    }
}

// MARK: - Tempo History Chart

struct TempoHistoryChart: View {
    let entries: [TempoHistoryEntry]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Tempo Progress")
                .font(.caption)
                .foregroundStyle(.secondary)

            Chart(entries, id: \.sessionId) { entry in
                LineMark(
                    x: .value("Date", entry.sessionDate),
                    y: .value("BPM", entry.tempo)
                )
                .foregroundStyle(.teal)

                PointMark(
                    x: .value("Date", entry.sessionDate),
                    y: .value("BPM", entry.tempo)
                )
                .foregroundStyle(.teal)
            }
            .frame(height: 120)
        }
    }
}

// MARK: - Flow Layout (for tags)

struct FlowLayout: Layout {
    var spacing: CGFloat = 8

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let result = layout(subviews: subviews, width: proposal.width ?? .infinity)
        return result.size
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let result = layout(subviews: subviews, width: bounds.width)
        for (index, position) in result.positions.enumerated() {
            subviews[index].place(
                at: CGPoint(x: bounds.minX + position.x, y: bounds.minY + position.y),
                proposal: ProposedViewSize(subviews[index].sizeThatFits(.unspecified))
            )
        }
    }

    private func layout(subviews: Subviews, width: CGFloat) -> (size: CGSize, positions: [CGPoint]) {
        var positions: [CGPoint] = []
        var currentX: CGFloat = 0
        var currentY: CGFloat = 0
        var lineHeight: CGFloat = 0
        var maxWidth: CGFloat = 0

        for subview in subviews {
            let size = subview.sizeThatFits(.unspecified)
            if currentX + size.width > width, currentX > 0 {
                currentX = 0
                currentY += lineHeight + spacing
                lineHeight = 0
            }
            positions.append(CGPoint(x: currentX, y: currentY))
            lineHeight = max(lineHeight, size.height)
            currentX += size.width + spacing
            maxWidth = max(maxWidth, currentX - spacing)
        }

        return (CGSize(width: maxWidth, height: currentY + lineHeight), positions)
    }
}
