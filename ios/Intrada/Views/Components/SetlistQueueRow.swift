import SharedTypes
import SwiftUI

/// A compact queue row for the builder's bottom tray — tighter than the Library
/// browse cards. The reorder grip is supplied by the enclosing List's edit mode.
struct SetlistQueueRow: View {
  let entry: SetlistEntryView
  let onRemove: () -> Void

  var body: some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      Button(action: onRemove) {
        Image(systemName: "minus.circle.fill")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.danger)
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Remove \(entry.itemTitle)")

      entry.itemType.bar
        .frame(width: 3, height: 24)
        .clipShape(Capsule())

      Text(entry.itemTitle)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.ink)
        .lineLimit(1)
      Spacer(minLength: IntradaSpacing.controlGap)
    }
    .padding(.vertical, IntradaSpacing.controlGap)
    .accessibilityElement(children: .combine)
    .accessibilityLabel("\(entry.itemType.label), \(entry.itemTitle)")
  }
}
