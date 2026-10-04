import SwiftUI

/// A one-tap offer the musician may ignore: a dashed pill, so it reads as a
/// suggestion rather than something already set ("Last time: B · Dotted").
struct OfferChip: View {
  let title: String
  var systemImage = "arrow.counterclockwise"
  let action: () -> Void

  init(
    _ title: String, systemImage: String = "arrow.counterclockwise", action: @escaping () -> Void
  ) {
    self.title = title
    self.systemImage = systemImage
    self.action = action
  }

  var body: some View {
    Button(action: action) {
      HStack(spacing: 6) {
        Image(systemName: systemImage)
          .iconSize(.badge, weight: .semibold)
          .accessibilityHidden(true)
        Text(title)
          .font(IntradaFont.smallMedium)
          .multilineTextAlignment(.leading)
      }
      .foregroundStyle(IntradaColor.ink)
      .padding(.vertical, 8)
      .padding(.horizontal, IntradaSpacing.cardCompact)
      .background(IntradaColor.cardFill, in: Capsule())
      .overlay {
        Capsule().strokeBorder(
          IntradaColor.addDashOutline, style: StrokeStyle(lineWidth: 1, dash: [4, 3]))
      }
      .frame(minHeight: 44)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
  }
}

#if DEBUG
  #Preview("Offer chip") {
    VStack(alignment: .leading, spacing: 16) {
      OfferChip("Last time: B · Dotted rhythms") {}
      OfferChip("Focus: A1 at 84", systemImage: "plus") {}
    }
    .padding()
    .background(IntradaColor.paperTop)
  }
#endif
