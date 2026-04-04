import SwiftUI

/// Between-item transition sheet with optional scoring.
///
/// Shows "Up Next" preview, 1–5 score selector, tempo input,
/// notes field, and Continue/Skip buttons. On the last item,
/// shows "Finish" instead of "Continue".
struct TransitionPromptSheet: View {
    let session: ActiveSessionView
    let isLastItem: Bool
    let onContinue: (UInt8?, UInt16?, String?) -> Void
    let onSkip: () -> Void

    @State private var selectedScore: UInt8? = nil
    @State private var selectedTempo: Int = 0
    @State private var notesText: String = ""

    /// Default tempo from the item's target BPM or last achieved tempo.
    private var defaultTempo: Int {
        let pos = Int(session.currentPosition)
        guard pos < session.entries.count else { return 0 }
        let entry = session.entries[pos]
        // Prefer last achieved tempo, then fall back to 0 (no selection)
        if let achieved = entry.achievedTempo { return Int(achieved) }
        return 0
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Header — Up Next / Session Complete
            VStack(alignment: .leading, spacing: 4) {
                Text(isLastItem ? "Session Complete" : "Up Next")
                    .font(.system(size: 11, weight: .semibold))
                    .tracking(1.5)
                    .foregroundStyle(Color.textMuted)

                if isLastItem {
                    Text("Ready to finish?")
                        .font(.system(size: 20, weight: .bold))
                        .foregroundStyle(Color.textPrimary)
                } else if let nextTitle = session.nextItemTitle {
                    Text(nextTitle)
                        .font(.system(size: 20, weight: .bold))
                        .foregroundStyle(Color.textPrimary)
                }

                // Next item badge
                if !isLastItem {
                    nextItemBadge
                }
            }
            .padding(.horizontal, Spacing.cardComfortable)
            .padding(.top, Spacing.card)
            .padding(.bottom, Spacing.cardCompact)

            Divider().background(Color.borderDefault)

            // Scoring section
            VStack(alignment: .leading, spacing: 12) {
                Text("How did it go?")
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundStyle(Color.textSecondary)

                ScoreSelectorView(selectedScore: $selectedScore)
                    .frame(maxWidth: .infinity)

                // Tempo picker
                VStack(alignment: .leading, spacing: 4) {
                    Text("Tempo achieved (BPM)")
                        .font(.system(size: 13, weight: .medium))
                        .foregroundStyle(Color.textMuted)

                    Picker("BPM", selection: $selectedTempo) {
                        Text("—").tag(0)
                        ForEach(30...300, id: \.self) { bpm in
                            Text("\(bpm)").tag(bpm)
                        }
                    }
                    .pickerStyle(.wheel)
                    .frame(height: 100)
                    .clipped()
                }

                // Notes input
                HStack(spacing: 12) {
                    Text("Notes")
                        .font(.system(size: 13, weight: .medium))
                        .foregroundStyle(Color.textMuted)

                    TextField("", text: $notesText)
                        .font(.system(size: 13))
                        .foregroundStyle(Color.textPrimary)
                        .padding(.horizontal, 12)
                        .frame(height: 36)
                        .background(Color.surfaceInput)
                        .clipShape(RoundedRectangle(cornerRadius: DesignRadius.input))
                        .overlay(
                            RoundedRectangle(cornerRadius: DesignRadius.input)
                                .stroke(Color.borderInput, lineWidth: 1)
                        )
                }
            }
            .padding(.horizontal, Spacing.cardComfortable)
            .padding(.vertical, Spacing.cardCompact)

            Divider().background(Color.borderDefault)

            // Actions
            VStack(spacing: 8) {
                ButtonView(isLastItem ? "Finish" : "Continue", variant: .primary) {
                    let tempo: UInt16? = selectedTempo > 0 ? UInt16(selectedTempo) : nil
                    let notes: String? = notesText.isEmpty ? nil : notesText
                    onContinue(selectedScore, tempo, notes)
                }

                Button {
                    onSkip()
                } label: {
                    Text("Skip scoring")
                        .font(.system(size: 12, weight: .medium))
                        .foregroundStyle(Color.textFaint)
                }
            }
            .padding(.horizontal, Spacing.cardComfortable)
            .padding(.top, Spacing.cardCompact)
            .padding(.bottom, 40)
        }
        .background(Color.backgroundApp)
        .onAppear {
            selectedScore = nil
            selectedTempo = defaultTempo
            notesText = ""
        }
    }

    @ViewBuilder
    private var nextItemBadge: some View {
        // Find the next entry to show its type badge
        let nextPos = Int(session.currentPosition) + 1
        if nextPos < session.entries.count {
            TypeBadge(kind: session.entries[nextPos].itemType)
        }
    }
}

#Preview("TransitionPromptSheet") {
    TransitionPromptSheet(
        session: ActiveSessionView(
            currentItemTitle: "Scales in C Major",
            currentItemType: .exercise,
            currentPosition: 1,
            totalItems: 3,
            startedAt: "",
            entries: [],
            sessionIntention: nil,
            currentRepTarget: nil,
            currentRepCount: nil,
            currentRepTargetReached: nil,
            currentRepHistory: nil,
            currentPlannedDurationSecs: 600,
            nextItemTitle: "Bach Prelude in G"
        ),
        isLastItem: false,
        onContinue: { _, _, _ in },
        onSkip: { }
    )
    .preferredColorScheme(.dark)
}
