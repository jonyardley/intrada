import SharedTypes
import SwiftUI

/// A single library row: type-coded left bar, serif title, composer subtitle,
/// and a `key · tempo` meta line. The bar is the always-on type signal
/// (`ItemKind.bar`); a filtered list needs no extra badge.
struct LibraryItemCard: View {
  let item: LibraryItemView

  var body: some View {
    VStack(alignment: .leading, spacing: 3) {
      Text(item.title)
        .font(IntradaFont.cardTitle())
        .foregroundStyle(IntradaColor.ink)
      if !item.subtitle.isEmpty {
        Text(item.subtitle)
          .font(IntradaFont.subtitle)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      if let meta = metaLine {
        Text(meta)
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkFaint)
      }
      if !item.tags.isEmpty {
        TagPills(tags: item.tags)
          .padding(.top, 5)
      }
    }
    .padding(.vertical, 14)
    .padding(.leading, 20)
    .padding(.trailing, 14)
    .frame(maxWidth: .infinity, alignment: .leading)
    .background(IntradaColor.cardFill)
    // Bar as a leading overlay so it fills the content height without the
    // greedy gradient driving the row taller.
    .overlay(alignment: .leading) {
      item.itemType.bar.frame(width: 4)
    }
    .clipShape(RoundedRectangle(cornerRadius: 12))
    .overlay(
      RoundedRectangle(cornerRadius: 12)
        .stroke(IntradaColor.hairline, lineWidth: 1)
    )
    .accessibilityElement(children: .combine)
    .accessibilityLabel(accessibilityLabel)
  }

  private var metaLine: String? {
    let parts = [item.keyDisplay, item.tempoDisplay].compactMap { $0 }.filter { !$0.isEmpty }
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }

  private var accessibilityLabel: String {
    var parts = [item.itemType.label, item.title]
    if !item.subtitle.isEmpty { parts.append(item.subtitle) }
    if let key = item.keyDisplay { parts.append(key) }
    if let tempo = item.tempoSpoken { parts.append(tempo) }
    return parts.joined(separator: ", ")
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(spacing: 14) {
        LibraryItemCard(item: .previewPiece)
        LibraryItemCard(item: .previewExercise)
      }
      .padding(16)
    }
  }
#endif
