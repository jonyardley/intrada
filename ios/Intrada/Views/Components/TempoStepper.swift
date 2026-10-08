import IntradaCoreFFI
import SharedTypes
import SwiftUI

/// Stepper (not a numeric field, to avoid a keyboard mid-sheet) for logging
/// achieved tempo on the hand-off reflection sheet.
struct TempoStepper: View {
  @Binding var value: Int
  var unit: UInt8 = 4
  let step: Int
  let band: ClosedRange<Int>
  var accessibilityLabel: String = "Achieved tempo"

  var body: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      TempoStepButton(systemImage: "minus", label: "Slower") {
        value = stepped(by: -step)
      }
      Text(words.text)
        .font(IntradaFont.scoreNumeral(24))
        .monospacedDigit()
        .foregroundStyle(IntradaColor.ink)
        .frame(maxWidth: .infinity)
      TempoStepButton(systemImage: "plus", label: "Faster") {
        value = stepped(by: step)
      }
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(accessibilityLabel)
    .accessibilityValue(words.spoken)
    .accessibilityAdjustableAction { direction in
      switch direction {
      case .increment: value = stepped(by: step)
      case .decrement: value = stepped(by: -step)
      default: break
      }
    }
  }

  private var words: TempoWords { clickTempoWords(bpm: UInt16(clamping: value), unit: unit) }

  private func stepped(by delta: Int) -> Int {
    min(band.upperBound, max(band.lowerBound, value + delta))
  }
}

#if DEBUG
  #Preview("Tempo stepper") {
    VStack(spacing: 24) {
      let limits = LimitsView.preview
      let band = limits.clickBand(unit: 4)
      TempoStepper(
        value: .constant(Int(limits.clickTempoDefault)), step: limits.clickStep, band: band)
      TempoStepper(value: .constant(band.lowerBound), step: limits.clickStep, band: band)
      TempoStepper(value: .constant(band.upperBound), step: limits.clickStep, band: band)
    }
    .padding()
    .background(IntradaColor.paperTop)
  }
#endif
