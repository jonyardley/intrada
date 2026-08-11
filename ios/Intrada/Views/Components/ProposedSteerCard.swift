import SharedTypes
import SwiftUI

/// C3 — last night's reflection, back as one proposal (decision 12: propose,
/// confirm, never plan), in Journey A's shape-advice shape. Serif is reserved
/// for the user's own words.
struct ProposedSteerCard: View {
  let steer: ProposedSteer
  var onAccept: () -> Void
  var onDecline: () -> Void

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Label("You said, last night", systemImage: "quote.opening")
        .font(IntradaFont.eyebrow)
        .textCase(.uppercase)
        .kerning(1)
        .foregroundStyle(IntradaColor.accent)
      Text("“\(steer.quote)” \(steer.offer)")
        .font(IntradaFont.cardTitle(16))
        .foregroundStyle(IntradaColor.ink)
        .fixedSize(horizontal: false, vertical: true)
      HStack(spacing: IntradaSpacing.row) {
        Button("Add it to today", action: onAccept)
          .font(IntradaFont.metaMedium)
          .foregroundStyle(IntradaColor.accent)
          .accessibilityHint("Puts one \(steer.minutes)-minute block in today's session")
        Button("Not today", action: onDecline)
          .font(IntradaFont.metaMedium)
          .foregroundStyle(IntradaColor.inkSecondary)
          .accessibilityHint("Leaves today's session as it is, and doesn't ask again")
      }
      .buttonStyle(.plain)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(IntradaSpacing.card)
    .cardSurface()
  }
}

#if DEBUG
  extension ProposedSteer {
    static var preview: ProposedSteer {
      ProposedSteer(
        reflectionId: "01REFLECTION00000000000001",
        quote: "The bridge still rushes when I go from memory.",
        offer: "Give the bridge of Alice in Wonderland 8 minutes today?",
        minutes: 8)
    }
  }

  #Preview {
    ZStack {
      PaperBackground()
      ProposedSteerCard(steer: .preview, onAccept: {}, onDecline: {})
        .padding(IntradaSpacing.card)
    }
  }
#endif
