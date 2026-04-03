import SwiftUI

/// Displays one entry result from a session — shared by summary (editable)
/// and history detail (read-only).
///
/// Shows status icon, title, type badge, duration, and optional
/// score/tempo/rep/notes badges. When editable, score and tempo are
/// interactive and notes has a text field.
struct SessionEntryResultRow: View {
    let entry: SetlistEntryView
    let isEditable: Bool
    var onScoreChanged: ((UInt8?) -> Void)? = nil
    var onTempoChanged: ((UInt16?) -> Void)? = nil
    var onNotesChanged: ((String?) -> Void)? = nil

    @State private var selectedScore: UInt8? = nil
    @State private var notesText: String = ""
    @State private var selectedTempo: Int = 0

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            statusIcon

            VStack(alignment: .leading, spacing: 6) {
                // Title row
                HStack(spacing: 8) {
                    Text(entry.itemTitle)
                        .font(.system(size: 15, weight: .semibold))
                        .foregroundStyle(entry.status == .completed ? Color.textPrimary : Color.textMuted)
                        .lineLimit(1)

                    TypeBadge(kind: entry.itemType)

                    Spacer()

                    Text(durationText)
                        .font(.system(size: 12, weight: .medium))
                        .foregroundStyle(Color.textMuted)
                }

                // Badges row
                if hasBadges {
                    FlowLayout(spacing: 6) {
                        if let score = displayScore {
                            badge("★ \(score)", color: .warmAccentText, bg: .surfaceSecondary)
                        }
                        if let tempo = entry.achievedTempo {
                            badge("♪ \(tempo) BPM", color: .accentText, bg: .surfaceSecondary)
                        }
                        if let target = entry.repTarget {
                            let count = entry.repCount ?? 0
                            let reached = entry.repTargetReached ?? false
                            badge(
                                "\(count)/\(target) reps",
                                color: reached ? .successText : .textMuted,
                                bg: reached ? .successSurface : .surfaceSecondary
                            )
                        }
                        if entry.status == .skipped {
                            badge("Skipped", color: .textFaint, bg: .surfaceSecondary)
                        }
                        if entry.status == .notAttempted {
                            badge("Not attempted", color: .textFaint, bg: .surfaceSecondary)
                        }
                    }
                }

                // Intention
                if let intention = entry.intention, !intention.isEmpty {
                    Text(intention)
                        .font(.system(size: 12))
                        .foregroundStyle(Color.textMuted)
                        .fontWeight(.regular)
                        .italic()
                }

                // Notes (display or edit)
                if isEditable && entry.status == .completed {
                    notesEditor
                } else if let notes = entry.notes, !notes.isEmpty {
                    Text(notes)
                        .font(.system(size: 12))
                        .foregroundStyle(Color.textMuted)
                        .italic()
                }

                // Editable score
                if isEditable && entry.status == .completed {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Confidence")
                            .font(.system(size: 11, weight: .medium))
                            .foregroundStyle(Color.textFaint)
                        ScoreSelectorView(selectedScore: $selectedScore)
                    }
                }

