import SharedTypes
import SwiftUI

/// Presents whichever of Journey B's three altitudes the core has running. A
/// dumb pipe like `DrillLoopHost`: taps in, events out, no decision about what
/// an altitude means. No click — none of the three has a gate or a ceiling.
struct PlayThroughHost: View {
  @Environment(Store.self) private var store

  private var runThrough: RunThroughView? { store.viewModel?.coach.runThrough }
  private var openPlay: OpenPlayView? { store.viewModel?.coach.openPlay }
  /// C2, at the one altitude that earns it: a gated run wrote a record, so the
  /// close may ask. Off-piste has already asked twice on the way out and
  /// unmonitored promised minutes and nothing else — the core decides which,
  /// and only ever sets this where the question is owed (decision 8).
  private var reflection: Bool { store.viewModel?.built.reflection ?? false }

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
          // Writes the record so the time is still the user's; no evidence (B1).
          onDiscard: { store.send(.coach(.discardRunThrough(now: SessionClock.nowRFC3339()))) },
          onFinish: close,
          onDismiss: close)
      } else if let openPlay {
        OpenPlayScreen(state: openPlay, onDone: close)
      } else if reflection {
        SessionReflectionHost()
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

  /// `RunThroughState.now` otherwise advances only when a section is judged —
  /// a clock that jumps a minute per tap. Free: `recovery_key` ignores the
  /// instant, so a second passing writes no crash-recovery blob.
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
