import SwiftUI

/// "I'm stuck" — the one large target on the drill screen. A surface, not a
/// gradient: it must be findable without looking down, not attractive.
struct StuckTarget: View {
  enum Emphasis {
    /// A2 — the sole resident control, at the loop's shared target height.
    case target
    /// A3 — same place, demoted beneath a rule; the tap-verdict is the ask.
    case quiet
  }

  var title: String = "I'm stuck"
  var emphasis: Emphasis = .target
  let action: () -> Void

  var body: some View {
    CoachAction(
      title: title, emphasis: emphasis == .target ? .key : .quiet,
      hint: "Makes the next attempt easier", action: action)
  }
}

#if DEBUG
  #Preview {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: IntradaSpacing.section) {
        StuckTarget(action: {})
        StuckTarget(emphasis: .quiet, action: {})
        HairlineDivider()
        StuckTarget(action: {})
          .environment(\.coachScale, .regular)
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
