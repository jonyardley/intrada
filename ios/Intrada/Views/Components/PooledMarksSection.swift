import SharedTypes
import SwiftUI

/// One card per variation or key, saying on how many items it is solid (#2250).
struct PooledMarksSection: View {
  let title: String
  let rows: [PooledMarkView]

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(title: title)
      VStack(spacing: IntradaSpacing.cardCompact) {
        ForEach(rows, id: \.label) { row in
          SolidCountRow(
            title: row.label,
            trailing: row.caption,
            solid: Int(row.solid),
            total: Int(row.total),
            accessibilityLabel: "\(row.label), \(row.caption)")
        }
      }
    }
  }
}
