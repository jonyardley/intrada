import IntradaCoreFFI
import SharedTypes
import SwiftUI

/// A number set by dragging it up or down in steps, with a faint up and down
/// glyph saying it can be dragged (#1823). A tap either runs the caller's
/// action or opens the number pad. The caller owns the value, the range and the write.
struct DragValue<Readout: View>: View {
  enum Tap {
    /// The tap is the caller's, such as starting the metronome.
    case action(() -> Void)
    /// The tap opens the number pad; `prefix` sits before the typed number, as "♩ =".
    case typing(prefix: String)
  }

  let value: Int
  let step: Int
  let range: ClosedRange<Int>
  let tap: Tap
  let accessibilityLabel: String
  let accessibilityValue: String
  let accessibilityIdentifier: String
  /// The capsule's fill at rest; a drag always washes it in the accent.
  var restingFill: Color = .clear
  /// The least time between writes during a drag; the last step always lands on release.
  var commitInterval: Duration = .zero
  /// A VoiceOver swipe's step; by default it moves the value one step within the range.
  var onAdjust: ((Int) -> Void)?
  /// Every write, an absolute value inside the range.
  let onChange: (Int) -> Void
  /// The number as shown, and whether a drag is under way.
  @ViewBuilder let readout: (Int, Bool) -> Readout

  @Environment(\.accessibilityReduceMotion) private var reduceMotion

  /// The value the drag started from; every frame steps off this anchor,
  /// not the last frame's value (#1823).
  @State private var dragAnchor: Int?
  @State private var liveValue: Int?
  @State private var lastCommitted: Int?
  @State private var lastCommitAt: ContinuousClock.Instant?
  @State private var draft = ""
  @FocusState private var typing: Bool

  init(
    value: Int, step: Int, range: ClosedRange<Int>, tap: Tap,
    accessibilityLabel: String, accessibilityValue: String, accessibilityIdentifier: String,
    restingFill: Color = .clear, commitInterval: Duration = .zero,
    initiallyDragging: Bool = false,
    onAdjust: ((Int) -> Void)? = nil,
    onChange: @escaping (Int) -> Void,
    @ViewBuilder readout: @escaping (Int, Bool) -> Readout
  ) {
    self.value = value
    self.step = step
    self.range = range
    self.tap = tap
    self.accessibilityLabel = accessibilityLabel
    self.accessibilityValue = accessibilityValue
    self.accessibilityIdentifier = accessibilityIdentifier
    self.restingFill = restingFill
    self.commitInterval = commitInterval
    self.onAdjust = onAdjust
    self.onChange = onChange
    self.readout = readout
    // Previews and snapshots can't drive a real finger drag; this seeds the
    // dragging look in its place (#1823).
    _dragAnchor = State(initialValue: initiallyDragging ? value : nil)
    _liveValue = State(initialValue: initiallyDragging ? value : nil)
  }

  private var isDragging: Bool { dragAnchor != nil }
  private var displayed: Int { liveValue ?? value }

  var body: some View {
    Button(action: tapped) {
      capsule
    }
    .buttonStyle(PressRebound())
    .accessibilityLabel(accessibilityLabel)
    .accessibilityValue(accessibilityValue)
    .accessibilityAdjustableAction(adjust)
    .accessibilityIdentifier(accessibilityIdentifier)
    .highPriorityGesture(dragGesture)
    .overlay { typingField }
    .onDisappear(perform: resetDrag)
    .animation(reduceMotion ? nil : IntradaMotion.snappy, value: isDragging)
  }

  private var capsule: some View {
    HStack(spacing: IntradaSpacing.controlGap / 2) {
      readout(displayed, isDragging)
      if !isDragging {
        Image(systemName: "arrow.up.and.down")
          .iconSize(.caption, weight: .semibold)
          .foregroundStyle(IntradaColor.inkFaintIcon)
          .accessibilityHidden(true)
      }
    }
    .opacity(typing ? 0 : 1)
    .padding(.horizontal, IntradaSpacing.cardCompact)
    .frame(minHeight: 44)
    .background(fill, in: Capsule())
    .dropShadow(isDragging ? .lifted : .none)
    .scaleEffect(isDragging ? 1.04 : 1)
  }

  private var fill: Color {
    isDragging ? IntradaColor.accent.opacity(IntradaOpacity.wash) : restingFill
  }

