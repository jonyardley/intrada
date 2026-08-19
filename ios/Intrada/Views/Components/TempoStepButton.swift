import SwiftUI

/// The bordered ± button both tempo surfaces share. `TempoStepper` ignores its
/// children in favour of one adjustable value, so the label is inert there;
/// `ClickControl` sits its steppers beside a toggle, where each speaks for itself.
struct TempoStepButton: View {
  let systemImage: String
  let label: String
  let action: () -> Void

  var body: some View {
    Button {
      UISelectionFeedbackGenerator().selectionChanged()
      action()
    } label: {
      Image(systemName: systemImage)
        .font(.system(size: 15, weight: .semibold))
        .foregroundStyle(IntradaColor.inkSecondary)
        .frame(width: 44, height: 44)
        .background(IntradaColor.cardFill)
        .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
        .overlay(
          RoundedRectangle(cornerRadius: IntradaRadius.card)
            .strokeBorder(IntradaColor.hairline, lineWidth: 1.5))
    }
    .buttonStyle(.plain)
    .accessibilityLabel(label)
  }
}
