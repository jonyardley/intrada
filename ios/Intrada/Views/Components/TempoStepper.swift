import IntradaCoreFFI
import SharedTypes
import SwiftUI

/// A tempo nudged with the ring buttons, held to repeat, or typed. A typed
/// number is kept as a draft and clamped to the band when typing ends, so
/// typing 96 does not snap at the 9.
struct TempoStepper: View {
  @Binding var value: Int
  var unit: UInt8 = 4
  let step: Int
  let band: ClosedRange<Int>
  var accessibilityLabel: String = "Achieved tempo"

  @State private var draft = ""
  @FocusState private var typing: Bool

  var body: some View {
    HStack(spacing: 0) {
      TempoStepButton(
        systemImage: "minus", label: "Slower", isEnabled: value > band.lowerBound,
        repeatsWhileHeld: true
      ) { nudge(by: -step) }
      HStack(spacing: IntradaSpacing.segmentGap) {
        Text(prefix)
          .font(IntradaFont.figureLarge)
          .foregroundStyle(IntradaColor.ink)
        TextField("", text: typedText)
          .keyboardType(.numberPad)
          .font(IntradaFont.figureLarge)
          .foregroundStyle(IntradaColor.ink)
          .focused($typing)
          .fixedSize()
      }
      .frame(minWidth: StepperValue.width, minHeight: 44)
      TempoStepButton(
        systemImage: "plus", label: "Faster", isEnabled: value < band.upperBound,
        repeatsWhileHeld: true
      ) { nudge(by: step) }
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
    .onChange(of: typing) { _, now in
      if now { draft = "\(value)" } else { commit() }
    }
    .toolbar {
      if typing {
        ToolbarItemGroup(placement: .keyboard) {
          Spacer()
          Button("Done") { typing = false }
        }
      }
    }
  }

  private var typedText: Binding<String> {
    Binding(get: { typing ? draft : "\(value)" }, set: { draft = $0 })
  }

  private var words: TempoWords { clickTempoWords(bpm: UInt16(clamping: value), unit: unit) }

  /// The core's readout without its number, "♩ =" or "♪ =", beside the typed value.
  private var prefix: String {
    let number = String(value)
    guard words.text.hasSuffix(number) else { return words.text }
    return String(words.text.dropLast(number.count)).trimmingCharacters(in: .whitespaces)
  }

  private func nudge(by delta: Int) {
    if typing {
      commit()
      typing = false
    }
    value = stepped(by: delta)
  }

  private func commit() {
    if let typed = Int(draft) { value = clamped(typed) }
    draft = "\(value)"
  }

  private func stepped(by delta: Int) -> Int { clamped(value + delta) }

  private func clamped(_ tempo: Int) -> Int {
    min(band.upperBound, max(band.lowerBound, tempo))
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
