import SwiftUI

/// The tap-verdict pair (spec decision 18): the user hears their own attempt
/// and taps pass or fail against a countable criterion. The two are asymmetric
/// in width only — a miss is equally hittable, and calm.
struct TapVerdict: View {
  let onClean: () -> Void
  let onMissed: () -> Void

  @Environment(\.coachScale) private var scale
  @Environment(\.dynamicTypeSize) private var typeSize
  @ScaledMetric(relativeTo: .subheadline) private var typeScale: CGFloat = 1

  private var height: CGFloat { scale.target * min(max(typeScale, 1), 1.25) }
  /// Stacked at accessibility sizes, which is also where the fuller
  /// "No — missed it" fits.
  private var stacked: Bool { typeSize.isAccessibilitySize }

  private var gap: CGFloat { IntradaSpacing.controlGap + 2 }
  /// Clean is half again wider — asymmetric weight without shouting.
  private let cleanShare: CGFloat = 0.6
  /// Hand's reach, not stand's width (430 + 280 from the iPad comp).
  private var maxPairWidth: CGFloat { scale == .compact ? .infinity : 726 }

  var body: some View {
    if stacked {
      VStack(spacing: IntradaSpacing.controlGap) {
        clean
        missed(title: "No — missed it")
      }
    } else {
      // Explicit shares: `maxWidth: .infinity` splits 50/50, and squeezing one
      // side instead collapses its label to the glyph.
      GeometryReader { proxy in
        let usable = min(proxy.size.width, maxPairWidth) - gap
        HStack(spacing: gap) {
          clean.frame(width: usable * cleanShare)
          missed(title: scale == .compact ? "Missed it" : "No — missed it")
            .frame(width: usable * (1 - cleanShare))
        }
        .frame(maxWidth: .infinity)
      }
      .frame(height: height)
    }
  }

  private var clean: some View {
    button(
      title: "Yes — clean", icon: "checkmark", fg: IntradaColor.repCleanFg,
      bg: IntradaColor.repCleanBg, border: IntradaColor.repCleanBorder,
      size: scale.targetLabel, action: onClean
    )
    .accessibilityLabel("Yes, clean")
    .accessibilityHint("Banks this repetition against the gate")
  }

  private func missed(title: String) -> some View {
    button(
      title: title, icon: "xmark", fg: IntradaColor.repMissedFg,
      bg: IntradaColor.repMissedBg, border: IntradaColor.slotOutline,
      size: scale.targetLabel - 2, action: onMissed
    )
    .accessibilityLabel("No, missed it")
    .accessibilityHint("Logs the attempt without banking it")
  }

  private func button(
    title: String, icon: String, fg: Color, bg: Color, border: Color, size: CGFloat,
    action: @escaping () -> Void
  ) -> some View {
    Button(action: action) {
      HStack(spacing: IntradaSpacing.controlGap) {
        Image(systemName: icon)
          .font(.system(size: size + 3, weight: .semibold))
        Text(title)
          .lineLimit(1)
          .minimumScaleFactor(0.8)
      }
      .font(IntradaFont.ambientStrong(size))
      .foregroundStyle(fg)
      .frame(maxWidth: .infinity)
      .frame(height: height)
      .padding(.horizontal, IntradaSpacing.controlGap)
      .background(bg, in: RoundedRectangle(cornerRadius: scale.targetRadius, style: .continuous))
      .overlay(
        RoundedRectangle(cornerRadius: scale.targetRadius, style: .continuous)
          .strokeBorder(border, lineWidth: 1))
    }
    .buttonStyle(PressRebound(scale: 0.96))
  }
}

#if DEBUG
  #Preview {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: IntradaSpacing.section) {
        TapVerdict(onClean: {}, onMissed: {})
        TapVerdict(onClean: {}, onMissed: {})
          .environment(\.coachScale, .regular)
        TapVerdict(onClean: {}, onMissed: {})
          .dynamicTypeSize(.accessibility3)
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
