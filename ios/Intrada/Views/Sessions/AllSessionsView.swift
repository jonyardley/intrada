import SwiftUI

/// Chronological list of all completed sessions.
struct AllSessionsView: View {
    @Environment(IntradaCore.self) private var core

    private var sessions: [PracticeSessionView] {
        core.viewModel.sessions.sorted { $0.startedAt > $1.startedAt }
    }

    var body: some View {
        ScrollView {
            if sessions.isEmpty {
                EmptyStateView(
                    icon: "music.note.list",
                    title: "No sessions yet",
                    message: "Start practising to see your history here."
                )
            } else {
                LazyVStack(spacing: 12) {
                    ForEach(sessions, id: \.id) { session in
                        NavigationLink(value: session.id) {
                            SessionCardView(session: session)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding()

                Text("\(sessions.count) session\(sessions.count == 1 ? "" : "s")")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
                    .padding(.bottom)
            }
        }
        .navigationTitle("All Sessions")
        .navigationBarTitleDisplayMode(.inline)
        .navigationDestination(for: String.self) { sessionId in
            SessionDetailView(sessionId: sessionId)
        }
    }
}
