import SwiftUI

/// State-driven router for the Practice tab.
///
/// Reads `sessionStatus` from the Crux ViewModel and renders the
/// matching screen. This makes impossible states impossible — e.g.
/// the session builder is unreachable when a session is active.
///
/// | Status   | View                          |
/// |----------|-------------------------------|
/// | idle     | Session list (placeholder)    |
/// | building | SessionBuilderView            |
/// | active   | ActivePracticeView            |
/// | summary  | Session summary (placeholder) |
struct PracticeTabRouter: View {
    @Environment(IntradaCore.self) private var core

    var body: some View {
        let viewModel = core.viewModel

        Group {
            switch viewModel.sessionStatus {
            case .idle:
                PracticeIdleView()
            case .building:
                SessionBuilderView()
                    .onAppear { resetLibraryQuery() }
            case .active:
                ActivePracticeView()
            case .summary:
                SummaryPlaceholderView()
            }
        }
    }

    /// Reset the shared library query so the session builder starts unfiltered.
    private func resetLibraryQuery() {
        core.update(.setQuery(ListQuery(text: nil, itemType: nil, key: nil, tags: [])))
    }
}

// MARK: - Idle View (Session List Placeholder)

/// Placeholder for the session list / history view.
/// Shows a "New Session" CTA to enter the builder.
private struct PracticeIdleView: View {
    @Environment(IntradaCore.self) private var core

    var body: some View {
        NavigationStack {
            VStack(spacing: 24) {
                Spacer()

                Image(systemName: "play.circle")
                    .font(.system(size: 56))
                    .foregroundStyle(Color.textFaint)

                Text("Practice")
                    .font(.title2)
                    .fontWeight(.semibold)
                    .foregroundStyle(Color.textSecondary)

                Text("Start a session to track your practice")
                    .font(.subheadline)
                    .foregroundStyle(Color.textMuted)

                Button {
                    core.update(.session(.startBuilding))
                } label: {
                    HStack(spacing: 8) {
                        Image(systemName: "plus")
                        Text("New Session")
                    }
                    .font(.body.weight(.semibold))
                    .foregroundStyle(Color.textPrimary)
                    .frame(maxWidth: 200)
                    .frame(height: 44)
                    .background(Color.accent)
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                }

                Spacer()
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Color.backgroundApp)
            .navigationTitle("Practice")
        }
    }
}

// MARK: - Summary Placeholder (temporary — replaced by #198)

/// Temporary placeholder with a "Done" button to return to idle.
private struct SummaryPlaceholderView: View {
    @Environment(IntradaCore.self) private var core

    var body: some View {
        NavigationStack {
            VStack(spacing: 24) {
                Spacer()

                Image(systemName: "checkmark.circle.fill")
                    .font(.system(size: 48))
                    .foregroundStyle(Color.successText)

                Text("Session Complete!")
                    .font(.title2)
                    .fontWeight(.semibold)
                    .foregroundStyle(Color.textSecondary)

                Text("Session summary view coming in #198")
                    .font(.subheadline)
                    .foregroundStyle(Color.textFaint)

                Spacer()

                Button {
                    core.update(.session(.discardSession))
                } label: {
                    Text("Done")
                        .font(.body.weight(.semibold))
                        .foregroundStyle(Color.textPrimary)
                        .frame(maxWidth: .infinity)
                        .frame(height: 44)
                        .background(Color.accent)
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                }
                .padding(.horizontal, 32)
                .padding(.bottom, 40)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Color.backgroundApp)
            .navigationTitle("Session Summary")
        }
    }
}

#Preview {
    PracticeTabRouter()
        .environment(IntradaCore())
        .preferredColorScheme(.dark)
}
