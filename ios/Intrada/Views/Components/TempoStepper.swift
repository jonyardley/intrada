import SwiftUI

/// Achieved-tempo control for the hand-off reflection sheet: two 44×44 step
/// buttons flanking a large tabular BPM readout. A stepper, not a numeric
/// field, so logging tempo never summons a keyboard mid-sheet; ±2 BPM per tap
/// covers the practical range in a few taps either way. Prefilled at the
/// item's target — untouched reads as "played at target" (design/briefs/
/// 2026-07-reflection-and-narrative.md, DECISIONS.md surface 2).
struct TempoStepper: View {
  @Binding var value: Int

  /// The UI-designed range (DECISIONS.md surface 2). Narrower than the
  /// core's 1–400 BPM validation by design — a target outside this range
  /// (e.g. a Presto marking) is clamped on entry via `clamp(_:)` rather
  /// than widening the control past what fits the sheet.
  static let range = 40...208
  static let step = 2

  /// Symmetric clamp: always moves an out-of-range value *toward* the
  /// range, never away from it (a plain `min`/`max` pair applied
  /// one-sided would let an increment on a below-range value overshoot
  /// further below, or a decrement on an above-range value overshoot
  /// further above).
  static func clamp(_ value: Int) -> Int {
    min(range.upperBound, max(range.lowerBound, value))
  }

  static func stepped(from value: Int, by delta: Int) -> Int {
    clamp(value + delta)
  }

  var body: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      stepButton(systemImage: "minus") {
        value = Self.stepped(from: value, by: -Self.step)
      }
      Text("♩ = \(value)")
        .font(IntradaFont.scoreNumeral(24))
        .monospacedDigit()
        .foregroundStyle(IntradaColor.ink)
        .frame(maxWidth: .infinity)
      stepButton(systemImage: "plus") {
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

  // The parent's `.accessibilityElement(children: .ignore)` already removes
  // these from the accessibility tree — no per-button label/hidden needed
  // (matches `ScoreSelector`'s pattern).
  private func stepButton(systemImage: String, action: @escaping () -> Void)
    -> some View
  {
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
