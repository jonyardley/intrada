import IntradaCoreFFI
import SharedTypes
import SwiftUI

/// The Focus Player's metronome row. The steppers exist only while the click
/// sounds, so configuration stays one layer down (design-principles T14, T2).
struct ClickControl: View {
  let bpm: Int
  /// The metre's beat value: the readout says `♪ = 168` in 7/8, never `♩`.
  var unit: UInt8 = 4
  let step: Int
  /// The core's band in `unit`, which the drag stops at (#2225).
  let band: ClosedRange<Int>
  let isRunning: Bool
  let unavailable: Bool
  /// False once `bpm` is stepped off what the item seeded: the row then reads as
  /// the click, so stopping never advertises a tempo the next tap would not play.
  let atSeededTempo: Bool
  /// The item's declared tempo. Nil when the row names the click instead.
  let targetDisplay: String?
  let targetSpoken: String?
  let onToggle: () -> Void
  let onStep: (Int) -> Void
  /// The drag's absolute target, unlike `onStep`'s relative nudge (#1823).
  let onDragChange: (Int) -> Void

  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Environment(\.marker) private var marker
  private let initiallyDragging: Bool

  /// Must stay at or above `ClickEngine.leadInSeconds`, or a commit inside
  /// the previous restart's lead-in cancels it before it ever sounds, and
  /// the click goes silent for the whole drag (#1823). Not `private`, so
  /// `ClickControlTests` can assert the invariant directly.
  static let commitInterval: Duration = .milliseconds(
    Int(ClickEngine.leadInSeconds * 1000) + 20)

  init(
    bpm: Int, unit: UInt8 = 4, step: Int, band: ClosedRange<Int>, isRunning: Bool,
    unavailable: Bool, atSeededTempo: Bool,
    targetDisplay: String?, targetSpoken: String?, onToggle: @escaping () -> Void,
    onStep: @escaping (Int) -> Void, onDragChange: @escaping (Int) -> Void,
    initiallyDragging: Bool = false
  ) {
    self.bpm = bpm
    self.unit = unit
    self.step = step
    self.band = band
    self.isRunning = isRunning
    self.unavailable = unavailable
    self.atSeededTempo = atSeededTempo
    self.targetDisplay = targetDisplay
    self.targetSpoken = targetSpoken
    self.onToggle = onToggle
    self.onStep = onStep
    self.onDragChange = onDragChange
    self.initiallyDragging = initiallyDragging
  }

  var body: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      if isRunning {
        TempoStepButton(systemImage: "minus", label: "Slower") { onStep(-step) }
          .transition(.opacity)
      }
      toggle
      if isRunning {
        TempoStepButton(systemImage: "plus", label: "Faster") { onStep(step) }
          .transition(.opacity)
      }
    }
    .animation(reduceMotion ? nil : IntradaMotion.snappy, value: isRunning)
  }

  private var toggle: some View {
    DragValue(
      value: bpm, step: step, range: band,
      tap: .action(onToggle),
      accessibilityLabel: isRunning ? "Stop the metronome" : "Start the metronome",
      accessibilityValue: spokenValue, accessibilityIdentifier: "click.toggle",
      restingFill: isRunning ? marker : .clear,
      commitInterval: Self.commitInterval, initiallyDragging: initiallyDragging,
      onAdjust: onStep, onChange: onDragChange
    ) { shown, dragging in
      Label(readout(bpm: shown), systemImage: "metronome")
        .font(dragging ? IntradaFont.title : IntradaFont.bodyMedium)
        .monospacedDigit()
        // "♩ = 208" is one notation token; at the largest text sizes it wraps
        // between the glyph and the number, which reads as two facts.
        .lineLimit(1)
        .minimumScaleFactor(0.6)
        .foregroundStyle(dragging ? IntradaColor.accent : tint)
    }
  }

  // The steppers are gone while the click is stopped and a drag is out of
  // VoiceOver's reach, so a swipe is the only way to set the tempo (#1943).
  func adjust(_ direction: AccessibilityAdjustmentDirection) {
    switch direction {
    case .increment: onStep(step)
    case .decrement: onStep(-step)
    @unknown default: break
    }
  }

  var readout: String { readout(bpm: bpm) }

  private func readout(bpm shown: Int) -> String {
    if unavailable { return "Metronome unavailable" }
    if showsBpmNumeral { return words(shown).text }
    return targetDisplay ?? "Metronome"
  }

  private var showsBpmNumeral: Bool { isRunning || !atSeededTempo }

  private var tint: Color {
    if unavailable { return IntradaColor.danger }
    return isRunning ? IntradaColor.accent : IntradaColor.inkSecondary
  }

  var spokenValue: String {
    if unavailable { return "unavailable" }
    if showsBpmNumeral { return words(bpm).spoken }
    return targetSpoken ?? words(bpm).spoken
  }

  private func words(_ shown: Int) -> TempoWords {
    clickTempoWords(bpm: UInt16(clamping: shown), unit: unit)
  }
}

#if DEBUG
  #Preview("Click control") {
    VStack(spacing: 32) {
      ClickControl(
        bpm: 66, step: LimitsView.preview.clickStep,
        band: LimitsView.preview.clickBand(unit: 4), isRunning: false, unavailable: false,
        atSeededTempo: true,
        targetDisplay: "Andante · ♩ = 66", targetSpoken: "Andante, 66 beats per minute",
        onToggle: {}, onStep: { _ in }, onDragChange: { _ in })
      ClickControl(
        bpm: 72, step: LimitsView.preview.clickStep,
        band: LimitsView.preview.clickBand(unit: 4), isRunning: true, unavailable: false,
        atSeededTempo: false,
        targetDisplay: "Andante · ♩ = 66", targetSpoken: "Andante, 66 beats per minute",
        onToggle: {}, onStep: { _ in }, onDragChange: { _ in })
      ClickControl(
        bpm: 96, step: LimitsView.preview.clickStep,
        band: LimitsView.preview.clickBand(unit: 4), isRunning: false, unavailable: false,
        atSeededTempo: true,
        targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
        onDragChange: { _ in })
      ClickControl(
        bpm: 96, step: LimitsView.preview.clickStep,
        band: LimitsView.preview.clickBand(unit: 4), isRunning: false, unavailable: true,
        atSeededTempo: true,
        targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
        onDragChange: { _ in })
    }
    .padding()
    .background(RadialGradient.playerPaper)
  }
#endif
