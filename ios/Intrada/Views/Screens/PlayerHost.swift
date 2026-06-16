import SharedTypes
import SwiftUI

/// Full-screen host for the session player, presented by `RootView` while a
/// session is in progress. Switches on the core's phase — Active → the Focus
/// screen, Summary → review — and the core drives dismissal (Save/Discard → Idle).
struct PlayerHost: View {
  @Environment(Store.self) private var store

  var body: some View {
    Group {
      if store.viewModel?.activeSession != nil {
        FocusPlayerScreen()
      } else if store.viewModel?.summary != nil {
        SessionSummaryScreen()
      }
    }
    // The player is a fullScreenCover, so RootView's banner is occluded while
    // it's up. Re-surface viewModel.error here — otherwise an error raised mid
    // session (a swallowed score tap, a failed save) is a silent no-op (#846).
    .safeAreaInset(edge: .top, spacing: 0) {
      if let error = store.viewModel?.error {
        GlobalBanner(message: error) { store.send(.clearError) }
      }
    }
  }
}
