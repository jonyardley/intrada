import SharedTypes
import SwiftUI

/// Presents whichever of Journey B's three altitudes the core has running, and
/// nothing else. A dumb pipe like `DrillLoopHost`: taps in, events out, and no
/// decision about what an altitude means.
///
/// No click: none of the three has a gate, a ladder or a ceiling. The seconds
/// are still reported, because a run-through's elapsed time lives on the engine
/// (`RunThroughState.now`) and otherwise advances only when a section is judged
/// — a clock that jumps a minute per tap. The two lower altitudes have nothing
/// for a tick to move and draw their own clocks from `startedAt`.
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
    .task { await reportSeconds() }
  }

  /// A tick moves the run-through's clock and nothing else: `recovery_key`
  /// ignores the instant, so a second passing writes no crash-recovery blob.
  private func reportSeconds() async {
    while !Task.isCancelled {
      try? await Task.sleep(for: .seconds(1))
      guard !Task.isCancelled else { return }
      store.send(.coach(.tick(now: SessionClock.nowRFC3339())))
    }
  }

  /// `closeSession`, never `leaveSession`: leaving is only defined for a
  /// planned session's block, and at the two lower altitudes it would set the
  /// state to closing without writing the minutes the run banked (#1285).
  private func close() {
    store.send(.coach(.closeSession(now: SessionClock.nowRFC3339())))
  }
}
