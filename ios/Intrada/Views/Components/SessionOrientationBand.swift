import SharedTypes
import SwiftUI

/// The Focus player's orientation strip: how long, how far, which item, and the
/// way out (design-principles T19). Store-free so it can be exercised at the
/// sizes that break it; the screen supplies its own options menu.
struct SessionOrientationBand<Menu: View>: View {
  let sessionElapsed: Int?
  let positionLabel: String
  let types: [ItemKind]
  let filled: Int
  @ViewBuilder let menu: () -> Menu

  /// Equal side *minimums* keep the position label centred. A minimum rather
  /// than a fixed width because `H:MM:SS` past an hour, and `.caption` under
  /// Dynamic Type, both outgrow it, and an off-centre label beats a clipped clock.
  static var slot: CGFloat { 56 }

  var body: some View {
    VStack(spacing: 12) {
      HStack(spacing: IntradaSpacing.controlGap) {
        elapsed
          .frame(minWidth: Self.slot, alignment: .leading)
        Text(positionLabel)
          .font(IntradaFont.badge)
          .tracking(1.5)
          .foregroundStyle(IntradaColor.inkFaint)
          // Scales rather than truncating: "FOCUS · 3 OF 5" losing its tail to an
          // ellipsis costs the item number, which is the half that carries the fact.
          .lineLimit(1)
          .minimumScaleFactor(0.5)
          .frame(maxWidth: .infinity)
        menu()
          .frame(minWidth: Self.slot, alignment: .trailing)
      }
      SegmentedProgress(types: types, filled: filled)
    }
  }

  @ViewBuilder private var elapsed: some View {
    if let sessionElapsed {
      Text(SessionClock.clockDisplay(sessionElapsed))
        .font(IntradaFont.metaMedium)
        .monospacedDigit()
        .lineLimit(1)
        .minimumScaleFactor(0.7)
        .foregroundStyle(IntradaColor.inkSecondary)
        .accessibilityLabel("Session so far")
        .accessibilityValue(SessionClock.clockDisplay(sessionElapsed))
    }
  }
}
