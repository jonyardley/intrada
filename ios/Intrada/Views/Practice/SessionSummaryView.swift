import SwiftUI

/// Post-session review screen with inline editing.
///
/// Shows header stats, per-item results with editable scores/tempo/notes,
/// session notes, and Save/Discard actions. Replaces SummaryPlaceholderView.
struct SessionSummaryView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.horizontalSizeClass) private var sizeClass

    @State private var sessionNotesText: String = ""
    @State private var showDiscardConfirmation: Bool = false
    @State private var sessionNotesCommitTask: Task<Void, Never>? = nil

    private var summary: SummaryView? {
        core.viewModel.summary
    }

    var body: some View {
        Group {
            if sizeClass == .regular {
                iPadLayout
            } else {
                iPhoneLayout
            }
        }
        .background(Color.backgroundApp)
        .navigationTitle("Session Summary")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            sessionNotesText = summary?.notes ?? ""
        }
        .confirmationDialog(
            "Discard this session?",
            isPresented: $showDiscardConfirmation,
            titleVisibility: .visible
        ) {
            Button("Discard", role: .destructive) {
                core.update(.session(.discardSession))
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("All progress will be lost.")
        }
    }

    // MARK: - iPhone Layout

    private var iPhoneLayout: some View {
        ScrollView {
            VStack(spacing: Spacing.cardCompact) {
                headerSection

                CardView(padding: 0) {
                    entryList
                }
                .padding(.horizontal, Spacing.card)

                CardView {
                    sessionNotesContent
                }
                .padding(.horizontal, Spacing.card)

                actionsSection
            }
        }
    }

    // MARK: - iPad Layout

    private var iPadLayout: some View {
        HStack(spacing: 0) {
            // Left: stats + notes + actions
            ScrollView {
                VStack(spacing: Spacing.cardCompact) {
                    headerSection

                    CardView {
                        sessionNotesContent
                    }
                    .padding(.horizontal, Spacing.card)

                    actionsSection
                }
            }
            .frame(width: 360)

            Divider().background(Color.borderDefault)

            // Right: entry list
            ScrollView {
                VStack(spacing: Spacing.cardCompact) {
                    HStack {
                        Text("ITEMS PRACTICED")
                            .font(.system(size: 9, weight: .semibold))
                            .tracking(1.5)
                            .foregroundStyle(Color.textFaint)
                        Spacer()
                    }
                    .padding(.horizontal, Spacing.cardComfortable)
                    .padding(.top, Spacing.card)

                    CardView(padding: 0) {
                        entryList
                    }
                    .padding(.horizontal, Spacing.card)
                }
            }
        }
    }

    // MARK: - Header

    private var headerSection: some View {
        VStack(spacing: 8) {
            Image(systemName: "checkmark.circle")
                .font(.system(size: 40))
                .foregroundStyle(Color.successText)

            Text("Session Complete!")
                .font(.heading(size: 22))
                .foregroundStyle(Color.textPrimary)

            if let summary {
                HStack(spacing: 8) {
                    Text(summary.totalDurationDisplay)
                        .font(.system(size: 14, weight: .medium))
                        .foregroundStyle(Color.textSecondary)

                    Text("·")
                        .foregroundStyle(Color.textFaint)

                    Text("\(summary.entries.count) items")
                        .font(.system(size: 14, weight: .medium))
                        .foregroundStyle(Color.textSecondary)

                    Text("·")
                        .foregroundStyle(Color.textFaint)

                    completionBadge(summary.completionStatus)
                }

                if let intention = summary.sessionIntention, !intention.isEmpty {
                    Text(intention)
                        .font(.system(size: 13))
                        .foregroundStyle(Color.textMuted)
                        .italic()
                }
            }
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, Spacing.cardComfortable)
        .padding(.horizontal, Spacing.cardComfortable)
    }

    // MARK: - Entry List

    private var entryList: some View {
        LazyVStack(spacing: 0) {
            if let summary {
                ForEach(Array(summary.entries.enumerated()), id: \.element.id) { (index: Int, entry: SetlistEntryView) in
                    if index > 0 {
                        Divider().background(Color.borderDefault)
                    }

                    SessionEntryResultRow(
                        entry: entry,
                        isEditable: true,
                        onScoreChanged: { score in
                            core.update(.session(.updateEntryScore(entryId: entry.id, score: score)))
                        },
                        onTempoChanged: { tempo in
                            core.update(.session(.updateEntryTempo(entryId: entry.id, tempo: tempo)))
                        },
                        onNotesChanged: { notes in
                            core.update(.session(.updateEntryNotes(entryId: entry.id, notes: notes)))
                        }
                    )
                    .padding(.horizontal, Spacing.cardComfortable)
                }
            }
        }
    }

    // MARK: - Session Notes

    private var sessionNotesContent: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Session Notes")
                .font(.system(size: 13, weight: .semibold))
                .foregroundStyle(Color.textSecondary)

            TextField("How did this session go?", text: $sessionNotesText, axis: .vertical)
                .font(.system(size: 13))
                .foregroundStyle(Color.textPrimary)
                .lineLimit(3...6)
                .padding(Spacing.cardCompact)
                .background(Color.surfaceInput)
                .clipShape(RoundedRectangle(cornerRadius: DesignRadius.input))
                .overlay(
                    RoundedRectangle(cornerRadius: DesignRadius.input)
                        .stroke(Color.borderInput, lineWidth: 1)
                )
                .onSubmit {
                    commitSessionNotes()
                }
                .onChange(of: sessionNotesText) {
                    sessionNotesCommitTask?.cancel()
                    sessionNotesCommitTask = Task {
                        try? await Task.sleep(for: .seconds(1))
                        guard !Task.isCancelled else { return }
                        commitSessionNotes()
                    }
                }
        }
    }

    // MARK: - Actions

    private var actionsSection: some View {
        VStack(spacing: Spacing.cardCompact) {
            ButtonView("Save Session", variant: .primary) {
                let now = ISO8601DateFormatter().string(from: Date())
                core.update(.session(.saveSession(now: now)))
            }

            RoutineSaveForm { name in
                core.update(.routine(.saveSummaryAsRoutine(name: name)))
            }

            ButtonView("Discard", variant: .dangerOutline) {
                showDiscardConfirmation = true
            }
        }
        .padding(.horizontal, Spacing.cardComfortable)
        .padding(.top, 8)
        .padding(.bottom, Spacing.section)
    }

    // MARK: - Helpers

    private func commitSessionNotes() {
        core.update(.session(.updateSessionNotes(notes: sessionNotesText.isEmpty ? nil : sessionNotesText)))
    }

    @ViewBuilder
    private func completionBadge(_ status: CompletionStatus) -> some View {
        switch status {
        case .completed:
            Text("Completed")
                .font(.system(size: 14, weight: .medium))
                .foregroundStyle(Color.successText)
        case .endedEarly:
            Text("Ended Early")
                .font(.system(size: 12, weight: .semibold))
                .foregroundStyle(Color.warmAccentText)
                .padding(.horizontal, 8)
                .padding(.vertical, 2)
                .background(Color.warmAccentSurface)
                .clipShape(RoundedRectangle(cornerRadius: DesignRadius.badge))
        }
    }
}

#Preview("SessionSummaryView") {
    SessionSummaryView()
        .environment(IntradaCore())
        .preferredColorScheme(.dark)
}
