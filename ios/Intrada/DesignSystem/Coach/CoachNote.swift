import SwiftUI

/// The block-boundary note — the only place the app speaks in sentences. Three
/// parts, hard-capped: why, the trend, one thing to listen for. Callers show it
/// at most once a session; a boundary with nothing to say omits it entirely
/// rather than rendering this empty.
struct CoachNote: View {
  let thought: String
  var trend: String?
  var listenFor: String?

  var body: some View {
    VStack(alignment: .leading, spacing: 14) {
      Text(thought)
        .font(IntradaFont.cardTitle(23))
        .foregroundStyle(IntradaColor.ink)
        .fixedSize(horizontal: false, vertical: true)
      if let trend {
        Label {
          Text(trend)
        } icon: {
          Image(systemName: "chart.line.uptrend.xyaxis")
            .foregroundStyle(IntradaColor.successTeal)
        }
        .font(IntradaFont.ambient())
        .foregroundStyle(IntradaColor.inkSecondary)
      }
      if let listenFor {
        Text(listenFor)
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.ink)
          .fixedSize(horizontal: false, vertical: true)
      }
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(IntradaSpacing.row)
    .background(
      IntradaColor.cardFill,
      in: RoundedRectangle(cornerRadius: IntradaRadius.panel, style: .continuous)
    )
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.panel, style: .continuous)
        .strokeBorder(IntradaColor.hairline, lineWidth: 1)
    )
    .accessibilityElement(children: .combine)
  }
}

#if DEBUG
  #Preview {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: IntradaSpacing.section) {
        CoachNote(
          thought: "Shells and rootless are what sit between you and improvising over Strasbourg.",
          trend: "100 → 120 bpm this fortnight",
          listenFor: "Next time: keep the left hand off the beat.")
        CoachNote(thought: "Everyone's enclosures sound mechanical for about three weeks.")
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
