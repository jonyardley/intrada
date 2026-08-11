import SharedTypes
import SwiftUI

/// C3 — last night's reflection, back as one proposal above the untouched hero.
/// Decision 12: the app proposes, the user confirms, it never plans. Same shape
/// as Journey A's shape-advice card (two inline choices, no button chrome),
/// because both are offers the day survives being declined.
///
/// The quote is the user's own sentence, verbatim from the core, so it is set in
/// serif — the voice reserved for the user's words.
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
