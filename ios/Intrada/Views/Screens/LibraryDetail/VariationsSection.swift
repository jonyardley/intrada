import SharedTypes
import SwiftUI

struct VariationsSection: View {
  let item: LibraryItemView
  let onAddVariations: () -> Void

  @Environment(Store.self) private var store

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      header
      if item.variants.isEmpty {
        emptyState
      } else if item.ladderIsKeys {
        ScrollView(.horizontal, showsIndicators: false) {
          HStack(spacing: IntradaSpacing.card) {
            ForEach(item.variants, id: \.id) { variation in
              VariationRingItem(variation: variation)
            }
          }
          .padding(IntradaSpacing.cardCompact)
        }
        .cardSurface(cornerRadius: IntradaRadius.card)
      } else {
        VStack(spacing: 0) {
          ForEach(Array(item.variants.enumerated()), id: \.element.id) { index, variation in
            if index > 0 {
              HairlineDivider()
            }
            VariationListRow(variation: variation)
          }
        }
        .cardSurface()
      }
    }
  }

  private var header: some View {
    HStack(alignment: .firstTextBaseline) {
      Eyebrow(item.ladderIsKeys ? "Keys" : "Variations")
      if !item.variants.isEmpty {
        Text("\(item.solidVariationCount) of \(item.variants.count) solid")
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      Spacer()
    }
  }

  private var emptyState: some View {
    VStack(spacing: IntradaSpacing.controlGap) {
      AddRowButton(title: "Add 12 major keys") { addKeyPreset(KeyHelper.circleMajor) }
        .accessibilityLabel("Add 12 major keys as this exercise's variations")
      AddRowButton(title: "Add 12 minor keys") { addKeyPreset(KeyHelper.circleMinor) }
        .accessibilityLabel("Add 12 minor keys as this exercise's variations")
      AddRowButton(title: "Add variations", style: .plain, action: onAddVariations)
        .accessibilityLabel("Add variations to this exercise")
    }
    .padding(IntradaSpacing.card)
    .cardSurface()
  }

  private func addKeyPreset(_ labels: [String]) {
    store.send(.item(.setVariants(id: item.id, labels: labels)), onSuccess: .impact)
  }
}

/// One column in the Variations horizontal scroller: a ring (letter + arc)
/// and a state caption below: Solid, calm and static, no pulse (`breathe` and
/// `metro` are retired per `design/CLAUDE.md` "Motion"), or a dash for not yet
/// reached.
private struct VariationRingItem: View {
  let variation: VariantView

  var body: some View {
    VStack(spacing: 6) {
      ScoreRing(
        score: variation.latestScore.map(Int.init), size: 44, solid: variation.isSolid,
        labelOverride: variation.label)
      Text(captionText)
        .font(IntradaFont.meta)
        .foregroundStyle(captionColor)
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(accessibilityLabel)
  }

  private var captionText: String {
    if variation.isSolid { return "Solid" }
    return "—"
  }

  private var captionColor: Color {
    variation.isSolid ? IntradaColor.accent : IntradaColor.inkFaintIcon
  }

  private var accessibilityLabel: String {
    guard let score = variation.latestScore else { return "\(variation.label), not yet attempted" }
    return variation.isSolid
      ? "\(variation.label), solid, \(score) of 10" : "\(variation.label), \(score) of 10"
  }
}

/// Non-key variations row (#1786): a ring's label shrinks to fit, which a
/// free-text name like "Hands together, two octaves" can't survive, so this
/// lays out like `VariationPickerSheet.row` instead: full-width label above
/// the caption, never sharing a line with it, so a long name always keeps
/// the whole row rather than giving up width to the caption.
private struct VariationListRow: View {
  let variation: VariantView

  var body: some View {
    VStack(alignment: .leading, spacing: 2) {
      Text(variation.label)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.leading)
        .fixedSize(horizontal: false, vertical: true)
      Text(captionText)
        .font(IntradaFont.meta)
        .foregroundStyle(captionColor)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(.vertical, IntradaSpacing.card)
    .padding(.horizontal, IntradaSpacing.card)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(accessibilityLabel)
  }

  private var captionText: String {
    variation.caption
  }

  private var captionColor: Color {
    variation.isSolid ? IntradaColor.ink : IntradaColor.inkSecondary
  }

  private var accessibilityLabel: String {
    "\(variation.label), \(variation.caption)"
  }
}
