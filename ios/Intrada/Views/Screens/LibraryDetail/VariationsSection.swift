import SharedTypes
import SwiftUI

/// The keys an item is practised in and the variations it uses (#2247), each
/// chosen in its own sheet.
struct VariationsSection: View {
  let item: LibraryItemView
  let onChooseKeys: () -> Void
  let onChooseVariations: () -> Void

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.card) {
      if !item.keys.isEmpty {
        VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
          SectionHeader(title: "Keys", action: choose("keys", onChooseKeys))
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
      }
      VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
        SectionHeader(
          title: "Variations",
          action: item.variations.isEmpty ? nil : choose("variations", onChooseVariations))
        if item.variations.isEmpty {
          AddRowButton(
            title: "Choose variations", hint: "Hands separately, dotted rhythms, your own",
            action: onChooseVariations
          )
          .accessibilityLabel("Choose variations for this \(item.itemType.label.lowercased())")
          .accessibilityIdentifier("variations.choose")
        } else {
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
      if item.keys.isEmpty {
        AddRowButton(title: "Practise in other keys", style: .plain, action: onChooseKeys)
          .accessibilityIdentifier("keys.choose")
          .cardSurface()
      }
    }
  }

  private func choose(_ what: String, _ action: @escaping () -> Void) -> SectionHeader.Action {
    .init(title: "Choose", accessibilityLabel: "Choose \(what)", perform: action)
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
