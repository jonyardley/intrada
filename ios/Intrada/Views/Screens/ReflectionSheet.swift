import SwiftUI

/// Hand-off reflection: after an item, score it (1–10), log the tempo reached
/// (when the item declares a target), and jot an optional note. Selector-only
/// — the ring was dropped as redundant with the pills. Pure UI: the caller
/// applies the writes and advances (the score and tempo both need the entry
/// Completed first, so ordering lives in the player, not here).
struct ReflectionSheet: View {
  let itemTitle: String
  let elapsedDisplay: String
  /// The item's own declared tempo marking (the practice target), if any.
  /// `nil` hides the tempo stepper entirely — nothing to log against.
  let tempoTarget: UInt16?
  let onSave: (_ score: UInt8?, _ note: String, _ achievedTempo: UInt16?) -> Void
  let onSkip: () -> Void

  @State private var score: Int = 0
  @State private var note: String = ""
  @State private var achievedTempo: Int

  init(
    itemTitle: String, elapsedDisplay: String, tempoTarget: UInt16?,
    onSave: @escaping (_ score: UInt8?, _ note: String, _ achievedTempo: UInt16?) -> Void,
    onSkip: @escaping () -> Void
  ) {
    self.itemTitle = itemTitle
    self.elapsedDisplay = elapsedDisplay
    self.tempoTarget = tempoTarget
    self.onSave = onSave
    self.onSkip = onSkip
    // Prefilled at target — untouched reads as "played at target".
    _achievedTempo = State(initialValue: Int(tempoTarget ?? 96))
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
          tempoTarget == nil ? nil : UInt16(achievedTempo))
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
          onSave: { _, _, _ in }, onSkip: {}
        )
        .presentationDetents([.medium, .large])
      }
  }

  #Preview("Reflection · with tempo target") {
    Color.black.opacity(0.2).ignoresSafeArea()
      .sheet(isPresented: .constant(true)) {
        ReflectionSheet(
          itemTitle: "Scales · D♭", elapsedDisplay: "7:00", tempoTarget: 96,
          onSave: { _, _, _ in }, onSkip: {}
        )
        .presentationDetents([.medium, .large])
      }
  }
#endif
