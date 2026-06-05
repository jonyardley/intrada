import SharedTypes
import SwiftUI

/// Full-screen host for the session player, presented by `RootView` while a
/// session is in progress. Switches on the core's phase — Active → the Focus
/// screen, Summary → review — and the core drives dismissal (Save/Discard → Idle).
struct PlayerHost: View {
  @Environment(Store.self) private var store

  var body: some View {
    if store.viewModel?.activeSession != nil {
      FocusPlayerScreen()
    } else if store.viewModel?.summary != nil {
      SessionSummaryScreen()
    }
  }
}
