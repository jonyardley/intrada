import SwiftUI

/// The core's offer to take time away off the item (#2306). Ignoring it keeps the time.
struct AwayOfferBanner: View {
  let label: String
  let onLeaveOut: () -> Void
  let onKeep: () -> Void

  var body: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      Image(systemName: "clock")
        .iconSize(.inline)
        .foregroundStyle(IntradaColor.inkSecondary)
        .accessibilityHidden(true)
      Text(label)
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.ink)
        .frame(maxWidth: .infinity, alignment: .leading)
      Button("Leave it out", action: onLeaveOut)
        .font(IntradaFont.button)
        .foregroundStyle(IntradaColor.ink)
        .frame(minHeight: 44)
        .accessibilityIdentifier("player.leaveAwayOut")
      Button(action: onKeep) {
        Image(systemName: "xmark")
          .iconSize(.inline)
          .foregroundStyle(IntradaColor.inkSecondary)
          .frame(width: 40, height: 44)
      }
      .accessibilityLabel("Keep the time")
      .accessibilityIdentifier("player.keepAway")
    }
    .padding(.leading, IntradaSpacing.cardCompact)
    .padding(.vertical, IntradaSpacing.controlGap)
    .background(IntradaColor.cardFill, in: RoundedRectangle(cornerRadius: IntradaRadius.control))
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.control)
        .stroke(IntradaColor.hairline, lineWidth: 1)
    )
    .cardShadow()
  }
}

#if DEBUG
  #Preview("Away offer") {
    AwayOfferBanner(label: "Away 6 minutes. Leave it out?", onLeaveOut: {}, onKeep: {})
      .padding()
      .background(IntradaColor.paperTop)
  }
#endif
