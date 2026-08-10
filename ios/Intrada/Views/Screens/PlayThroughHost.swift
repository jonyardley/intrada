import SharedTypes
import SwiftUI

/// Presents whichever of Journey B's three altitudes the core has running, and
/// nothing else. A dumb pipe like `DrillLoopHost`: taps in, events out, and no
/// decision about what an altitude means.
///
/// There is no click and no tick here — none of the three has a gate, a ladder
/// or a ceiling, so the core has nothing to decide per second and the screens
/// draw their own clocks from `startedAt`.
struct PlayThroughHost: View {
  @Environment(Store.self) private var store

  private var runThrough: RunThroughView? { store.viewModel?.coach.runThrough }
  private var openPlay: OpenPlayView? { store.viewModel?.coach.openPlay }

  var body: some View {
    Group {
      if let runThrough {
        RunThroughScreen(
          state: runThrough,
          onVerdict: { held in
            store.send(
              .coach(.judgeSection(held: held, now: SessionClock.nowRFC3339())),
              onSuccess: held ? .impact : .selection)
          },
          // Writes the record so the time is still the user's, and lands no
          // evidence (B1).
          onDiscard: { store.send(.coach(.discardRunThrough(now: SessionClock.nowRFC3339()))) },
          onFinish: close,
          onDismiss: close)
      } else if let openPlay {
        OpenPlayScreen(state: openPlay, onDone: close)
      } else {
        Color.clear
      }
    }
    // The cover occludes RootView's banner, so a refusal surfaces here or
    // nowhere (#846, and DrillLoopHost's own reason for this inset).
    .safeAreaInset(edge: .top, spacing: 0) {
      if let error = store.viewModel?.error {
        GlobalBanner(message: error) { store.send(.clearError) }
      }
    }
  }

  /// `closeSession`, never `leaveSession`: leaving is only defined for a
  /// planned session's block, and at the two lower altitudes it would set the
  /// state to closing without writing the minutes the run banked (#1285).
  private func close() {
    store.send(.coach(.closeSession(now: SessionClock.nowRFC3339())))
  }
}
