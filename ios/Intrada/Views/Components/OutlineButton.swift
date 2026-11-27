import SwiftUI

/// A small outlined action under the content it changes, such as "Trim to fit"
/// under the minutes used bar.
struct OutlineButton: View {
  let title: String
  let action: () -> Void

  var body: some View {
    Button(action: action) {
      Text(title)
        .font(IntradaFont.label)
        .foregroundStyle(IntradaColor.accent)
        .lineLimit(1)
        .padding(.horizontal, IntradaSpacing.cardCompact)
        .padding(.vertical, IntradaSpacing.small)
        .overlay {
          RoundedRectangle(cornerRadius: IntradaRadius.card)
            .strokeBorder(IntradaColor.divider, lineWidth: 1)
        }
        .frame(minHeight: 44)
        .contentShape(Rectangle())
    }
    .buttonStyle(PressRebound())
  }
}

#if DEBUG
  #Preview("Outline button") {
    HStack(spacing: IntradaSpacing.controlGap) {
      OutlineButton(title: "Trim to fit") {}
      OutlineButton(title: "Drop last item") {}
    }
    .padding(IntradaSpacing.section)
    .background(IntradaColor.paperTop)
  }
#endif
