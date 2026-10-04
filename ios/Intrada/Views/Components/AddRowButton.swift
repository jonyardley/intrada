import SwiftUI

/// Full-width "+ Add …" affordance. Two styles share one label: `.dashed`
/// (default) reads as an empty slot inviting input; `.plain` is the borderless
/// text footer. `hint` sits under the label, saying what the slot is for.
struct AddRowButton: View {
  enum Style { case dashed, plain }

  let title: String
  var hint: String?
  var style: Style = .dashed
  let action: () -> Void

  var body: some View {
    Button(action: action) {
      VStack(spacing: 4) {
        Label(title, systemImage: "plus")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.accent)
        if let hint {
          Text(hint)
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
            .multilineTextAlignment(.center)
        }
      }
      .frame(maxWidth: .infinity)
      .padding(.vertical, verticalPadding)
      .background(background)
    }
    .buttonStyle(.plain)
  }

  private var verticalPadding: CGFloat {
    switch style {
    case .plain: IntradaSpacing.cardCompact
    case .dashed: IntradaSpacing.card
    }
  }

  @ViewBuilder private var background: some View {
    switch style {
    case .dashed:
      RoundedRectangle(cornerRadius: IntradaRadius.card)
        .strokeBorder(
          IntradaColor.addDashOutline, style: StrokeStyle(lineWidth: 1, dash: [4, 4]))
    case .plain:
      Color.clear
    }
  }
}

#if DEBUG
  #Preview("Add row") {
    VStack(spacing: 16) {
      AddRowButton(title: "Add a related exercise") {}
      AddRowButton(title: "Add a related exercise", style: .plain) {}
      AddRowButton(title: "Add sections", hint: "A1, B, Coda, or bars 12 to 14") {}
    }
    .padding()
    .background(IntradaColor.cardFill)
  }
#endif
