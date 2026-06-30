import SwiftUI

/// Full-width dashed "+ Add …" row — the default add affordance. The transparent
/// fill and dashed outline read as an empty slot inviting input.
struct AddRowButton: View {
  let title: String
  let action: () -> Void

  var body: some View {
    Button(action: action) {
      Label(title, systemImage: "plus")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.accent)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.row)
        .background(
          RoundedRectangle(cornerRadius: IntradaRadius.card)
            .strokeBorder(
              IntradaColor.addDashOutline,
              style: StrokeStyle(lineWidth: 1, dash: [4, 4])))
    }
    .buttonStyle(.plain)
    .accessibilityLabel(title)
  }
}

#if DEBUG
  #Preview("Add row") {
    AddRowButton(title: "Add a related exercise") {}
      .padding()
      .background(IntradaColor.cardFill)
  }
#endif
