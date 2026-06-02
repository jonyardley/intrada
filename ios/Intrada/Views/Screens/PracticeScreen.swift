import SharedTypes
import SwiftUI

struct PracticeScreen: View {
  @Environment(Store.self) private var store

  private var sessions: [PracticeSessionView] { store.viewModel?.sessions ?? [] }

  var body: some View {
    ScreenScaffold(title: "Practice", subtitle: subtitle) {
      VStack(spacing: 0) {
        startButton
          .padding(.horizontal, 16)
          .padding(.top, 16)
        content
      }
    }
  }

  // The front door. The builder/player don't exist yet, so it's present-but-
  // disabled to establish the one-primary-action hierarchy without a dead-end.
  private var startButton: some View {
    VStack(spacing: 6) {
      Label("Start practising", systemImage: "play.fill")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.onAccent)
        .frame(maxWidth: .infinity)
        .padding(.vertical, 14)
        .background(LinearGradient.brandBar)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .opacity(0.5)
      Text("Coming soon")
        .font(IntradaFont.micro)
        .foregroundStyle(IntradaColor.inkFaint)
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Start practising, coming soon")
  }

  @ViewBuilder private var content: some View {
    if sessions.isEmpty {
      PlaceholderContent(
        systemImage: "metronome.fill",
        message: "Your practice sessions will appear here.")
    } else {
      ScrollView {
        LazyVStack(spacing: 14) {
          ForEach(sessions, id: \.id) { session in
            SessionCard(session: session)
          }
        }
        .padding(.horizontal, 16)
        .padding(.top, 16)
        .padding(.bottom, 16)
      }
      .scrollEdgeShadow()
    }
  }

  private var subtitle: String {
    let count = sessions.count
    return count == 0 ? "No sessions yet" : "\(count) session\(count == 1 ? "" : "s")"
  }
}

#if DEBUG
  #Preview("Populated") {
    PracticeScreen()
      .environment(Store.previewSeeded)
  }

  #Preview("Empty") {
    PracticeScreen()
      .environment(Store.preview)
  }
#endif
