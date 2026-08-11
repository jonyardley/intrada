import SharedTypes
import SwiftUI

/// C1's three answers, in a musician's words rather than a 1–5 scale. The same
/// key shape as `TapVerdict`, at the same size, because both are "one tap and
/// the screen turns" — but only "It sang" takes the success tint: feel is not a
/// verdict, so two of the three stay neutral.
///
/// At most one of these per block, and never beside GateDots — the budget is
/// the core's (decision 17), and the surface only draws what it is offered.
struct FeelChips: View {
  var onFeel: (Feel) -> Void

  @Environment(\.coachScale) private var scale
  @Environment(\.dynamicTypeSize) private var typeSize
  @ScaledMetric(relativeTo: .subheadline) private var typeScale: CGFloat = 1

  private var height: CGFloat { scale.target * min(max(typeScale, 1), 1.25) }
  /// Three labels share the width two do in `TapVerdict`, so they start two
  /// points smaller; past accessibility sizes they stack and get it back.
  private var labelSize: CGFloat { scale.targetLabel - 2 }
  private var stacked: Bool { typeSize.isAccessibilitySize }

  var body: some View {
    if stacked {
      VStack(spacing: IntradaSpacing.controlGap) {
        chip(.foughtIt)
        chip(.gettingThere)
        chip(.itSang)
      }
    } else {
      HStack(spacing: IntradaSpacing.controlGap + 2) {
        chip(.foughtIt)
        chip(.gettingThere)
        chip(.itSang)
      }
    }
  }

  private func chip(_ feel: Feel) -> some View {
    Button {
      onFeel(feel)
    } label: {
      Text(Self.title(feel))
        .font(IntradaFont.ambientStrong(labelSize))
        .foregroundStyle(feel == .itSang ? IntradaColor.repCleanFg : IntradaColor.ink)
        .lineLimit(2)
        .minimumScaleFactor(0.75)
        .multilineTextAlignment(.center)
        .frame(maxWidth: .infinity)
        .frame(height: height)
        .padding(.horizontal, IntradaSpacing.controlGap)
        .background(
          feel == .itSang ? IntradaColor.repCleanBg : IntradaColor.cardFill,
          in: RoundedRectangle(cornerRadius: scale.targetRadius, style: .continuous)
        )
        .overlay(
          RoundedRectangle(cornerRadius: scale.targetRadius, style: .continuous)
            .strokeBorder(
              feel == .itSang ? IntradaColor.repCleanBorder : IntradaColor.slotOutline,
              lineWidth: 1))
    }
    .buttonStyle(PressRebound(scale: 0.96))
    .accessibilityLabel(Self.title(feel))
    .accessibilityHint("Kept with the session — it changes nothing about what comes next")
  }

  /// The words themselves, here rather than at the call site, so T13 governs
  /// them in one place.
  static func title(_ feel: Feel) -> String {
    switch feel {
    case .foughtIt: "Fought it"
    case .gettingThere: "Getting there"
    case .itSang: "It sang"
    }
  }
}

#if DEBUG
  #Preview {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: IntradaSpacing.section) {
        FeelChips(onFeel: { _ in })
        FeelChips(onFeel: { _ in })
          .environment(\.coachScale, .regular)
        FeelChips(onFeel: { _ in })
          .dynamicTypeSize(.accessibility3)
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
