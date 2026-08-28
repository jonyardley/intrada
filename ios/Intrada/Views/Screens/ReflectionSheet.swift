import SharedTypes
import SwiftUI

/// A tempo and whether the user set it, as one value: `bpm` is not writable
/// except through `set`, so the sheet cannot report a number without also
/// reporting where it came from (#1420).
@MainActor
struct TrackedTempo {
  private(set) var bpm: Int
  private(set) var userSet = false

  init(startingBpm: Int) { bpm = TempoStepper.clamp(startingBpm) }

  mutating func set(_ next: Int) {
    bpm = TempoStepper.clamp(next)
    userSet = true
  }
}

/// What the sheet collected. `tempoUserSet` is an observation, not a
/// judgement: whether it amounts to evidence is the core's ruling (#1420).
struct ReflectionResult {
  let score: UInt8?
  let note: String
  let achievedTempo: UInt16
  /// The user moved the stepper rather than accepting the pre-fill.
  let tempoUserSet: Bool
  let variantId: String?
}

struct ReflectionSheet: View {
  let itemTitle: String
  let elapsedDisplay: String
  /// The item's own declared tempo marking (the practice target), if any.
  let tempoTarget: UInt16?
  /// The item's step ladder, if any. Empty hides the step picker entirely.
  let variants: [VariantView]
  let currentVariantId: String?
  let onSave: (ReflectionResult) -> Void
  let onSkip: () -> Void

  @State private var score: Int = 0
  @State private var note: String = ""
  @State private var achievedTempo: TrackedTempo
  @State private var selectedVariantId: String?

  init(
    itemTitle: String, elapsedDisplay: String, tempoTarget: UInt16?,
    startingTempoBpm: Int = ClickController.defaultBpm,
    variants: [VariantView] = [], currentVariantId: String? = nil,
    onSave: @escaping (ReflectionResult) -> Void,
    onSkip: @escaping () -> Void
  ) {
    self.itemTitle = itemTitle
    self.elapsedDisplay = elapsedDisplay
    self.tempoTarget = tempoTarget
    self.variants = variants
    self.currentVariantId = currentVariantId
    self.onSave = onSave
    self.onSkip = onSkip
    _achievedTempo = State(initialValue: TrackedTempo(startingBpm: startingTempoBpm))
    _selectedVariantId = State(
      initialValue: Self.initialVariantId(currentVariantId: currentVariantId, variants: variants))
  }

  /// Always pre-selected — falls back to the first step by position when
  /// nothing's been tagged yet, so the picker never opens unset.
  static func initialVariantId(currentVariantId: String?, variants: [VariantView]) -> String? {
    currentVariantId ?? variants.first?.id
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

      eyebrow("Mark").padding(.top, IntradaSpacing.section)
      ScoreSelector(score: score, accessibilityLabel: "Mark for \(itemTitle)") { next in
        score = next.map(Int.init) ?? 0
      }
      .padding(.top, IntradaSpacing.controlGap)

      if !variants.isEmpty {
        eyebrow("Step").padding(.top, IntradaSpacing.card)
        stepPicker
          .padding(.top, IntradaSpacing.controlGap)
      }

      eyebrow(tempoTarget.map { "Tempo reached · target ♩ = \($0)" } ?? "Tempo reached")
        .padding(.top, IntradaSpacing.card)
      TempoStepper(value: achievedTempoBinding)
        .padding(.top, IntradaSpacing.controlGap)

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
          ReflectionResult(
            score: score == 0 ? nil : UInt8(score),
            note: note.trimmingCharacters(in: .whitespacesAndNewlines),
            achievedTempo: UInt16(achievedTempo.bpm),
            tempoUserSet: achievedTempo.userSet,
            variantId: selectedVariantId))
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

  // Tap-to-select chips, pre-selected to the current step: this is an input
  // (unlike the display-only ladder on the detail screen), so SegmentedPills
  // applies. The everyday save never touches it — only the rare "actually it
  // was step 3" changes the selection.
  private var stepPicker: some View {
    SegmentedPills(
      options: variants.map(\.id), selection: selectedVariantIdBinding, label: chipLabel)
  }

  // TempoStepper only writes on an explicit tap or accessibility adjustment,
  // never on appear, so a write here is the user considering the number (#1420).
  private var achievedTempoBinding: Binding<Int> {
    Binding(get: { achievedTempo.bpm }, set: { achievedTempo.set($0) })
  }

  private var selectedVariantIdBinding: Binding<String> {
    Binding(
      get: { selectedVariantId ?? variants.first?.id ?? "" },
      set: { selectedVariantId = $0 })
  }

  private func chipLabel(for id: String) -> String {
    guard let step = variants.first(where: { $0.id == id }) else { return "" }
    return step.isCurrent ? "\(step.label) · current" : step.label
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
          onSave: { _ in }, onSkip: {}
        )
        .presentationDetents([.medium, .large])
      }
  }

  #Preview("Reflection · with tempo target") {
    Color.black.opacity(0.2).ignoresSafeArea()
      .sheet(isPresented: .constant(true)) {
        ReflectionSheet(
          itemTitle: "Scales · D♭", elapsedDisplay: "7:00", tempoTarget: 96,
          onSave: { _ in }, onSkip: {}
        )
        .presentationDetents([.medium, .large])
      }
  }
#endif
