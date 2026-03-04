import SwiftUI

/// Full detail view for a completed practice session.
struct SessionDetailView: View {
    @Environment(IntradaCore.self) private var core

    let sessionId: String

    private var session: PracticeSessionView? {
        core.viewModel.sessions.first(where: { $0.id == sessionId })
    }

    var body: some View {
        Group {
            if let session {
                ScrollView {
                    VStack(alignment: .leading, spacing: 20) {
                        // Header
                        VStack(alignment: .leading, spacing: 8) {
                            HStack {
                                Text(session.totalDurationDisplay)
                                    .font(.title2)
                                    .fontWeight(.semibold)
                                Spacer()
                                Text(session.completionStatus.replacingOccurrences(of: "_", with: " ").capitalized)
                                    .font(.caption)
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 4)
                                    .background(.quaternary)
                                    .clipShape(Capsule())
                            }

                            if let intention = session.sessionIntention, !intention.isEmpty {
                                Text(intention)
                                    .font(.subheadline)
                                    .foregroundStyle(.secondary)
                                    .italic()
                            }
                        }

                        // Entries
                        VStack(alignment: .leading, spacing: 12) {
                            Text("Items Practised")
                                .font(.headline)

                            ForEach(session.entries, id: \.id) { entry in
                                SessionEntryRow(entry: entry)
                            }
                        }

                        // Notes
                        if let notes = session.notes, !notes.isEmpty {
                            VStack(alignment: .leading, spacing: 4) {
                                Text("Notes")
                                    .font(.headline)
                                Text(notes)
                                    .font(.subheadline)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                    .padding()
                }
                .navigationTitle(formattedDate(session.startedAt))
                .navigationBarTitleDisplayMode(.inline)
            } else {
                ContentUnavailableView("Session not found", systemImage: "music.note")
            }
        }
    }

    private func formattedDate(_ isoString: String) -> String {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        var date = formatter.date(from: isoString)
        if date == nil {
            formatter.formatOptions = [.withInternetDateTime]
            date = formatter.date(from: isoString)
        }
        guard let d = date else { return isoString }
        let display = DateFormatter()
        display.dateStyle = .medium
        display.timeStyle = .short
        return display.string(from: d)
    }
}

// MARK: - Session Entry Row

struct SessionEntryRow: View {
    let entry: SetlistEntryView

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                TypeBadge(itemType: entry.itemType)
                Text(entry.itemTitle)
                    .font(.subheadline)
                    .fontWeight(.medium)
                Spacer()
                Text(entry.durationDisplay)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            HStack(spacing: 16) {
                if let score = entry.score {
                    HStack(spacing: 4) {
                        Text("Score")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                        ScoreIndicator(score: score)
                    }
                }

                if let tempo = entry.achievedTempo {
                    Label("\(tempo) BPM", systemImage: "metronome")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }

                Text(entry.status.replacingOccurrences(of: "_", with: " ").capitalized)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }

            if let notes = entry.notes, !notes.isEmpty {
                Text(notes)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .italic()
            }
        }
        .padding()
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 10))
    }
}
