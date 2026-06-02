import SwiftUI

/// A tag chip. `.sunken` (recessed fill) is for chips inside a card; `.outlined`
/// (card fill + hairline) is for chips on bare paper, where they need their own
/// definition. Pass `onRemove` for an editable chip with a ✕.
struct TagChip: View {
  enum Style { case sunken, outlined }

  let text: String
  var style: Style = .sunken
  var onRemove: (() -> Void)?

  init(_ text: String, style: Style = .sunken, onRemove: (() -> Void)? = nil) {
    self.text = text
    self.style = style
    self.onRemove = onRemove
  }

  var body: some View {
    if let onRemove {
      chip
        .contentShape(Capsule())
        .onTapGesture(perform: onRemove)
        .accessibilityElement(children: .combine)
        .accessibilityLabel(text)
        // `.isButton` so VoiceOver advertises the tap as activatable — a bare
        // onTapGesture isn't surfaced reliably (matches the KeyPicker convention).
        .accessibilityAddTraits(.isButton)
        .accessibilityHint("Double tap to remove")
    } else {
      chip
    }
  }

  private var chip: some View {
    let removable = onRemove != nil
    return HStack(spacing: 5) {
      Text(text)
        .font(IntradaFont.metaMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
        .lineLimit(1)
      if removable {
        Image(systemName: "xmark")
          .font(.system(size: 9, weight: .semibold))
          .foregroundStyle(IntradaColor.inkFaint)
      }
    }
    .padding(.vertical, style == .outlined ? 5 : (removable ? 4 : 3))
    .padding(.horizontal, style == .outlined ? 10 : (removable ? 9 : 8))
    .background(
      style == .sunken ? IntradaColor.surfaceSunken : IntradaColor.cardFill, in: Capsule()
    )
    .overlay {
      if style == .outlined {
        Capsule().stroke(IntradaColor.hairline, lineWidth: 1)
      }
    }
  }
}
