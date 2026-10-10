import SharedTypes
import SwiftUI

/// One card per exercise, saying how many of its variations are solid (#1762).
struct VariationCoverageSection: View {
  let caption: String
  let rows: [VariationCoverageView]

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(title: "Variations", trailing: caption)
      VStack(spacing: IntradaSpacing.cardCompact) {
        ForEach(rows, id: \.itemId) { row in
          SolidCountRow(
            title: row.title,
            trailing: row.caption,
            solid: Int(row.solid),
            total: Int(row.total),
            accessibilityLabel: row.spoken)
        }
      }
    }
  }
}
