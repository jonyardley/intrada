import SwiftUI

/// Full-width "+ Add …" affordance. Three styles share one label:
/// `.dashed` (default) reads as an empty slot inviting input; `.outlined` is the
/// solid-accent-border empty-state CTA; `.plain` is the borderless text footer
/// (e.g. "Add a related exercise" under a populated card).
struct AddRowButton: View {
  enum Style { case dashed, outlined, plain }

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
    case .dashed, .outlined: IntradaSpacing.row
    }
  }

  @ViewBuilder private var background: some View {
    switch style {
    case .dashed:
      RoundedRectangle(cornerRadius: IntradaRadius.card)
        .strokeBorder(
          IntradaColor.addDashOutline, style: StrokeStyle(lineWidth: 1, dash: [4, 4]))
    case .outlined:
      RoundedRectangle(cornerRadius: IntradaRadius.card)
        .strokeBorder(IntradaColor.accent, lineWidth: 1)
    case .plain:
      Color.clear
    }
  }
}

#if DEBUG
  #Preview("Add row") {
    VStack(spacing: 16) {
      AddRowButton(title: "Add a related exercise") {}
      AddRowButton(title: "Add your first exercise", style: .outlined) {}
      AddRowButton(title: "Add a related exercise", style: .plain) {}
    }
    .padding()
    .background(IntradaColor.cardFill)
  }
#endif
