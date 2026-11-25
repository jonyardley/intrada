import SwiftUI

/// Minus or plus in a thin ring, the stepper button every stepper shares (T39).
/// A stepper ignores its children for one adjustable value, so the label is
/// inert there, not unused.
struct TempoStepButton: View {
  let systemImage: String
  let label: String
  var isEnabled = true
  let action: () -> Void

  var body: some View {
    Button {
      Haptic.selection.play()
      action()
    } label: {
      Image(systemName: systemImage)
        .iconSize(.inline)
        .foregroundStyle(isEnabled ? IntradaColor.ink : IntradaColor.inkFainter)
        .frame(width: 32, height: 32)
        .overlay { Circle().strokeBorder(IntradaColor.divider, lineWidth: 1) }
        .frame(width: 44, height: 44)
        .contentShape(Rectangle())
    }
    .buttonStyle(PressDimming())
    .disabled(!isEnabled)
    .accessibilityLabel(label)
  }
}

/// The least width between a stepper's rings, the same for every stepper, so
/// the minus and plus line up down a screen; larger text widens it past this.
enum StepperValue {
  static let width: CGFloat = 88
}

private struct PressDimming: ButtonStyle {
  func makeBody(configuration: Configuration) -> some View {
    configuration.label
      .opacity(configuration.isPressed ? IntradaOpacity.soft : 1)
  }
}
