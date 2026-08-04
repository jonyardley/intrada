import SwiftUI

/// "I'm stuck" — the one large target on the drill screen, hittable without
/// looking down. A surface, not a gradient: it must be findable, not
/// attractive. It grows with Dynamic Type rather than staying fixed, because at
/// the largest sizes the user is likelier to need it.
struct StuckTarget: View {
  enum Emphasis {
    /// A2: the sole resident control, full-width at the shared 66pt height.
    case target
    /// A3: still present, same place, demoted beneath a rule — the tap-verdict
    /// is the primary ask between reps.
    case quiet
  }

  var title: String = "I'm stuck"
  var emphasis: Emphasis = .target
  let action: () -> Void

  @Environment(\.coachScale) private var scale
  @ScaledMetric(relativeTo: .subheadline) private var typeScale: CGFloat = 1

  private var height: CGFloat {
    let base = emphasis == .target ? scale.target : 46
    // Cap the growth: past ~1.25× the row stops being a target and starts
    // being the screen.
    return base * min(max(typeScale, 1), 1.25)
  }

  var body: some View {
    Button(action: action) {
      Text(title)
        .font(IntradaFont.ambient(emphasis == .target ? scale.targetLabel : 14).weight(.semibold))
        .foregroundStyle(emphasis == .target ? IntradaColor.ink : IntradaColor.repMissedFg)
        // On iPad it matches the Clean button's width rather than spanning the
        // stand: a target the width of the screen is further from either hand,
        // not nearer.
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
      // The 2pt seated edge from the mock — the target reads as a physical key
      // rather than a floating card.
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
