import SwiftUI

/// The Progress hero. At accessibility sizes the ring drops below the text: the
/// column beside it left the eyebrow too narrow for its own first word (#1471).
struct MasteryHeroCard: View {
  let mastery: Double
  let change: String?
  let climbing: String?

  @Environment(\.dynamicTypeSize) private var dynamicTypeSize

  private let gutter: CGFloat = 18

  var body: some View {
    Group {
      if dynamicTypeSize.isAccessibilitySize {
        VStack(alignment: .leading, spacing: gutter) {
          MasteryDial(value: mastery)
          summary
        }
      } else {
        HStack(spacing: gutter) {
          MasteryDial(value: mastery)
          summary
          Spacer(minLength: 0)
        }
      }
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(gutter)
    .background(IntradaColor.cardFill)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.panel))
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.panel)
        .stroke(IntradaColor.hairline, lineWidth: 1))
  }

  private var summary: some View {
    VStack(alignment: .leading, spacing: 6) {
      SectionTitle("Overall mastery")
      if let change {
        HStack(spacing: 5) {
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

  #Preview("Accessibility size") {
    ZStack {
      PaperBackground()
      MasteryHeroCard(
        mastery: 6.4, change: "+1.2 this week", climbing: "Climbing steadily across 5 items."
      )
      .padding(IntradaSpacing.card)
    }
    .dynamicTypeSize(.accessibility3)
  }
#endif
