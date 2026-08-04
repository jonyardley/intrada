import SwiftUI

/// The one-glance acknowledgement of a tap-verdict — glyph plus at most one
/// fact. A miss is taupe, never red, and draws at the same size as a pass: it
/// is the user's own honest report, not a failure the app caught them in.
struct RepVerdict: View {
  enum Outcome {
    case clean
    case missed
  }

  let outcome: Outcome
  /// The one fact, in the musician's own words ("Gate open"). Never a score.
  var fact: String?

  @Environment(\.coachScale) private var scale

  var body: some View {
    VStack(spacing: IntradaSpacing.section) {
      disc
      if let fact {
        Text(fact)
          .font(IntradaFont.verdict(scale == .compact ? 38 : 58))
          .foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.center)
      }
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel([label, fact].compactMap { $0 }.joined(separator: ". "))
  }

  private var disc: some View {
    Circle()
      .fill(outcome == .clean ? IntradaColor.repCleanBg : IntradaColor.repMissedBg)
      .overlay(
        Circle().strokeBorder(
          outcome == .clean ? IntradaColor.repCleanBorder : IntradaColor.slotOutline,
          lineWidth: 1)
      )
      .overlay(
        Image(systemName: outcome == .clean ? "checkmark" : "xmark")
          .font(.system(size: scale.verdictDisc * 0.44, weight: .medium))
          .foregroundStyle(
            outcome == .clean ? IntradaColor.repCleanFg : IntradaColor.repMissedFg)
      )
      .frame(width: scale.verdictDisc, height: scale.verdictDisc)
      .popOnAppear()
  }

  private var label: String {
    outcome == .clean ? "Clean" : "Missed it"
  }
}

#if DEBUG
  #Preview {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: IntradaSpacing.stage) {
        RepVerdict(outcome: .clean)
        RepVerdict(outcome: .missed)
        RepVerdict(outcome: .clean, fact: "Gate open")
      }
    }
  }
#endif
