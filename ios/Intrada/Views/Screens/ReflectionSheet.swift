import SharedTypes
import SwiftUI

/// Hand-off reflection: after an item, score it (1–10), log the tempo reached
/// (when the item declares a target), pick which step it was (when the item
/// has a step ladder), and jot an optional note. Pure UI — the caller
/// applies the writes and advances.
struct ReflectionSheet: View {
  let itemTitle: String
  let elapsedDisplay: String
  /// The item's own declared tempo marking (the practice target), if any.
  /// `nil` hides the tempo stepper entirely — nothing to log against.
  let tempoTarget: UInt16?
  /// The item's step ladder, if any. Empty hides the step picker entirely.
  let steps: [StepView]
  let currentVariantId: String?
  let onSave:
    (_ score: UInt8?, _ note: String, _ achievedTempo: UInt16?, _ variantId: String?) -> Void
  let onSkip: () -> Void

  @State private var score: Int = 0
  @State private var note: String = ""
  @State private var achievedTempo: Int
  @State private var selectedVariantId: String?

  init(
    itemTitle: String, elapsedDisplay: String, tempoTarget: UInt16?,
    steps: [StepView] = [], currentVariantId: String? = nil,
    onSave:
      @escaping (
        _ score: UInt8?, _ note: String, _ achievedTempo: UInt16?, _ variantId: String?
      ) -> Void,
    onSkip: @escaping () -> Void
  ) {
    self.itemTitle = itemTitle
    self.elapsedDisplay = elapsedDisplay
    self.tempoTarget = tempoTarget
    self.steps = steps
    self.currentVariantId = currentVariantId
    self.onSave = onSave
    self.onSkip = onSkip
    // Prefilled at target (clamped to the stepper's range) — untouched reads
    // as "played at target".
    _achievedTempo = State(initialValue: TempoStepper.clamp(Int(tempoTarget ?? 96)))
    _selectedVariantId = State(
      initialValue: Self.initialVariantId(currentVariantId: currentVariantId, steps: steps))
  }

  /// Always pre-selected — falls back to the first step by position when
  /// nothing's been tagged yet, so the picker never opens unset. Pulled out
  /// for the same reason as `resolvedAchievedTempo`: directly testable.
  static func initialVariantId(currentVariantId: String?, steps: [StepView]) -> String? {
    currentVariantId ?? steps.first?.id
  }

  /// Pure resolution of the onSave payload's tempo argument — pulled out of
  /// the button closure so the "no target → never send a write" branch is
  /// directly testable without rendering the view.
  static func resolvedAchievedTempo(tempoTarget: UInt16?, current: Int) -> UInt16? {
    guard tempoTarget != nil else { return nil }
    return UInt16(current)
  }

  var body: some View {
    VStack(alignment: .leading, spacing: 0) {
      VStack(spacing: 8) {
        Text("Item complete · \(elapsedDisplay)")
          .font(IntradaFont.badge).textCase(.uppercase).kerning(1.5)
          .foregroundStyle(IntradaColor.exerciseBadgeFg)
        Text("How did \(itemTitle) go?")
          .font(IntradaFont.pageTitle(24)).foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.center)
      }
      .frame(maxWidth: .infinity)
      .padding(.top, IntradaSpacing.card)

      eyebrow("Score").padding(.top, IntradaSpacing.section)
      ScoreSelector(score: score, accessibilityLabel: "Score for \(itemTitle)") { next in
        score = next.map(Int.init) ?? 0
      }
      .padding(.top, IntradaSpacing.controlGap)

      if !steps.isEmpty {
        eyebrow("Step").padding(.top, IntradaSpacing.card)
        stepPicker
          .padding(.top, IntradaSpacing.controlGap)
      }

      if let tempoTarget {
        eyebrow("Tempo reached · target ♩ = \(tempoTarget)").padding(.top, IntradaSpacing.card)
        TempoStepper(value: $achievedTempo)
          .padding(.top, IntradaSpacing.controlGap)
      }

      eyebrow("Reflection · optional").padding(.top, IntradaSpacing.card)
      TextField("What went well? What to fix next time?", text: $note, axis: .vertical)
        .lineLimit(3...5)
        .font(IntradaFont.field)
        .foregroundStyle(IntradaColor.ink)
        .padding(IntradaSpacing.cardCompact)
        .cardSurface(cornerRadius: IntradaRadius.control)
        .padding(.top, IntradaSpacing.controlGap)

      BrandBarButton {
        onSave(
          score == 0 ? nil : UInt8(score),
          note.trimmingCharacters(in: .whitespacesAndNewlines),
          Self.resolvedAchievedTempo(tempoTarget: tempoTarget, current: achievedTempo),
          selectedVariantId)
      } label: {
        Text("Save & continue")
        Image(systemName: "arrow.right")
      }
      .padding(.top, IntradaSpacing.card)

      Button("Skip rating") { onSkip() }
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
        .frame(maxWidth: .infinity)
        .padding(.top, IntradaSpacing.cardCompact)
    }
    .padding(.horizontal, IntradaSpacing.section)
    .padding(.bottom, IntradaSpacing.section)
  }

  // Menu-driven, not a sheet: pre-selected, so the everyday save never slows
  // down to make a choice — this is only for the rare "actually it was step 3".
  private var stepPicker: some View {
    Menu {
      ForEach(steps, id: \.id) { step in
        Button(step.label) { selectedVariantId = step.id }
      }
    } label: {
      HStack {
        Text(selectedStepLabel)
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.ink)
        Spacer()
        Image(systemName: "chevron.up.chevron.down")
          .imageScale(.small)
          .foregroundStyle(IntradaColor.inkFaint)
      }
      .padding(IntradaSpacing.cardCompact)
      .cardSurface(cornerRadius: IntradaRadius.control)
    }
    .buttonStyle(.plain)
    .accessibilityLabel("Step: \(selectedStepLabel)")
    .accessibilityHint("Choose a different step")
  }

  private var selectedStepLabel: String {
    steps.first(where: { $0.id == selectedVariantId })?.label ?? "Choose a step"
  }

  private func eyebrow(_ text: String) -> some View {
    Text(text)
      .font(IntradaFont.eyebrow).textCase(.uppercase).kerning(1.2)
      .foregroundStyle(IntradaColor.inkFaint)
      .frame(maxWidth: .infinity, alignment: .leading)
  }
}

#if DEBUG
  #Preview("Reflection") {
    Color.black.opacity(0.2).ignoresSafeArea()
      .sheet(isPresented: .constant(true)) {
        ReflectionSheet(
          itemTitle: "Scales · D♭", elapsedDisplay: "7:00", tempoTarget: nil,
          onSave: { _, _, _, _ in }, onSkip: {}
        )
        .presentationDetents([.medium, .large])
      }
  }

  #Preview("Reflection · with tempo target") {
    Color.black.opacity(0.2).ignoresSafeArea()
      .sheet(isPresented: .constant(true)) {
        ReflectionSheet(
          itemTitle: "Scales · D♭", elapsedDisplay: "7:00", tempoTarget: 96,
          onSave: { _, _, _, _ in }, onSkip: {}
        )
        .presentationDetents([.medium, .large])
      }
  }
#endif
