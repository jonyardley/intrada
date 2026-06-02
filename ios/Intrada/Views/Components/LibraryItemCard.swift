import SharedTypes
import SwiftUI

/// A single library row. The type-coded left bar (`ItemKind.bar`) is the
/// always-on type signal, so list rows carry no separate type badge.
struct LibraryItemCard: View {
  let item: LibraryItemView

  var body: some View {
    HStack(alignment: .bottom, spacing: 12) {
      VStack(alignment: .leading, spacing: 3) {
        Text(item.title)
          .font(IntradaFont.cardTitle())
          .foregroundStyle(IntradaColor.ink)
        if !item.subtitle.isEmpty {
          Text(item.subtitle)
            .font(IntradaFont.subtitle)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      Spacer(minLength: 12)
      if hasAttributes {
        VStack(alignment: .trailing, spacing: 4) {
          if let key = item.keyDisplay, !key.isEmpty {
            Text(key)
              .font(IntradaFont.meta)
              .foregroundStyle(IntradaColor.inkFaint)
          }
          if let tempo = item.tempoDisplay, !tempo.isEmpty {
            Text(tempo)
              .font(IntradaFont.meta)
              .foregroundStyle(IntradaColor.inkFaint)
          }
          if !item.tags.isEmpty {
            TagPills(tags: item.tags)
              .padding(.top, 1)
          }
        }
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

  private var hasAttributes: Bool {
    item.keyDisplay?.isEmpty == false || item.tempoDisplay?.isEmpty == false || !item.tags.isEmpty
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
