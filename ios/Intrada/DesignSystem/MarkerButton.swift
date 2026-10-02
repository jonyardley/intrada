import SwiftUI

/// The full-width button in the musician's highlighter: the one call to action on a screen.
struct MarkerButton: View {
  let title: String
  let action: () -> Void

  @Environment(\.marker) private var marker

  init(_ title: String, action: @escaping () -> Void) {
    self.title = title
    self.action = action
  }

  var body: some View {
    Button(action: action) {
      Text(title)
        .font(IntradaFont.button)
        .foregroundStyle(IntradaColor.onMarker)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.card)
        .background(marker)
        .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    }
    .buttonStyle(PressRebound())
  }
}
