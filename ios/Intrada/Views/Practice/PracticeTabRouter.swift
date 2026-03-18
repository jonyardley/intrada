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
/// | active   | Active session (placeholder)  |
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
            case .active:
                PracticePlaceholderView(
                    title: "Active Session",
                    icon: "play.circle.fill",
                    message: "Active session view coming in #197"
                )
            case .summary:
                PracticePlaceholderView(
                    title: "Session Summary",
                    icon: "checkmark.circle.fill",
                    message: "Session summary view coming in #198"
                )
            }
        }
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

// MARK: - Generic Placeholder

/// Reusable placeholder for tabs not yet implemented.
private struct PracticePlaceholderView: View {
    let title: String
    let icon: String
    let message: String

    var body: some View {
        NavigationStack {
            VStack(spacing: 16) {
                Image(systemName: icon)
                    .font(.system(size: 48))
                    .foregroundStyle(Color.textFaint)

                Text(title)
                    .font(.title2)
                    .fontWeight(.semibold)
                    .foregroundStyle(Color.textSecondary)

                Text(message)
                    .font(.subheadline)
                    .foregroundStyle(Color.textFaint)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Color.backgroundApp)
            .navigationTitle(title)
        }
    }
}

#Preview {
    PracticeTabRouter()
        .environment(IntradaCore())
        .preferredColorScheme(.dark)
}
