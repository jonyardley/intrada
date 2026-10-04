import SharedTypes
import SwiftUI

struct VariationsSection: View {
  let item: LibraryItemView
  let onAddVariations: () -> Void

  @Environment(Store.self) private var store

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      if item.keys.isEmpty && item.variations.isEmpty {
        SectionTitle("Variations")
        emptyState
      }
      if !item.keys.isEmpty {
        SectionTitle("Keys")
        ScrollView(.horizontal, showsIndicators: false) {
          HStack(spacing: IntradaSpacing.card) {
            ForEach(Array(item.keys.enumerated()), id: \.offset) { _, key in
              KeyRingItem(key: key)
            }
          }
          .padding(IntradaSpacing.cardCompact)
        }
        .cardSurface(cornerRadius: IntradaRadius.card)
      }
      if !item.variations.isEmpty {
        SectionTitle("Variations")
        VStack(spacing: 0) {
          ForEach(Array(item.variations.enumerated()), id: \.element.id) { index, variation in
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

  private var emptyState: some View {
    VStack(spacing: IntradaSpacing.controlGap) {
      AddRowButton(title: "Add 12 major keys") { addKeyPreset(.major) }
        .accessibilityLabel("Add 12 major keys to practise this in")
      AddRowButton(title: "Add 12 minor keys") { addKeyPreset(.minor) }
        .accessibilityLabel("Add 12 minor keys to practise this in")
      AddRowButton(title: "Add variations", style: .plain, action: onAddVariations)
        .accessibilityLabel("Add variations to this item")
    }
    .padding(IntradaSpacing.card)
    .cardSurface()
  }

  private func addKeyPreset(_ mode: Modality) {
    store.send(.item(.updateKeys(id: item.id, keys: KeyHelper.circle(mode))), onSuccess: .impact)
  }
}

private struct KeyRingItem: View {
  let key: ItemKeyView

  var body: some View {
    VStack(spacing: 6) {
      ScoreRing(score: key.latestScore.map(Int.init), size: 44, labelOverride: shortLabel)
      Text(key.caption)
        .font(IntradaFont.secondary)
        .foregroundStyle(IntradaColor.inkSecondary)
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("\(key.label), \(key.caption)")
  }

  private var shortLabel: String {
    guard let selection = KeyHelper.selection(key.key) else { return key.label }
    let tonic = KeyHelper.prettify(selection.spelling)
    return selection.mode == .minor ? "\(tonic)m" : tonic
  }
}

/// A ring's label shrinks to fit, which a free-text name like "Hands
/// together, two octaves" can't survive (#1786), so a variation lays out like
/// `VariationPickerSheet.row`: the full-width label above the caption.
private struct VariationListRow: View {
  let variation: VariationView

  var body: some View {
    VStack(alignment: .leading, spacing: 2) {
      Text(variation.label)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.leading)
        .fixedSize(horizontal: false, vertical: true)
      Text(variation.caption)
        .font(IntradaFont.secondary)
        .foregroundStyle(IntradaColor.inkSecondary)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(.vertical, IntradaSpacing.card)
    .padding(.horizontal, IntradaSpacing.card)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("\(variation.label), \(variation.caption)")
  }
}
