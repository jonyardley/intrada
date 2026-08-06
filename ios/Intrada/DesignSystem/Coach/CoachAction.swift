import SwiftUI

/// A labelled key in the drill loop: a surface you can find without looking
/// down, not a gradient that asks to be admired. `StuckTarget` and the card's
/// Start / Skip are all this one shape at three weights.
struct CoachAction: View {
  enum Emphasis {
    /// The one thing to do next.
    case primary
    /// A resident control on paper.
    case key
    /// Demoted beneath a rule; the ask is elsewhere.
    case quiet
  }

  let title: String
  var emphasis: Emphasis = .key
  var hint: String?
  let action: () -> Void

  @Environment(\.coachScale) private var scale
  @ScaledMetric(relativeTo: .subheadline) private var typeScale: CGFloat = 1

  private var height: CGFloat {
    let base = emphasis == .quiet ? scale.quietTarget : scale.target
    // Past ~1.25× the row stops being a target and starts being the screen.
    return base * min(max(typeScale, 1), 1.25)
  }

  private var labelSize: CGFloat {
    emphasis == .quiet ? scale.quietLabel : scale.targetLabel
  }

  private var foreground: Color {
    switch emphasis {
    case .primary: IntradaColor.onAccent
    case .key: IntradaColor.ink
    case .quiet: IntradaColor.inkSecondary
    }
  }

  var body: some View {
    Button(action: action) {
      Text(title)
        .font(IntradaFont.ambientStrong(labelSize))
        .foregroundStyle(foreground)
        .lineLimit(1)
        .minimumScaleFactor(0.8)
        // Matches the Clean button on iPad: a stand-width target is further
        // from either hand, not nearer.
        .frame(maxWidth: scale == .compact ? .infinity : 430)
        .frame(height: height)
        .background(background)
    }
    .frame(maxWidth: .infinity)
    .buttonStyle(PressRebound(scale: 0.97))
    .accessibilityLabel(title)
    .accessibilityHint(hint ?? "")
  }

  @ViewBuilder private var background: some View {
    switch emphasis {
    case .primary:
      seated(fill: IntradaColor.accent, edge: IntradaColor.heroGradientBottom, border: nil)
    case .key:
      seated(
        fill: IntradaColor.cardFill, edge: IntradaColor.divider,
        border: IntradaColor.slotOutline)
    case .quiet:
      Color.clear
    }
  }

  /// The 2pt seated edge: reads as a key, not a floating card.
  private func seated(fill: Color, edge: Color, border: Color?) -> some View {
    let shape = RoundedRectangle(cornerRadius: scale.targetRadius, style: .continuous)
    return ZStack {
      shape.fill(fill)
      if let border { shape.strokeBorder(border, lineWidth: 1) }
    }
    .background(shape.fill(edge).offset(y: 2))
  }
}

#if DEBUG
  #Preview {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: IntradaSpacing.section) {
        CoachAction(title: "Start", emphasis: .primary, action: {})
        CoachAction(title: "I'm stuck", action: {})
        CoachAction(title: "Skip this block", emphasis: .quiet, action: {})
        HairlineDivider()
        CoachAction(title: "Start", emphasis: .primary, action: {})
          .environment(\.coachScale, .regular)
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
