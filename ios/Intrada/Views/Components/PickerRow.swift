import SwiftUI

/// One row of a pick-one list, ticked when current: the variation picker and
/// the key picker share it so the two cannot drift apart.
struct PickerRow: View {
  let label: String
  let caption: String?
  let isCurrent: Bool
  let hint: String
  let identifier: String
  let action: () -> Void

  var body: some View {
    Button(action: action) {
      HStack(spacing: IntradaSpacing.cardCompact) {
        VStack(alignment: .leading, spacing: 2) {
          Text(label)
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.ink)
            .multilineTextAlignment(.leading)
          if let caption {
            Text(caption)
              .font(IntradaFont.secondary)
              .foregroundStyle(IntradaColor.inkSecondary)
              .multilineTextAlignment(.leading)
          }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        if isCurrent {
          Image(systemName: "checkmark")
            .font(IntradaFont.segment.weight(.semibold))
            .foregroundStyle(IntradaColor.accent)
        }
      }
      .padding(.horizontal, IntradaSpacing.card)
      .frame(minHeight: 56)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel(caption.map { "\(label), \($0)" } ?? label)
    .accessibilityHint(isCurrent ? "" : hint)
    .accessibilityAddTraits(isCurrent ? [.isSelected] : [])
    .accessibilityIdentifier(identifier)
  }
}
