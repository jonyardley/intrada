import SwiftUI

/// Full-width "+ Add …" affordance. Two styles share one label: `.dashed`
/// (default) reads as an empty slot inviting input; `.plain` is the borderless
/// text footer.
struct AddRowButton: View {
  enum Style { case dashed, plain }

  let title: String
  var style: Style = .dashed
  let action: () -> Void

  var body: some View {
    Button(action: action) {
      Label(title, systemImage: "plus")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.accent)
        .frame(maxWidth: .infinity)
        .padding(.vertical, verticalPadding)
        .background(background)
    }
    .buttonStyle(.plain)
  }

  private var verticalPadding: CGFloat {
    switch style {
    case .plain: IntradaSpacing.cardCompact
    case .dashed: IntradaSpacing.row
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
    }
    .padding()
    .background(IntradaColor.cardFill)
  }
#endif