  /// Always present when typing is allowed, so focus can land on it; seen and
  /// heard only while typing, since VoiceOver adjusts instead.
  @ViewBuilder private var typingField: some View {
    if case .typing(let prefix) = tap {
      HStack(spacing: IntradaSpacing.segmentGap) {
        Text(prefix)
        TextField("", text: $draft)
          .keyboardType(.numberPad)
          .focused($typing)
          .fixedSize()
      }
      .font(IntradaFont.title)
      .monospacedDigit()
      .foregroundStyle(IntradaColor.ink)
      .opacity(typing ? 1 : 0)
      .allowsHitTesting(typing)
      .accessibilityHidden(!typing)
      .onChange(of: typing) { _, now in
        if now { draft = "\(value)" } else { commitTyped() }
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
  }

  private func tapped() {
    switch tap {
    case .action(let action):
      Haptic.impact.play()
      action()
    case .typing:
      typing = true
    }
  }

  /// A typed number is kept as typed and clamped when typing ends, so typing 96
  /// does not snap at the 9.
  private func commitTyped() {
    if let typed = Int(draft) { onChange(clamped(typed)) }
  }

  private var dragGesture: some Gesture {
    DragGesture(minimumDistance: 4)
      .onChanged { drag in
        if typing { typing = false }
        let anchor = dragAnchor ?? value
        if dragAnchor == nil {
          dragAnchor = anchor
          liveValue = value
          lastCommitted = value
          lastCommitAt = .now
        }
        let next = DragStep.value(
          fromDragTranslation: drag.translation.height, anchor: anchor, step: step, range: range)
        guard next != liveValue else { return }
        Haptic.selection.play()
        liveValue = next
        commit(next)
      }
      .onEnded { _ in
        if let liveValue, liveValue != lastCommitted {
          onChange(liveValue)
        }
        resetDrag()
      }
  }

  /// Caps how often a drag writes (#1823): the readout and haptic track every
  /// step, the caller only takes what `commitInterval` allows.
  private func commit(_ next: Int) {
    let now = ContinuousClock.now
    if let lastCommitAt, now - lastCommitAt < commitInterval { return }
    lastCommitted = next
    lastCommitAt = now
    onChange(next)
  }

  private func adjust(_ direction: AccessibilityAdjustmentDirection) {
    let delta: Int
    switch direction {
    case .increment: delta = step
    case .decrement: delta = -step
    @unknown default: return
    }
    if let onAdjust {
      onAdjust(delta)
    } else {
      onChange(clamped(value + delta))
    }
  }

  private func clamped(_ next: Int) -> Int {
    min(range.upperBound, max(range.lowerBound, next))
  }

  private func resetDrag() {
    dragAnchor = nil
    liveValue = nil
    lastCommitted = nil
    lastCommitAt = nil
  }
}

/// Outside the generic type, so tests and callers can reach it without a readout.
enum DragStep {
  /// Points of vertical drag per step, tuned by feel on the simulator (#1823):
  /// fine enough that a step reads as deliberate, loose enough that a thumb's
  /// length drag crosses most of the usable range.
  static let pointsPerStep: CGFloat = 8
}

extension DragStep {
  /// Anchored to the value the gesture began at, not the last frame's value,
  /// so drift never compounds across steps (#1823).
  static func value(
    fromDragTranslation translation: CGFloat, anchor: Int, step: Int, range: ClosedRange<Int>
  ) -> Int {
    let steps = Int(-translation / pointsPerStep)
    return min(range.upperBound, max(range.lowerBound, anchor + steps * step))
  }
}

#if DEBUG
  #Preview("Drag value") {
    VStack(spacing: IntradaSpacing.section) {
      TempoDrag(value: .constant(88), step: 2, band: 40...208)
      TempoDrag(value: .constant(168), unit: 8, step: 2, band: 80...416)
    }
    .padding(IntradaSpacing.section)
    .background(IntradaColor.paperTop)
  }
#endif

/// A tempo as a drag value: the core's readout set as the metronome sets it, in
/// Hanken Grotesk with tabular digits, since "♩ =" is words, not a figure; typed on a tap.
struct TempoDrag: View {
  @Binding var value: Int
  var unit: UInt8 = 4
  let step: Int
  let band: ClosedRange<Int>
  var accessibilityLabel: String = "Achieved tempo"

  var body: some View {
    DragValue(
      value: value, step: step, range: band, tap: .typing(prefix: prefix),
      accessibilityLabel: accessibilityLabel, accessibilityValue: words(value).spoken,
      accessibilityIdentifier: "reflection.tempo", onChange: { value = $0 },
      readout: { shown, dragging in
        Text(words(shown).text)
          .font(IntradaFont.title)
          .monospacedDigit()
          .foregroundStyle(dragging ? IntradaColor.accent : IntradaColor.ink)
          .lineLimit(1)
          .fixedSize()
      })
  }

  private func words(_ tempo: Int) -> TempoWords {
    clickTempoWords(bpm: UInt16(clamping: tempo), unit: unit)
  }

  /// The core's readout without its number, "♩ =" or "♪ =", beside the typed value.
  private var prefix: String {
    let text = words(value).text
    let number = String(value)
    guard text.hasSuffix(number) else { return text }
    return String(text.dropLast(number.count)).trimmingCharacters(in: .whitespaces)
  }
}
