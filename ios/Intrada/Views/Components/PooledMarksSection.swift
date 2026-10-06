import SharedTypes
import SwiftUI

/// One card per variation or key, saying on how many items it is solid (#2250).
struct PooledMarksSection: View {
  @Environment(\.dynamicTypeSize) private var dynamicTypeSize

  let title: String
  let rows: [PooledMarkView]

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(title: title)
      VStack(spacing: IntradaSpacing.cardCompact) {
        ForEach(rows, id: \.label) { row in
          VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
            if dynamicTypeSize.isAccessibilitySize {
              label(row)
              caption(row)
            } else {
              HStack(alignment: .firstTextBaseline, spacing: IntradaSpacing.cardCompact) {
                label(row)
                  .frame(maxWidth: .infinity, alignment: .leading)
                caption(row)
              }
            }
            SegmentedProgress(count: Int(row.total), filled: Int(row.solid), label: row.caption)
          }
          .padding(IntradaSpacing.cardCompact)
          .cardSurface(cornerRadius: IntradaRadius.card)
          .accessibilityElement(children: .combine)
          .accessibilityLabel("\(row.label), solid on \(row.solid) of \(row.total) items")
        }
      }
    }
  }

  private func label(_ row: PooledMarkView) -> some View {
    Text(row.label)
      .font(IntradaFont.bodyMedium)
      .foregroundStyle(IntradaColor.ink)
  }

  private func caption(_ row: PooledMarkView) -> some View {
    Text(row.caption)
      .font(IntradaFont.secondary)
      .foregroundStyle(IntradaColor.inkSecondary)
  }
}
