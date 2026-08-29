import SharedTypes
import SwiftUI

/// A library row that toggles membership: checkmark and accent stroke once
/// added. Shared by the two add sheets (#1103) so the added-state treatment
/// can't drift between them.
struct SelectableLibraryRow: View {
  let item: LibraryItemView
  let added: Bool
  let addHint: String
  let removeHint: String
  let action: () -> Void

  var body: some View {
    Button(action: action) {
      LibraryItemCard(item: item)
        .overlay(alignment: .trailing) {
          Image(systemName: added ? "checkmark.circle.fill" : "plus.circle")
            .font(.title2)
            .foregroundStyle(added ? IntradaColor.accent : IntradaColor.inkFaint)
            .padding(.trailing, IntradaSpacing.card)
            .accessibilityHidden(true)
        }
        .overlay(
          RoundedRectangle(cornerRadius: IntradaRadius.card)
            .stroke(IntradaColor.accent, lineWidth: 2).opacity(added ? 1 : 0))
    }
    .buttonStyle(.plain)
    .accessibilityValue(added ? "Added" : "Not added")
    .accessibilityHint(added ? removeHint : addHint)
  }
}
