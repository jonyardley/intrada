import SwiftUI

/// A card naming one thing and how much of it is solid, with the bar beneath.
struct SolidCountRow: View {
  @Environment(\.dynamicTypeSize) private var dynamicTypeSize

  let title: String
  let trailing: String
  let solid: Int
  let total: Int
  let accessibilityLabel: String

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      if dynamicTypeSize.isAccessibilitySize {
        titleText
        trailingText
      } else {
        HStack(alignment: .firstTextBaseline, spacing: IntradaSpacing.cardCompact) {
          titleText
            .frame(maxWidth: .infinity, alignment: .leading)
          trailingText
        }
      }
      SegmentedProgress(count: total, filled: solid, label: trailing)
    }
    .padding(IntradaSpacing.cardCompact)
    .cardSurface(cornerRadius: IntradaRadius.card)
    .accessibilityElement(children: .combine)
    .accessibilityLabel(accessibilityLabel)
  }

  private var titleText: some View {
    Text(title)
      .font(IntradaFont.bodyMedium)
      .foregroundStyle(IntradaColor.ink)
  }

  private var trailingText: some View {
    Text(trailing)
      .font(IntradaFont.secondary)
      .foregroundStyle(IntradaColor.inkSecondary)
  }
}
