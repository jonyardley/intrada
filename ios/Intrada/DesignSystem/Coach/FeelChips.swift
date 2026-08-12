import SharedTypes
import SwiftUI

/// C1's three answers. Only "It sang" takes the success tint: feel is not a
/// verdict, so two of the three stay neutral (decision 17).
struct FeelChips: View {
  var onFeel: (Feel) -> Void

  @Environment(\.coachScale) private var scale
  @Environment(\.dynamicTypeSize) private var typeSize
  @ScaledMetric(relativeTo: .subheadline) private var typeScale: CGFloat = 1

  private var height: CGFloat { scale.target * min(max(typeScale, 1), 1.25) }
  /// Three labels share the width `TapVerdict`'s two do, so they start smaller.
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
        .coachKeySurface(
          height: height,
          fill: feel == .itSang ? IntradaColor.repCleanBg : IntradaColor.cardFill,
          border: feel == .itSang ? IntradaColor.repCleanBorder : IntradaColor.slotOutline)
    }
    .buttonStyle(PressRebound(scale: 0.96))
    .accessibilityLabel(Self.title(feel))
    .accessibilityHint("Kept with the session, and never scored against a gate")
  }

  /// Here rather than at the call site, so T13 governs them in one place.
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
