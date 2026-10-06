import SharedTypes
import SwiftUI

/// One card per exercise, saying how many of its variations are solid (#1762).
struct VariationCoverageSection: View {
  let rows: [VariationCoverageView]

  var body: some View {
    let solid = rows.reduce(0) { $0 + Int($1.solid) }
    let total = rows.reduce(0) { $0 + Int($1.total) }
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(title: "Variations", trailing: "\(solid) of \(total) solid")
      VStack(spacing: IntradaSpacing.cardCompact) {
        ForEach(rows, id: \.itemId) { row in
          SolidCountRow(
            title: row.title,
            trailing: "\(row.solid) of \(row.total) solid",
            solid: Int(row.solid),
            total: Int(row.total),
            accessibilityLabel: "\(row.title), \(row.solid) of \(row.total) variations solid")
        }
      }
    }
  }
}
