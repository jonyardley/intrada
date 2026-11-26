import SwiftUI

/// The Progress hero.
struct MasteryHeroCard: View {
  let mastery: Double
  let change: String?
  let climbing: String?

  private let gutter: CGFloat = 18

  var body: some View {
    HStack(spacing: gutter) {
      MasteryDial(value: mastery)
      summary
      Spacer(minLength: 0)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(gutter)
    .background(IntradaColor.cardFill)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.card)
        .stroke(IntradaColor.hairline, lineWidth: 1))
  }

  private var summary: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.small) {
      SectionTitle("Overall mastery")
      if let change {
        HStack(spacing: IntradaSpacing.segmentGap) {
          Image(systemName: "chart.line.uptrend.xyaxis")
          Text(change)
        }
        .font(IntradaFont.secondary)
        .foregroundStyle(IntradaColor.success)
      }
      if let climbing {
        Text(climbing)
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
    }
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      MasteryHeroCard(
        mastery: 6.4, change: "+1.2 this week", climbing: "Climbing steadily across 5 items."
      )
      .padding(IntradaSpacing.card)
    }
  }
#endif
