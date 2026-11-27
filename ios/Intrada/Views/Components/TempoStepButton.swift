import SwiftUI

/// Minus or plus in a thin ring, the stepper button every stepper shares (T39).
/// A stepper ignores its children for one adjustable value, so the label is
/// inert there, not unused.
struct TempoStepButton: View {
  let systemImage: String
  let label: String
  var isEnabled = true
  /// Called on each repeat while held, until it returns false or the press ends.
  var onHold: ((_ first: Bool) -> Bool)?
  let action: () -> Void

  @State private var holding: Task<Void, Never>?
  @State private var repeated = false

  var body: some View {
    Button {
      guard !repeated else {
        repeated = false
        return
      }
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
    .buttonStyle(PressDimming(onPress: pressChanged))
    .disabled(!isEnabled)
    .accessibilityLabel(label)
    .onDisappear { stopHolding() }
  }

  private func pressChanged(_ pressed: Bool) {
    guard let onHold else { return }
    stopHolding()
    guard pressed else { return }
    repeated = false
    holding = Task { @MainActor in
      try? await Task.sleep(for: HoldRepeat.firstDelay)
      var steps = 0
      while !Task.isCancelled, onHold(steps == 0) {
        repeated = true
        Haptic.selection.play()
        steps += 1
        try? await Task.sleep(for: HoldRepeat.delay(afterSteps: steps))
      }
    }
  }

  private func stopHolding() {
    holding?.cancel()
    holding = nil
  }
}

enum HoldRepeat {
  static let firstDelay: Duration = .milliseconds(400)
  static let stepsBeforeFast = 6

  static func delay(afterSteps steps: Int) -> Duration {
    steps < stepsBeforeFast ? .milliseconds(150) : .milliseconds(60)
  }
}

/// The least width between a stepper's rings, the same for every stepper, so
/// the minus and plus line up down a screen; larger text widens it past this.
enum StepperValue {
  static let width: CGFloat = 88
}

private struct PressDimming: ButtonStyle {
  let onPress: (Bool) -> Void

  func makeBody(configuration: Configuration) -> some View {
    configuration.label
      .opacity(configuration.isPressed ? IntradaOpacity.soft : 1)
      .onChange(of: configuration.isPressed) { _, pressed in onPress(pressed) }
  }
}