                // Editable tempo
                if isEditable && entry.status == .completed {
                    tempoEditor
                }
            }
        }
        .padding(.vertical, 12)
        .onAppear {
            selectedScore = entry.score
            notesText = entry.notes ?? ""
            selectedTempo = Int(entry.achievedTempo ?? 0)
        }
        .onChange(of: selectedScore) { oldScore, newScore in
            // Guard against dispatching on initial load — only fire when user changes the value
            guard oldScore != nil || entry.score == nil else { return }
            guard newScore != entry.score else { return }
            onScoreChanged?(newScore)
        }
    }

    // MARK: - Status Icon

    @ViewBuilder
    private var statusIcon: some View {
        switch entry.status {
        case .completed:
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 18))
                .foregroundStyle(Color.successText)
        case .skipped:
            Image(systemName: "xmark.circle")
                .font(.system(size: 18))
                .foregroundStyle(Color.textFaint)
        case .notAttempted:
            Image(systemName: "minus.circle")
                .font(.system(size: 18))
                .foregroundStyle(Color.textFaint)
        }
    }

    // MARK: - Notes Editor

    private var notesEditor: some View {
        TextField("Add notes...", text: $notesText)
            .font(.system(size: 12))
            .foregroundStyle(Color.textPrimary)
            .padding(.horizontal, 8)
            .padding(.vertical, 6)
            .background(Color.surfaceInput)
            .clipShape(RoundedRectangle(cornerRadius: 6))
            .overlay(
                RoundedRectangle(cornerRadius: 6)
                    .stroke(Color.borderInput, lineWidth: 1)
            )
            .onSubmit {
                onNotesChanged?(notesText.isEmpty ? nil : notesText)
            }
    }

    // MARK: - Tempo Editor

    private var tempoEditor: some View {
        HStack(spacing: 8) {
            Text("Tempo achieved")
                .font(.system(size: 11, weight: .medium))
                .foregroundStyle(Color.textFaint)

            Picker("BPM", selection: $selectedTempo) {
                Text("—").tag(0)
                ForEach(30...300, id: \.self) { bpm in
                    Text("\(bpm)").tag(bpm)
                }
            }
            .pickerStyle(.menu)
            .tint(Color.accentText)
            .onChange(of: selectedTempo) { _, newTempo in
                onTempoChanged?(newTempo > 0 ? UInt16(newTempo) : nil)
            }
        }
    }

    // MARK: - Helpers

    private var displayScore: UInt8? {
        isEditable ? nil : entry.score // Don't show badge when score selector is shown
    }

    private var durationText: String {
        if entry.status == .skipped { return "Skipped" }
        if entry.status == .notAttempted { return "—" }
        return entry.durationDisplay
    }

    private var hasBadges: Bool {
        displayScore != nil || entry.achievedTempo != nil || entry.repTarget != nil
            || entry.status == .skipped || entry.status == .notAttempted
    }

    @ViewBuilder
    private func badge(_ text: String, color: Color, bg: Color) -> some View {
        Text(text)
            .font(.system(size: 11, weight: .medium))
            .foregroundStyle(color)
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(bg)
            .clipShape(RoundedRectangle(cornerRadius: DesignRadius.badge))
    }
}

#Preview("SessionEntryResultRow") {
    let completed = SetlistEntryView(
        id: "e1", itemId: "i1", itemTitle: "Hanon Exercise #1",
        itemType: .exercise, position: 0, durationDisplay: "5:12",
        status: .completed, notes: "Felt solid", score: 4,
        intention: "Even tempo", repTarget: 5, repCount: 5,
        repTargetReached: true, repHistory: nil,
        plannedDurationSecs: 300, plannedDurationDisplay: "5 min",
        achievedTempo: 120
    )
    let skipped = SetlistEntryView(
        id: "e2", itemId: "i2", itemTitle: "Bach Prelude in G",
        itemType: .piece, position: 1, durationDisplay: "0:00",
        status: .skipped, notes: nil, score: nil,
        intention: nil, repTarget: nil, repCount: nil,
        repTargetReached: nil, repHistory: nil,
        plannedDurationSecs: nil, plannedDurationDisplay: nil,
        achievedTempo: nil
    )

    VStack(spacing: 0) {
        Text("Read-only").font(.caption2).foregroundStyle(Color.textFaint)
        SessionEntryResultRow(entry: completed, isEditable: false)
            .padding(.horizontal, 24)
        Divider().background(Color.borderDefault)
        SessionEntryResultRow(entry: skipped, isEditable: false)
            .padding(.horizontal, 24)
        Divider().background(Color.borderDefault)
        Text("Editable").font(.caption2).foregroundStyle(Color.textFaint).padding(.top, 16)
        SessionEntryResultRow(entry: completed, isEditable: true)
            .padding(.horizontal, 24)
    }
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
