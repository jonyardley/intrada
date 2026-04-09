import SwiftUI

/// Full item detail view showing all populated fields and practice summary.
/// Accessed by tapping an item in the library list.
struct ItemDetailView: View {
    let itemId: String
    @Binding var selectedItemId: String?
    @Environment(IntradaCore.self) private var core
    @Environment(ToastManager.self) private var toast
    @Environment(\.horizontalSizeClass) private var sizeClass
    @Environment(\.dismiss) private var dismiss
    @State private var showEditSheet: Bool = false
    @State private var showDeleteConfirmation: Bool = false

    private var item: LibraryItemView? {
        core.viewModel.items.first(where: { $0.id == itemId })
    }

    var body: some View {
        Group {
            if let item {
                ScrollView {
                    if sizeClass == .regular {
                        iPadLayout(item: item)
                    } else {
                        iPhoneLayout(item: item)
                    }
                }
                .navigationTitle(item.title)
                .navigationBarTitleDisplayMode(sizeClass == .regular ? .inline : .large)
                .toolbar {
                    ToolbarItem(placement: .primaryAction) {
                        Menu {
                            Button {
                                showEditSheet = true
                            } label: {
                                Label("Edit", systemImage: "pencil")
                            }

                            Button(role: .destructive) {
                                showDeleteConfirmation = true
                            } label: {
                                Label("Delete", systemImage: "trash")
                            }
                        } label: {
                            Image(systemName: "ellipsis.circle")
                                .accessibilityLabel("Item actions")
                        }
                    }
                }
                .confirmationDialog(
                    "Delete \(item.title)?",
                    isPresented: $showDeleteConfirmation,
                    titleVisibility: .visible
                ) {
                    Button("Delete", role: .destructive) {
                        core.update(.item(.delete(id: itemId)))
                        selectedItemId = nil
                        dismiss()
                        toast.show("Item deleted", variant: .success)
                    }
                    Button("Cancel", role: .cancel) {}
                } message: {
                    Text("This cannot be undone.")
                }
                .sheet(isPresented: $showEditSheet) {
                    NavigationStack {
                        EditItemView(itemId: itemId)
                    }
                }
            } else if core.isLoading {
                DetailSkeletonView()
            } else {
                EmptyStateView(
                    icon: "questionmark.circle",
                    title: "Item Not Found",
                    message: "This item may have been deleted"
                )
            }
        }
        .background(Color.backgroundApp)
    }

    // MARK: - iPhone Layout (single column)

    @ViewBuilder
    private func iPhoneLayout(item: LibraryItemView) -> some View {
        VStack(alignment: .leading, spacing: 20) {
            headerSection(item: item)
            metadataCard(item: item)
            tagsSection(item: item)
            notesCard(item: item)
            practiceSummarySection(item: item)
            timestampsSection(item: item)
        }
        .padding(Spacing.card)
    }

    // MARK: - iPad Layout (two columns)

    @ViewBuilder
    private func iPadLayout(item: LibraryItemView) -> some View {
        VStack(alignment: .leading, spacing: 24) {
            headerSection(item: item)

            // Main content card with metadata
            CardView {
                VStack(alignment: .leading, spacing: 16) {
                    metadataFields(item: item)
                    tagsSection(item: item)
                    notesInline(item: item)
                    timestampsSection(item: item)
                }
            }

            // Practice summary below in full width
            practiceSummarySection(item: item)
        }
        .padding(Spacing.cardComfortable)
    }

    // MARK: - Sections

    @ViewBuilder
    private func headerSection(item: LibraryItemView) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            // On iPhone, the large navigation title shows the title.
            // On iPad, the title is inline so we show it prominently here.
            if sizeClass == .regular {
                Text(item.title)
                    .font(.heading(size: 28))
                    .foregroundStyle(Color.textPrimary)
                    .lineLimit(3)
            }

            if !item.subtitle.isEmpty {
                Text(item.subtitle)
                    .font(.title3)
                    .foregroundStyle(Color.textSecondary)
            }

