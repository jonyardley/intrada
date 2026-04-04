import SwiftUI

/// State-driven router for the Practice tab.
///
/// Reads `sessionStatus` from the Crux ViewModel and renders the
/// matching screen. This makes impossible states impossible — e.g.
/// the session builder is unreachable when a session is active.
///
/// | Status   | View                          |
/// |----------|-------------------------------|
/// | idle     | SessionHistoryView            |
/// | building | SessionBuilderView            |
/// | active   | ActivePracticeView            |
/// | summary  | SessionSummaryView            |
struct PracticeTabRouter: View {
    @Environment(IntradaCore.self) private var core

    var body: some View {
        let viewModel = core.viewModel

        Group {
            switch viewModel.sessionStatus {
            case .idle:
                SessionHistoryView()
            case .building:
                SessionBuilderView()
                    .onAppear { resetLibraryQuery() }
            case .active:
                ActivePracticeView()
            case .summary:
                SessionSummaryView()
            }
        }
    }

    /// Reset the shared library query so the session builder starts unfiltered.
    private func resetLibraryQuery() {
        core.update(.setQuery(ListQuery(text: nil, itemType: nil, key: nil, tags: [])))
    }
}

#Preview {
    PracticeTabRouter()
        .environment(IntradaCore())
        .preferredColorScheme(.dark)
}
