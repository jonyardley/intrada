import SharedTypes
import SwiftUI

/// A single library row. The type-coded left bar (`ItemKind.bar`) is the
/// always-on type signal, so list rows carry no separate type badge.
struct LibraryItemCard: View {
  let item: LibraryItemView
  // Trailing space reserved for an external accessory the card doesn't own
  // (e.g. an overlaid control) so a long title wraps clear of it.
  var trailingGutter: CGFloat = 0
  // When true, the row shows a trailing MasteryMeter for the item's latest
  // 1–5 score (empty when never practised) — the glanceable mastery signal.
  var showsMastery: Bool = false

  var body: some View {
    HStack(spacing: IntradaSpacing.row) {
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
            .foregroundStyle(IntradaColor.inkSecondary)
        }
        if item.priority || !item.tags.isEmpty {
          HStack(spacing: 6) {
            if item.priority {
              Image(systemName: "star.fill")
                .font(.system(size: 11))
                .foregroundStyle(IntradaColor.accent)
                .accessibilityHidden(true)
            }
            if !item.tags.isEmpty {
              TagPills(tags: item.tags)
            }
          }
          .padding(.top, 5)
        }
      }
      .frame(maxWidth: .infinity, alignment: .leading)
      if showsMastery {
        MasteryMeter(level: item.practice?.latestScore.map(Int.init))
      }
    }
    .padding(.vertical, IntradaSpacing.row)
    .padding(.leading, 20)
    .padding(.trailing, IntradaSpacing.row + trailingGutter)
    .frame(maxWidth: .infinity, alignment: .leading)
    .background(IntradaColor.cardFill)
    // Bar as a leading overlay so it fills the content height without the
    // greedy gradient driving the row taller.
    .overlay(alignment: .leading) {
      item.itemType.bar.frame(width: 4)
    }
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.card)
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
    if item.priority { parts.append("starred") }
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
      VStack(spacing: IntradaSpacing.row) {
        LibraryItemCard(item: .previewPiece)
        LibraryItemCard(item: .previewExercise)
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
