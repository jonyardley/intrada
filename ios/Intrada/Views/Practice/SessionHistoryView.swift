import SwiftUI

/// Session history list — shows past sessions grouped by date.
///
/// Replaces `PracticeIdleView` on the Practice tab. When no sessions
/// exist, shows an empty state with "New Session" CTA.
struct SessionHistoryView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.horizontalSizeClass) private var sizeClass

    @State private var selectedSessionId: String? = nil
    @State private var deleteSessionId: String? = nil
    @State private var showDeleteConfirmation: Bool = false

    private var sessions: [PracticeSessionView] {
        core.viewModel.sessions
    }

    private var selectedSession: PracticeSessionView? {
        guard let id = selectedSessionId else { return nil }
        return sessions.first(where: { $0.id == id })
    }

    private var newSessionToolbar: some ToolbarContent {
        ToolbarItem(placement: .topBarTrailing) {
            Button {
                core.update(.session(.startBuilding))
            } label: {
                HStack(spacing: 4) {
                    Image(systemName: "plus")
                        .font(.system(size: 12, weight: .semibold))
                    Text("New Session")
                        .font(.system(size: 13, weight: .semibold))
                }
                .foregroundStyle(Color.textPrimary)
                .padding(.horizontal, Spacing.cardCompact)
                .padding(.vertical, 6)
                .background(Color.accent)
                .clipShape(RoundedRectangle(cornerRadius: DesignRadius.button))
            }
        }
    }

    var body: some View {
        Group {
            if sizeClass == .regular {
                iPadLayout
            } else {
                iPhoneLayout
            }
        }
        .confirmationDialog(
            "Delete this session?",
            isPresented: $showDeleteConfirmation,
            titleVisibility: .visible
        ) {
            Button("Delete", role: .destructive) {
                if let id = deleteSessionId {
                    core.update(.session(.deleteSession(id: id)))
                }
                deleteSessionId = nil
            }
            Button("Cancel", role: .cancel) {
                deleteSessionId = nil
            }
        } message: {
            Text("This cannot be undone.")
        }
    }

    // MARK: - iPhone Layout

    private var iPhoneLayout: some View {
        NavigationStack {
            Group {
                if sessions.isEmpty {
                    emptyState
                } else {
                    sessionList
                }
            }
            .background(Color.backgroundApp)
            .navigationTitle("Practice")
            .toolbar { newSessionToolbar }
            .navigationDestination(for: String.self) { sessionId in
                if let session = sessions.first(where: { $0.id == sessionId }) {
                    SessionDetailView(session: session)
                }
            }
        }
    }

    // MARK: - iPad Layout

    private var iPadLayout: some View {
        NavigationSplitView {
            Group {
                if sessions.isEmpty {
                    emptyState
                } else {
                    sessionList
                }
            }
            .background(Color.backgroundApp)
            .navigationTitle("Practice")
            .toolbar { newSessionToolbar }
        } detail: {
            if let session = selectedSession {
                SessionDetailView(session: session)
            } else {
                EmptyStateView(
                    icon: "hand.tap",
                    title: "Select a session",
                    message: "Tap a session to view its details"
                )
            }
        }
    }

    // MARK: - Session List

    private var sessionList: some View {
        List(selection: sizeClass == .regular ? $selectedSessionId : nil) {
            ForEach(groupedSessions, id: \.key) { (group: SessionGroup) in
                Section {
                    ForEach(group.sessions, id: \.id) { (session: PracticeSessionView) in
                        Group {
                            if sizeClass == .regular {
                                sessionCard(session)
                                    .tag(session.id)
                            } else {
                                NavigationLink(value: session.id) {
                                    sessionCard(session)
                                }
                            }
                        }
                        .listRowBackground(Color.surfaceSecondary)
                        .swipeActions(edge: .trailing) {
                            Button(role: .destructive) {
                                deleteSessionId = session.id
                                showDeleteConfirmation = true
                            } label: {
                                Label("Delete", systemImage: "trash")
                            }
                        }
                    }
                } header: {
                    Text(group.key)
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(Color.textMuted)
                }
            }
        }
        .listStyle(.plain)
        .scrollContentBackground(.hidden)
    }

    // MARK: - Session Card

    @ViewBuilder
    private func sessionCard(_ session: PracticeSessionView) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            // Top row: duration, items, status, time
            HStack(spacing: 8) {
                Text(session.totalDurationDisplay)
                    .font(.system(size: 16, weight: .bold))
                    .foregroundStyle(Color.textPrimary)

                Text("·")
                    .foregroundStyle(Color.textFaint)

                Text("\(session.entries.count) items")
                    .font(.system(size: 13))
                    .foregroundStyle(Color.textSecondary)

                if session.completionStatus == .endedEarly {
                    Text("Ended Early")
                        .font(.system(size: 10, weight: .semibold))
                        .foregroundStyle(Color.warmAccentText)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(Color.warmAccentSurface)
                        .clipShape(RoundedRectangle(cornerRadius: DesignRadius.badge))
                }

                Spacer()

                Text(formatTime(session.startedAt))
                    .font(.system(size: 12))
                    .foregroundStyle(Color.textFaint)
            }

            // Intention
            if let intention = session.sessionIntention, !intention.isEmpty {
                Text(intention)
                    .font(.system(size: 12))
                    .foregroundStyle(Color.textMuted)
                    .italic()
                    .lineLimit(1)
            }

            // Item names
            let itemNames = session.entries.prefix(3).map(\.itemTitle).joined(separator: " · ")
            if !itemNames.isEmpty {
                Text(itemNames)
                    .font(.system(size: 11))
                    .foregroundStyle(Color.textFaint)
                    .lineLimit(1)
            }
        }
        .padding(.vertical, 4)
    }

    // MARK: - Empty State

    private var emptyState: some View {
        EmptyStateView(
            icon: "play.circle",
            title: "No sessions yet",
            message: "Start a session to track your practice",
            actionTitle: "New Session"
        ) {
            core.update(.session(.startBuilding))
        }
    }

    // MARK: - Grouping

    private struct SessionGroup: Identifiable {
        let key: String
        let sessions: [PracticeSessionView]
        var id: String { key }
    }

    private static let isoFormatter: ISO8601DateFormatter = {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return f
    }()

    private static let dateFormatter: DateFormatter = {
        let f = DateFormatter()
        f.dateStyle = .medium
        return f
    }()

    private static let timeFormatter: DateFormatter = {
        let f = DateFormatter()
        f.timeStyle = .short
        return f
    }()

    private var groupedSessions: [SessionGroup] {
        let calendar = Calendar.current
        let today = calendar.startOfDay(for: Date())

        var groups: [String: [PracticeSessionView]] = [:]
        var groupOrder: [String] = []

        for session in sessions {
            let date = Self.isoFormatter.date(from: session.startedAt) ?? Date()
            let dayStart = calendar.startOfDay(for: date)

            let label: String
            if dayStart == today {
                label = "Today"
            } else if dayStart == calendar.date(byAdding: .day, value: -1, to: today) {
                label = "Yesterday"
            } else {
                label = Self.dateFormatter.string(from: date)
            }

            if groups[label] == nil {
                groupOrder.append(label)
            }
            groups[label, default: []].append(session)
        }

        return groupOrder.map { key in
            SessionGroup(key: key, sessions: groups[key] ?? [])
        }
    }

    private func formatTime(_ isoString: String) -> String {
        guard let date = Self.isoFormatter.date(from: isoString) else { return "" }
        return Self.timeFormatter.string(from: date)
    }
}

#Preview("SessionHistoryView") {
    SessionHistoryView()
        .environment(IntradaCore())
        .preferredColorScheme(.dark)
}
