import SharedTypes
import SwiftUI

/// C2's question, wired to the core. Both doors a session can end by reach it
/// through here, so the two answers mean the same thing either way.
struct SessionReflectionHost: View {
  @Environment(Store.self) private var store

  var body: some View {
    SessionReflectionScreen(onKeep: keep(_:), onDismiss: dismiss)
  }

  /// `audioPath` and `durationS` wait on the capture effect (#1309), and
  /// `sessionRef` on the coach session's id reaching the view (#1314).
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
