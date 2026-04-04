import SwiftUI

/// Full detail view for a past session from history.
///
/// Shows header stats and per-item results wrapped in cards.
/// Adapts layout for iPad with split view.
struct SessionDetailView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.horizontalSizeClass) private var sizeClass
    let session: PracticeSessionView

    var body: some View {
        Group {
            if sizeClass == .regular {
                iPadLayout
            } else {
                iPhoneLayout
            }
        }
        .background(Color.backgroundApp)
        .navigationTitle("Session Detail")
        .navigationBarTitleDisplayMode(.inline)
    }

    // MARK: - iPhone Layout

    private var iPhoneLayout: some View {
        ScrollView {
            VStack(spacing: Spacing.cardCompact) {
                headerCard

                CardView(padding: 0) {
                    entryList
                }
                .padding(.horizontal, Spacing.card)

                if let notes = session.notes, !notes.isEmpty {
                    CardView {
                        VStack(alignment: .leading, spacing: 6) {
                            Text("Session Notes")
                                .font(.system(size: 13, weight: .semibold))
                                .foregroundStyle(Color.textSecondary)
                            Text(notes)
                                .font(.system(size: 13))
                                .foregroundStyle(Color.textMuted)
                        }
                    }
                    .padding(.horizontal, Spacing.card)
                }
            }
            .padding(.bottom, Spacing.cardComfortable)
        }
    }

    // MARK: - iPad Layout

    private var iPadLayout: some View {
        HStack(spacing: 0) {
            // Left: header + notes
            ScrollView {
                VStack(spacing: Spacing.cardCompact) {
                    headerCard

                    if let notes = session.notes, !notes.isEmpty {
                        CardView {
                            VStack(alignment: .leading, spacing: 6) {
                                Text("Session Notes")
                                    .font(.system(size: 13, weight: .semibold))
                                    .foregroundStyle(Color.textSecondary)
                                Text(notes)
                                    .font(.system(size: 13))
                                    .foregroundStyle(Color.textMuted)
                            }
                        }
                        .padding(.horizontal, Spacing.card)
                    }
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

    // MARK: - Header Card

    private var headerCard: some View {
        CardView {
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
            }
            .frame(maxWidth: .infinity)
        }
        .padding(.horizontal, Spacing.card)
        .padding(.top, Spacing.card)
    }

    // MARK: - Entry List

    private var entryList: some View {
        LazyVStack(spacing: 0) {
            ForEach(Array(session.entries.enumerated()), id: \.element.id) { (index: Int, entry: SetlistEntryView) in
                if index > 0 {
                    Divider().background(Color.borderDefault)
                }

                SessionEntryResultRow(
                    entry: entry,
                    isEditable: false
                )
                .padding(.horizontal, Spacing.card)
            }
        }
    }

    // MARK: - Helpers

    private static let isoFormatter: ISO8601DateFormatter = {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return f
    }()

    private static let dateFormatter: DateFormatter = {
        let f = DateFormatter()
        f.dateStyle = .long
        f.timeStyle = .short
        return f
    }()

    private func formatDate(_ isoString: String) -> String {
        guard let date = Self.isoFormatter.date(from: isoString) else { return "" }
        return Self.dateFormatter.string(from: date)
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
