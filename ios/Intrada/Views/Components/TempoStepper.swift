import SwiftUI

/// Stepper (not a numeric field, to avoid a keyboard mid-sheet) for logging
/// achieved tempo on the hand-off reflection sheet.
struct TempoStepper: View {
  @Binding var value: Int

  /// UI range (DECISIONS.md surface 2) — narrower than the core's 1–400 BPM
  /// validation; out-of-range targets are clamped via `clamp(_:)`.
  static let range = 40...208
  static let step = 2

  /// Moves an out-of-range value toward the range, never past it further.
  static func clamp(_ value: Int) -> Int {
    min(range.upperBound, max(range.lowerBound, value))
  }

  static func stepped(from value: Int, by delta: Int) -> Int {
    clamp(value + delta)
  }

  var body: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      TempoStepButton(systemImage: "minus", label: "Slower") {
        value = Self.stepped(from: value, by: -Self.step)
      }
      Text("♩ = \(value)")
        .font(IntradaFont.scoreNumeral(24))
        .monospacedDigit()
        .foregroundStyle(IntradaColor.ink)
        .frame(maxWidth: .infinity)
      TempoStepButton(systemImage: "plus", label: "Faster") {
        value = Self.stepped(from: value, by: Self.step)
      }
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Achieved tempo")
    .accessibilityValue("\(value) beats per minute")
    .accessibilityAdjustableAction { direction in
      switch direction {
      case .increment: value = Self.stepped(from: value, by: Self.step)
      case .decrement: value = Self.stepped(from: value, by: -Self.step)
      default: break
      }
    }
  }
}

#if DEBUG
  #Preview("Tempo stepper") {
    VStack(spacing: 24) {
      TempoStepper(value: .constant(96))
      TempoStepper(value: .constant(40))
      TempoStepper(value: .constant(208))
    }
    .padding()
    .background(IntradaColor.paperTop)
  }
#endif
