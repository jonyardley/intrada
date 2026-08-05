import SwiftUI

/// The Practice pillar's one primary action: start today's planned session and
/// land in the drill loop. Every line of copy arrives already written by the
/// core's plan; this only lays it out.
struct PressStartHero: View {
  let headline: String
  var why: String?
  let footnote: String
  var onStart: () -> Void

  var body: some View {
    VStack(spacing: IntradaSpacing.cardCompact) {
      Eyebrow("Today", tint: IntradaColor.onAccent.opacity(0.7))

      Text(headline)
        .font(IntradaFont.pageTitle(25))
        .foregroundStyle(IntradaColor.paperTop)
        .multilineTextAlignment(.center)
        .fixedSize(horizontal: false, vertical: true)

      if let why {
        Label(why, systemImage: "arrow.turn.down.right")
          .font(IntradaFont.ambient())
          .foregroundStyle(IntradaColor.onAccent.opacity(0.85))
          .multilineTextAlignment(.center)
          .fixedSize(horizontal: false, vertical: true)
          .accessibilityLabel(spoken(why))
      }

      Button(action: onStart) {
        Image(systemName: "play.fill")
          .font(.system(size: 38))
          .foregroundStyle(IntradaColor.accent)
          .frame(width: 96, height: 96)
          .background(IntradaColor.playerBgTop)
          .clipShape(Circle())
          .shadow(color: .black.opacity(0.25), radius: 16, y: 8)
      }
      .buttonStyle(PressRebound())
      .accessibilityLabel("Start practising")
      .padding(.vertical, IntradaSpacing.controlGap)

      Text(footnote)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.onAccent.opacity(0.85))
        .multilineTextAlignment(.center)
        .accessibilityLabel(spoken(footnote))
    }
    .frame(maxWidth: .infinity)
    .padding(IntradaSpacing.section)
    .background(LinearGradient.practiceHero)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.hero))
    .shadow(color: .black.opacity(0.18), radius: 20, y: 10)
  }

  /// Most voices read "·" aloud as "middle dot" (same treatment as DrillScreen).
  private func spoken(_ line: String) -> String {
    line.replacingOccurrences(of: " · ", with: ", ")
  }
}

#if DEBUG
  #Preview("Planned") {
    ZStack {
      PaperBackground()
      PressStartHero(
        headline: "Rootless voicings",
        why: "Strasbourg / St. Denis",
        footnote: "5 blocks · about 30 minutes",
        onStart: {}
      )
      .padding(IntradaSpacing.card)
    }
  }

  #Preview("No plan yet") {
    ZStack {
      PaperBackground()
      PressStartHero(
        headline: "A focused session",
        footnote: "Tap to begin — one decision",
        onStart: {}
      )
      .padding(IntradaSpacing.card)
    }
  }
#endif
