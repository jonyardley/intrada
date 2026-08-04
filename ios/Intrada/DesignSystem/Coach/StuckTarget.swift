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

  @Environment(\.coachScale) private var scale
  @ScaledMetric(relativeTo: .subheadline) private var typeScale: CGFloat = 1

  private var height: CGFloat {
    let base = emphasis == .target ? scale.target : scale.quietTarget
    // Past ~1.25× the row stops being a target and starts being the screen.
    return base * min(max(typeScale, 1), 1.25)
  }

  var body: some View {
    Button(action: action) {
      Text(title)
        .font(IntradaFont.ambientStrong(emphasis == .target ? scale.targetLabel : scale.quietLabel))
        .foregroundStyle(emphasis == .target ? IntradaColor.ink : IntradaColor.inkSecondary)
        // Matches the Clean button on iPad: a stand-width target is further
        // from either hand, not nearer.
        .frame(maxWidth: scale == .compact ? .infinity : 430)
        .frame(height: height)
        .background(background)
    }
    .frame(maxWidth: .infinity)
    .buttonStyle(PressRebound(scale: 0.97))
    .accessibilityLabel(title)
    .accessibilityHint("Makes the next attempt easier")
  }

  @ViewBuilder private var background: some View {
    switch emphasis {
    case .target:
      ZStack {
        RoundedRectangle(cornerRadius: scale.targetRadius, style: .continuous)
          .fill(IntradaColor.cardFill)
        RoundedRectangle(cornerRadius: scale.targetRadius, style: .continuous)
          .strokeBorder(IntradaColor.slotOutline, lineWidth: 1)
      }
      // The 2pt seated edge: reads as a key, not a floating card.
      .background(
        RoundedRectangle(cornerRadius: scale.targetRadius, style: .continuous)
          .fill(IntradaColor.divider)
          .offset(y: 2))
    case .quiet:
      Color.clear
    }
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
