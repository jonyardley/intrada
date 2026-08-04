import SwiftUI

/// The only place the app speaks in sentences: a block boundary, at most once a
/// session. Exactly three parts — why (citing the declared campaign before graph
/// state), the trend, one thing to listen for. Serif carries the thought, Inter
/// carries the evidence. If there is nothing worth saying, the boundary ships
/// without it; this view is never the "empty" state of anything.
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
