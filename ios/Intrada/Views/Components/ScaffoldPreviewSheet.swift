import SharedTypes
import SwiftUI

/// Read-only preview of the curriculum derived from a piece's chord chart
/// (chart-to-scaffold Phase A). The core derives; this only renders. Committing
/// the specs into real linked exercises is Phase B — hence no selection here.
struct ScaffoldPreviewSheet: View {
  let preview: ScaffoldPreviewView

  var body: some View {
    BottomSheet(title: "Your curriculum") {
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
          header
          VStack(spacing: 0) {
            ForEach(Array(preview.specs.enumerated()), id: \.offset) { index, spec in
              if index > 0 {
                HairlineDivider()
              }
              SpecRow(spec: spec)
            }
          }
          .cardSurface()
          if preview.fallbackTotal > 0 {
            fallbackLegend
          }
        }
        .padding(IntradaSpacing.card)
      }
    }
  }

  private var header: some View {
    HStack(spacing: 6) {
      Image(systemName: "key")
        .font(IntradaFont.meta)
        .foregroundStyle(IntradaColor.exerciseBadgeFg)
        .accessibilityHidden(true)
      Text("Derived in \(preview.key) · \(preview.specs.count) exercises")
        .font(IntradaFont.meta)
        .foregroundStyle(IntradaColor.inkSecondary)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .accessibilityElement(children: .combine)
    .accessibilityLabel("Derived in \(preview.key), \(preview.specs.count) exercises")
  }

  private var fallbackLegend: some View {
    HStack(alignment: .top, spacing: 6) {
      Image(systemName: "arrow.turn.down.right")
        .font(IntradaFont.micro)
        .foregroundStyle(IntradaColor.exerciseBadgeFg)
        .accessibilityHidden(true)
      Text("Fallback marks a change we mapped to its arpeggio, not a scale.")
        .font(IntradaFont.micro)
        .foregroundStyle(IntradaColor.inkFaint)
    }
    .padding(.horizontal, IntradaSpacing.controlGap)
  }
}

private struct SpecRow: View {
  let spec: ScaffoldSpecView

  var body: some View {
    HStack(spacing: IntradaSpacing.row) {
      ItemKind.exercise.bar
        .frame(width: 4, height: 34)
        .clipShape(Capsule())
      VStack(alignment: .leading, spacing: 3) {
        HStack(spacing: IntradaSpacing.controlGap) {
          Text(spec.title)
            .font(IntradaFont.cardTitle())
            .foregroundStyle(IntradaColor.ink)
          if spec.alreadyLinked {
            FlagBadge(text: "Already linked", tint: IntradaColor.inkSecondary)
          }
          if spec.fallback {
            FlagBadge(text: "Fallback", tint: IntradaColor.exerciseBadgeFg)
          }
        }
        Text(spec.rationale)
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
          .fixedSize(horizontal: false, vertical: true)
      }
      .frame(maxWidth: .infinity, alignment: .leading)
    }
    .padding(.vertical, IntradaSpacing.row)
    .padding(.leading, 20)
    .padding(.trailing, IntradaSpacing.card)
    .background(IntradaColor.cardFill)
    .accessibilityElement(children: .combine)
    .accessibilityLabel(accessibilityLabel)
  }

  private var accessibilityLabel: String {
    var parts = [spec.title, spec.rationale]
    if spec.alreadyLinked { parts.append("already linked") }
    if spec.fallback { parts.append("uses an arpeggio fallback") }
    return parts.joined(separator: ", ")
  }
}

private struct FlagBadge: View {
  let text: String
  let tint: Color

  var body: some View {
    Text(text)
      .font(IntradaFont.micro)
      .textCase(.uppercase)
      .kerning(0.4)
      .foregroundStyle(tint)
      .padding(.horizontal, 6)
      .padding(.vertical, 2)
      .background(IntradaColor.surfaceSunken, in: Capsule())
      .accessibilityHidden(true)
  }
}

#if DEBUG
  #Preview("Curriculum") {
    Color.clear.sheet(isPresented: .constant(true)) {
      ScaffoldPreviewSheet(preview: .preview)
    }
  }

  extension ScaffoldPreviewView {
    static var preview: ScaffoldPreviewView {
      ScaffoldPreviewView(
        key: "G",
        specs: [
          ScaffoldSpecView(
            title: "Learn the melody", rationale: "Hear the tune before you build on it",
            key: "G", fallback: false, alreadyLinked: true),
          ScaffoldSpecView(
            title: "Shells", rationale: "3rd + 7th of every chord — the voice-leading skeleton",
            key: "G", fallback: false, alreadyLinked: false),
          ScaffoldSpecView(
            title: "Guide-tone lines", rationale: "Connect 3rds to 7ths across each change",
            key: "G", fallback: false, alreadyLinked: false),
          ScaffoldSpecView(
            title: "Scales to chord tones",
            rationale: "Run each chord-scale, landing on a chord tone",
            key: "G", fallback: true, alreadyLinked: false),
          ScaffoldSpecView(
            title: "Constrained improv", rationale: "Chord tones only, then rhythm — one ladder",
            key: "G", fallback: false, alreadyLinked: false),
        ],
        fallbackTotal: 1)
    }
  }
#endif
