#if DEBUG
  import SwiftUI

  /// Debug-only way in to the drill loop, so it can be run at a piano before
  /// press-start reaches it from Practice (#1182). Deliberately plain — no
  /// scaffold, no tokens; it is a door, not a screen, and it goes when the real
  /// entry lands.
  struct DrillLoopEntry: View {
    @State private var running = false

    var body: some View {
      VStack(spacing: IntradaSpacing.cardCompact) {
        Text("A2 / A3 full-screen, driven by the core's session state machine.")
          .font(IntradaFont.ambient())
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
        Button("Run the drill loop") { running = true }
          .buttonStyle(.borderedProminent)
      }
      .padding(IntradaSpacing.card)
      .fullScreenCover(isPresented: $running) {
        DrillLoopHost(onClose: { running = false })
      }
    }
  }
#endif
