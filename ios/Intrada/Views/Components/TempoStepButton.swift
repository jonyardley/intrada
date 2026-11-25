import SwiftUI

/// Minus or plus in a thin ring, the stepper button every stepper shares (T39).
/// A stepper ignores its children for one adjustable value, so the label is
/// inert there, not unused.
struct TempoStepButton: View {
  let systemImage: String
  let label: String
  var isEnabled = true
  var repeatsWhileHeld = false
  let action: () -> Void

  @State private var repeater: Task<Void, Never>?
  @State private var didRepeat = false

  var body: some View {
    Button {
      if didRepeat {
        didRepeat = false
        return
      }
      fire()
    } label: {
      Image(systemName: systemImage)
        .iconSize(.inline)
        .foregroundStyle(isEnabled ? IntradaColor.ink : IntradaColor.inkFainter)
        .frame(width: 32, height: 32)
        .overlay { Circle().strokeBorder(IntradaColor.divider, lineWidth: 1) }
        .frame(width: 44, height: 44)
        .contentShape(Rectangle())
    }
    .buttonStyle(PressWatching(onPress: pressChanged))
    .disabled(!isEnabled)
    .accessibilityLabel(label)
    .onChange(of: isEnabled) { _, enabled in
      if !enabled { stopRepeating() }
    }
    .onDisappear(perform: stopRepeating)
  }

  private func fire() {
    Haptic.selection.play()
    action()
  }

  private func pressChanged(_ pressed: Bool) {
    guard repeatsWhileHeld else { return }
    if pressed { startRepeating() } else { stopRepeating() }
  }

  private func startRepeating() {
    didRepeat = false
    repeater?.cancel()
    repeater = Task { @MainActor in
      var held: Duration = .zero
      while true {
        let wait = HoldRepeat.wait(after: held)
        do { try await Task.sleep(for: wait) } catch { return }
        held += wait
        didRepeat = true
        fire()
      }
    }
  }

  private func stopRepeating() {
    repeater?.cancel()
    repeater = nil
  }
}

/// The least width between a stepper's rings, the same for every stepper, so
/// the minus and plus line up down a screen; larger text widens it past this.
enum StepperValue {
  static let width: CGFloat = 88
}

/// Hold to repeat: the first repeat waits a moment so a tap stays one step,
/// then repeats steadily, then faster once held for a second.
enum HoldRepeat {
  static let delay: Duration = .milliseconds(400)
  static let interval: Duration = .milliseconds(120)
  static let fastInterval: Duration = .milliseconds(40)
  static let speedsUpAfter: Duration = .seconds(1)

  static func wait(after held: Duration) -> Duration {
    if held == .zero { return delay }
    return held < speedsUpAfter ? interval : fastInterval
  }
}

private struct PressWatching: ButtonStyle {
  let onPress: (Bool) -> Void

  func makeBody(configuration: Configuration) -> some View {
    configuration.label
      .opacity(configuration.isPressed ? IntradaOpacity.soft : 1)
      .onChange(of: configuration.isPressed) { _, pressed in onPress(pressed) }
  }
}
