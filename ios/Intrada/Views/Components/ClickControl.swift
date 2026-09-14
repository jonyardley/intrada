import SwiftUI

/// The Focus Player's metronome row. The steppers exist only while the click
/// sounds, so configuration stays one layer down (design-principles T14, T2).
struct ClickControl: View {
  let bpm: Int
  /// The metre's beat value: the readout says `♪ = 168` in 7/8, never `♩`.
  var unit: UInt8 = 4
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

  /// The tempo the drag started from; every frame steps off this anchor,
  /// not the last frame's value (#1823).
  @State private var dragAnchor: Int?
  /// Tracks every step crossed, for the readout and the haptic; distinct
  /// from what has actually reached the click engine (#1823).
  @State private var liveBpm: Int?
  @State private var lastCommittedBpm: Int?
  @State private var lastCommitAt: ContinuousClock.Instant?
  /// Must stay at or above `ClickEngine.leadInSeconds`, or a commit inside
  /// the previous restart's lead-in cancels it before it ever sounds, and
  /// the click goes silent for the whole drag (#1823). Not `private`, so
  /// `ClickControlTests` can assert the invariant directly.
  static let commitInterval: Duration = .milliseconds(
    Int(ClickEngine.leadInSeconds * 1000) + 20)

  private var isDragging: Bool { dragAnchor != nil }
  private var displayedBpm: Int { liveBpm ?? bpm }

  init(
    bpm: Int, unit: UInt8 = 4, isRunning: Bool, unavailable: Bool, atSeededTempo: Bool,
    targetDisplay: String?, targetSpoken: String?, onToggle: @escaping () -> Void,
    onStep: @escaping (Int) -> Void, onDragChange: @escaping (Int) -> Void,
    initiallyDragging: Bool = false
  ) {
    self.bpm = bpm
    self.unit = unit
    self.isRunning = isRunning
    self.unavailable = unavailable
    self.atSeededTempo = atSeededTempo
    self.targetDisplay = targetDisplay
    self.targetSpoken = targetSpoken
    self.onToggle = onToggle
    self.onStep = onStep
    self.onDragChange = onDragChange
    // Previews/snapshots can't drive a real finger drag; this seeds the
    // dragging visual state in its place (#1823).
    self._dragAnchor = State(initialValue: initiallyDragging ? bpm : nil)
    self._liveBpm = State(initialValue: initiallyDragging ? bpm : nil)
  }

