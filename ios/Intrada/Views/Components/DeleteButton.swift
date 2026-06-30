import SwiftUI

/// Full-width destructive button — outlined cream surface, red label + trash.
/// The default style for an in-content delete (the system alert action stays
/// system-styled).
struct DeleteButton: View {
  let title: String
  let action: () -> Void

  var body: some View {
    Button(role: .destructive, action: action) {
      Label(title, systemImage: "trash")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.danger)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.cardCompact)
        .background(IntradaColor.cardFill)
        .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
        .overlay(
          RoundedRectangle(cornerRadius: IntradaRadius.card)
            .stroke(IntradaColor.hairline, lineWidth: 1))
    }
    .buttonStyle(.plain)
  }
}

#if DEBUG
  #Preview("Delete button") {
    DeleteButton(title: "Delete piece") {}
      .padding()
      .background(IntradaColor.paperTop)
  }
#endif
