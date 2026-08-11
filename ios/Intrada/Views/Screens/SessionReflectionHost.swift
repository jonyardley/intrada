import SharedTypes
import SwiftUI

/// C2's question, wired to the core. Both places a session can end — the drill
/// loop and a gated run-through — reach it through here, so "keep it" and "not
/// tonight" mean the same thing whichever door the session came in by.
struct SessionReflectionHost: View {
  @Environment(Store.self) private var store

  var body: some View {
    SessionReflectionScreen(onKeep: keep(_:), onDismiss: dismiss)
  }

  /// `audioPath` and `durationS` stay nil until the capture effect lands
  /// (#1309): the words are kept as text, and a path to a file nobody recorded
  /// would be a row pointing at nothing. `sessionRef` waits on the coach
  /// session's id reaching the view (#1314).
  private func keep(_ text: String) {
    store.send(
      .builtSession(
        .recordReflection(
          kind: .sessionClose, sessionRef: nil, transcript: text.isEmpty ? nil : text,
          audioPath: nil, durationS: nil)),
      onSuccess: .impact)
  }

  private func dismiss() {
    store.send(.builtSession(.dismissReflection), onSuccess: .selection)
  }
}