  var body: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      if isRunning {
        TempoStepButton(systemImage: "minus", label: "Slower") { onStep(-TempoScale.step) }
          .transition(.opacity)
      }
      toggle
      if isRunning {
        TempoStepButton(systemImage: "plus", label: "Faster") { onStep(TempoScale.step) }
          .transition(.opacity)
      }
    }
    .animation(reduceMotion ? nil : IntradaMotion.snappy, value: isRunning)
  }

  private var toggle: some View {
    Button {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
      onToggle()
    } label: {
      readoutRow
        .padding(.horizontal, IntradaSpacing.cardCompact)
        .frame(minHeight: 44)
        .background(capsuleFill, in: Capsule())
        .shadow(
          color: isDragging ? IntradaColor.shadow : .clear,
          radius: isDragging ? 6 : 0, y: isDragging ? 3 : 0
        )
        .scaleEffect(isDragging ? 1.04 : 1)
    }
    .buttonStyle(PressRebound())
    .accessibilityLabel(isRunning ? "Stop the metronome" : "Start the metronome")
    .accessibilityValue(spokenValue)
    .highPriorityGesture(dragGesture)
    .onDisappear(perform: resetDrag)
    .animation(reduceMotion ? nil : IntradaMotion.snappy, value: isDragging)
  }

  private var readoutRow: some View {
    HStack(spacing: IntradaSpacing.controlGap / 2) {
      Label(readout, systemImage: "metronome")
        .font(isDragging ? IntradaFont.cardTitle(20) : IntradaFont.bodyMedium)
        .monospacedDigit()
        // "♩ = 208" is one notation token; at the largest text sizes it wraps
        // between the glyph and the number, which reads as two facts.
        .lineLimit(1)
        .minimumScaleFactor(0.6)
        .foregroundStyle(isDragging ? IntradaColor.accent : tint)
      if !isDragging {
        gripGlyph
      }
    }
  }

  /// The passive hint that this is a drag target, not just a toggle, with no
  /// first-use tooltip to dismiss (#1823).
  private var gripGlyph: some View {
    Image(systemName: "arrow.up.and.down")
      .font(.system(size: 11, weight: .semibold))
      .foregroundStyle(IntradaColor.inkFaintIcon)
      .accessibilityHidden(true)
  }

  private var dragGesture: some Gesture {
    DragGesture(minimumDistance: 4)
      .onChanged { value in
        let anchor = dragAnchor ?? bpm
        if dragAnchor == nil {
          dragAnchor = anchor
          liveBpm = bpm
          lastCommittedBpm = bpm
          lastCommitAt = .now
        }
        let next = TempoScale.bpm(
          fromDragTranslation: value.translation.height, anchor: anchor, unit: unit)
        guard next != liveBpm else { return }
        UISelectionFeedbackGenerator().selectionChanged()
        liveBpm = next
        commit(next)
      }
      .onEnded { _ in
        if let liveBpm, liveBpm != lastCommittedBpm {
          onDragChange(liveBpm)
        }
        resetDrag()
      }
  }

  /// Caps how often the drag restarts the click engine (#1823): the readout
  /// and haptic track every step, the engine only takes what its lead-in
  /// silence can absorb without the pulse reading as broken.
  private func commit(_ value: Int) {
    let now = ContinuousClock.now
    if let lastCommitAt, now - lastCommitAt < Self.commitInterval { return }
    lastCommittedBpm = value
    lastCommitAt = now
    onDragChange(value)
  }

  private func resetDrag() {
    dragAnchor = nil
    liveBpm = nil
    lastCommittedBpm = nil
    lastCommitAt = nil
  }

  var readout: String {
    if unavailable { return "Metronome unavailable" }
    if showsBpmNumeral { return TempoUnit.readout(displayedBpm, unit: unit) }
    return targetDisplay ?? "Metronome"
  }

  private var showsBpmNumeral: Bool { isRunning || !atSeededTempo }

  private var tint: Color {
    if unavailable { return IntradaColor.danger }
    return isRunning ? IntradaColor.accent : IntradaColor.inkSecondary
  }

  private var capsuleFill: Color {
    if isDragging { return IntradaColor.accent.opacity(0.14) }
    return isRunning ? marker : .clear
  }

  // VoiceOver never hears the ♩ glyph, so the bpm is spelled out.
  var spokenValue: String {
    if unavailable { return "unavailable" }
    if showsBpmNumeral { return TempoUnit.spoken(displayedBpm, unit: unit) }
    return targetSpoken ?? "no tempo set"
  }
}

#if DEBUG
  #Preview("Click control") {
    VStack(spacing: 32) {
      ClickControl(
        bpm: 66, isRunning: false, unavailable: false, atSeededTempo: true,
        targetDisplay: "Andante · ♩ = 66", targetSpoken: "Andante, 66 beats per minute",
        onToggle: {}, onStep: { _ in }, onDragChange: { _ in })
      ClickControl(
        bpm: 72, isRunning: true, unavailable: false, atSeededTempo: false,
        targetDisplay: "Andante · ♩ = 66", targetSpoken: "Andante, 66 beats per minute",
        onToggle: {}, onStep: { _ in }, onDragChange: { _ in })
      ClickControl(
        bpm: 96, isRunning: false, unavailable: false, atSeededTempo: true,
        targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
        onDragChange: { _ in })
      ClickControl(
        bpm: 96, isRunning: false, unavailable: true, atSeededTempo: true,
        targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
        onDragChange: { _ in })
    }
    .padding()
    .background(RadialGradient.playerPaper)
  }
#endif