            TypeBadge(kind: item.itemType)
        }
    }

    /// Inline metadata fields without card wrapping (for use inside iPad's combined card).
    @ViewBuilder
    private func metadataFields(item: LibraryItemView) -> some View {
        let hasKey = item.key?.isEmpty == false
        let hasTempo = item.tempo?.isEmpty == false

        if hasKey || hasTempo {
            HStack(spacing: 24) {
                if let key = item.key, !key.isEmpty {
                    metadataRow(label: "KEY", value: key)
                }
                if let tempo = item.tempo, !tempo.isEmpty {
                    metadataRow(label: "TEMPO", value: tempo)
                }
                Spacer()
            }
        }
    }

    /// Notes without card wrapping (for use inside iPad's combined card).
    @ViewBuilder
    private func notesInline(item: LibraryItemView) -> some View {
        if let notes = item.notes, !notes.isEmpty {
            VStack(alignment: .leading, spacing: 8) {
                Text("NOTES")
                    .fieldLabelStyle()
                Text(notes)
                    .font(.body)
                    .foregroundStyle(Color.textSecondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
    }

    @ViewBuilder
    private func metadataCard(item: LibraryItemView) -> some View {
        let hasKey = item.key?.isEmpty == false
        let hasTempo = item.tempo?.isEmpty == false

        if hasKey || hasTempo {
            CardView {
                VStack(alignment: .leading, spacing: 12) {
                    if let key = item.key, !key.isEmpty {
                        metadataRow(label: "KEY", value: key)
                    }
                    if let tempo = item.tempo, !tempo.isEmpty {
                        metadataRow(label: "TEMPO", value: tempo)
                    }
                }
            }
        }
    }

    private func metadataRow(label: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label)
                .fieldLabelStyle()
            Text(value)
                .font(.body)
                .foregroundStyle(Color.textPrimary)
        }
    }

    @ViewBuilder
    private func tagsSection(item: LibraryItemView) -> some View {
        if !item.tags.isEmpty {
            VStack(alignment: .leading, spacing: 8) {
                Text("TAGS")
                    .fieldLabelStyle()

                FlowLayout(spacing: 6) {
                    ForEach(item.tags, id: \.self) { tag in
                        Text(tag)
                            .font(.caption)
                            .foregroundStyle(Color.textSecondary)
                            .padding(.horizontal, 10)
                            .padding(.vertical, 4)
                            .background(Color.surfaceSecondary)
                            .clipShape(Capsule())
                            .overlay(
                                Capsule()
                                    .strokeBorder(Color.borderDefault, lineWidth: 1)
                            )
                    }
                }
            }
        }
    }

    @ViewBuilder
    private func notesCard(item: LibraryItemView) -> some View {
        if let notes = item.notes, !notes.isEmpty {
            CardView {
                VStack(alignment: .leading, spacing: 8) {
                    Text("NOTES")
                        .fieldLabelStyle()
                    Text(notes)
                        .font(.body)
                        .foregroundStyle(Color.textSecondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
        }
    }

    @ViewBuilder
    private func practiceSummarySection(item: LibraryItemView) -> some View {
        if let practice = item.practice {
            VStack(alignment: .leading, spacing: 16) {
                Text("PRACTICE SUMMARY")
                    .fieldLabelStyle()

                HStack(spacing: 12) {
                    StatCardView(
                        title: "Sessions",
                        value: "\(practice.sessionCount)"
                    )
                    StatCardView(
                        title: "Minutes",
                        value: "\(practice.totalMinutes)"
                    )
                    if let score = practice.latestScore {
                        StatCardView(
                            title: "Confidence",
                            value: "\(score)/5"
                        )
                    }
                }

                if !practice.scoreHistory.isEmpty {
                    ScoreHistoryList(entries: practice.scoreHistory)
                }

                if !practice.tempoHistory.isEmpty {
                    tempoHistorySection(entries: practice.tempoHistory)
                }
            }
        }
    }

    @ViewBuilder
    private func tempoHistorySection(entries: [TempoHistoryEntry]) -> some View {
        CardView {
            VStack(alignment: .leading, spacing: 8) {
                Text("TEMPO HISTORY")
                    .fieldLabelStyle()

                ForEach(entries, id: \.sessionId) { entry in
                    HStack {
                        Text(formatDate(entry.sessionDate))
                            .font(.caption)
                            .foregroundStyle(Color.textFaint)
                        Spacer()
                        Text("\(entry.tempo) BPM")
                            .font(.caption)
                            .fontWeight(.medium)
                            .foregroundStyle(Color.textSecondary)
                    }
                }
            }
        }
    }

    @ViewBuilder
    private func timestampsSection(item: LibraryItemView) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Created \(formatDate(item.createdAt))")
                .font(.caption)
                .foregroundStyle(Color.textFaint)
            Text("Updated \(formatDate(item.updatedAt))")
                .font(.caption)
                .foregroundStyle(Color.textFaint)
        }
    }
}

#Preview {
    Text("ItemDetailView Preview")
        .foregroundStyle(Color.textMuted)
        .preferredColorScheme(.dark)
}
