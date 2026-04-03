import SwiftUI

/// Full detail view for a past session from history.
///
/// Shows header stats and per-item results. Currently read-only;
/// US3 adds editing capability.
struct SessionDetailView: View {
    @Environment(IntradaCore.self) private var core
    let session: PracticeSessionView

    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                // Header
                VStack(spacing: 8) {
                    HStack(spacing: 8) {
                        Text(session.totalDurationDisplay)
                            .font(.system(size: 20, weight: .bold))
                            .foregroundStyle(Color.textPrimary)

                        Text("·")
                            .foregroundStyle(Color.textFaint)

                        Text("\(session.entries.count) items")
                            .font(.system(size: 15))
                            .foregroundStyle(Color.textSecondary)

                        if session.completionStatus == .endedEarly {
                            Text("Ended Early")
                                .font(.system(size: 11, weight: .semibold))
                                .foregroundStyle(Color.warmAccentText)
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(Color.warmAccentSurface)
                                .clipShape(RoundedRectangle(cornerRadius: DesignRadius.badge))
                        }
                    }

                    Text(formatDate(session.startedAt))
                        .font(.system(size: 13))
                        .foregroundStyle(Color.textMuted)

                    if let intention = session.sessionIntention, !intention.isEmpty {
                        Text(intention)
                            .font(.system(size: 13))
                            .foregroundStyle(Color.textMuted)
                            .italic()
                    }

                    if let notes = session.notes, !notes.isEmpty {
                        Text(notes)
                            .font(.system(size: 13))
                            .foregroundStyle(Color.textSecondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(12)
                            .background(Color.surfaceSecondary)
                            .clipShape(RoundedRectangle(cornerRadius: DesignRadius.card))
                    }
                }
                .frame(maxWidth: .infinity)
                .padding(.horizontal, 24)
                .padding(.vertical, 16)

                Divider().background(Color.borderDefault)

                // Entries
                LazyVStack(spacing: 0) {
                    ForEach(Array(session.entries.enumerated()), id: \.element.id) { (index: Int, entry: SetlistEntryView) in
                        if index > 0 {
                            Divider().background(Color.borderDefault)
                        }

                        SessionEntryResultRow(
                            entry: entry,
                            isEditable: false
                        )
                        .padding(.horizontal, 24)
                    }
                }
            }
        }
        .background(Color.backgroundApp)
        .navigationTitle("Session Detail")
        .navigationBarTitleDisplayMode(.inline)
    }

    private func formatDate(_ isoString: String) -> String {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        guard let date = formatter.date(from: isoString) else { return "" }
        let df = DateFormatter()
        df.dateStyle = .long
        df.timeStyle = .short
        return df.string(from: date)
    }
}

#Preview("SessionDetailView") {
    NavigationStack {
        SessionDetailView(
            session: PracticeSessionView(
                id: "s1",
                startedAt: "2026-04-03T15:00:00.000Z",
                finishedAt: "2026-04-03T15:23:00.000Z",
                totalDurationDisplay: "23 min",
                completionStatus: .completed,
                notes: "Good session overall",
                entries: [],
                sessionIntention: "Focus on dynamics"
            )
        )
    }
    .environment(IntradaCore())
    .preferredColorScheme(.dark)
}
