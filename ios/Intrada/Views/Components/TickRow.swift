import SwiftUI

/// One row of a sheet's tick list: a circle that fills when chosen, the
/// label, and an optional caption beneath it.
struct TickRow: View {
  let label: String
  var caption: String? = nil
  let chosen: Bool
  let identifier: String
  let toggle: () -> Void

  var body: some View {
    Button {
      toggle()
      Haptic.selection.play()
    } label: {
      HStack(spacing: IntradaSpacing.cardCompact) {
        Image(systemName: chosen ? "checkmark.circle.fill" : "circle")
          .iconSize(.control)
          .foregroundStyle(chosen ? IntradaColor.ink : IntradaColor.inkFaintIcon)
        VStack(alignment: .leading, spacing: 3) {
          Text(label)
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
          if let caption {
            Text(caption)
              .font(IntradaFont.secondary)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
        }
        .multilineTextAlignment(.leading)
        .fixedSize(horizontal: false, vertical: true)
        .frame(maxWidth: .infinity, alignment: .leading)
      }
      .padding(.vertical, IntradaSpacing.cardCompact)
      .padding(.horizontal, IntradaSpacing.card)
      .frame(minHeight: 44)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    // Sections may share a name (#2245), so the caption is spoken too.
    .accessibilityLabel(caption.map { "\(label), \($0)" } ?? label)
    .accessibilityAddTraits(chosen ? .isSelected : [])
    .accessibilityIdentifier(identifier)
  }
}
