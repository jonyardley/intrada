import SwiftUI

/// Stepper (not a numeric field, to avoid a keyboard mid-sheet) for logging
/// achieved tempo on the hand-off reflection sheet.
struct TempoStepper: View {
  @Binding var value: Int
  var unit: UInt8 = 4

  var body: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      TempoStepButton(systemImage: "minus", label: "Slower") {
        value = TempoScale.stepped(from: value, by: -TempoScale.step, unit: unit)
      }
      Text(TempoUnit.readout(value, unit: unit))
        .font(IntradaFont.scoreNumeral(24))
        .monospacedDigit()
        .foregroundStyle(IntradaColor.ink)
        .frame(maxWidth: .infinity)
      TempoStepButton(systemImage: "plus", label: "Faster") {
        value = TempoScale.stepped(from: value, by: TempoScale.step, unit: unit)
      }
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Achieved tempo")
    .accessibilityValue(TempoUnit.spoken(value, unit: unit))
    .accessibilityAdjustableAction { direction in
      switch direction {
      case .increment: value = TempoScale.stepped(from: value, by: TempoScale.step, unit: unit)
      case .decrement: value = TempoScale.stepped(from: value, by: -TempoScale.step, unit: unit)
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
