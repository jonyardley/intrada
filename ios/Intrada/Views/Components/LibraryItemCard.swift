import SharedTypes
import SwiftUI

/// A single library row. The type-coded left bar (`ItemKind.bar`) is the
/// always-on type signal, so list rows carry no separate type badge.
struct LibraryItemCard: View {
  let item: LibraryItemView
  // Trailing space reserved for an external accessory the card doesn't own
  // (e.g. an overlaid control) so a long title wraps clear of it.
  var trailingGutter: CGFloat = 0
  // When true, the row shows a trailing ScoreRing for the item's latest
  // 0–10 score (en-dash when never practised) — the glanceable mastery signal.
  var showsMastery: Bool = false
  // Library list only, not the picker sheets, which have their own "Add" (#1727).
  var showsMissingDetailsPrompt: Bool = false

  var body: some View {
    HStack(spacing: IntradaSpacing.card) {
      VStack(alignment: .leading, spacing: 3) {
        Text(item.title)
          .font(IntradaFont.cardTitle)
          .foregroundStyle(IntradaColor.ink)
        if !item.subtitle.isEmpty {
          Text(item.subtitle)
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
        if let meta = metaLine {
          Text(meta)
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
        } else if item.subtitle.isEmpty && showsMissingDetailsPrompt {
          // A prompt, not a collapsed ragged line (#1727).
          Text(missingDetailsPrompt)
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
        if item.priority || hasLinkedExercises || item.keys.count > 1 || item.variations.count > 1
          || !item.tags.isEmpty
        {
          HStack(spacing: 6) {
            if item.priority {
              Image(systemName: "star.fill")
                .iconSize(.caption)
                .foregroundStyle(IntradaColor.accent)
                .accessibilityHidden(true)
            }
            if hasLinkedExercises {
              countChip("\(item.linkedExercises.count)") {
                Image(systemName: "dumbbell.fill").iconSize(.badge)
              }
            }
            if item.keys.count > 1 {
              countChip("\(item.keys.count) keys") {
                Text(verbatim: "♯").font(IntradaFont.secondary)
              }
            }
            if item.variations.count > 1 {
              countChip("\(item.variations.count) variations") {
                Image(systemName: "stairs").iconSize(.badge)
              }
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
        ScoreRing(score: item.practice?.latestScore.map(Int.init), size: 32)
      }
    }
    .padding(.vertical, IntradaSpacing.card)
    .padding(.leading, 20)
    .padding(.trailing, IntradaSpacing.card + trailingGutter)
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

  private var hasLinkedExercises: Bool {
    item.itemType == .piece && !item.linkedExercises.isEmpty
  }

  private func countChip(_ text: String, @ViewBuilder leading: () -> some View) -> some View {
    HStack(spacing: 3) {
      leading()
      Text(text).font(IntradaFont.secondary)
    }
    .foregroundStyle(IntradaColor.exerciseBadgeFg)
    .padding(.horizontal, 7)
    .padding(.vertical, 3)
    .background(IntradaColor.exerciseBadgeBg, in: Capsule())
    .accessibilityHidden(true)
  }

  private var metaLine: String? {
    let parts = [item.keyDisplay, item.tempoLine].compactMap { $0 }.filter { !$0.isEmpty }
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }

  private var missingDetailsPrompt: String {
    item.itemType == .piece ? "Add composer, key and tempo" : "Add key and tempo"
  }

  private var accessibilityLabel: String {
    var parts = [item.itemType.label, item.title]
    if item.priority { parts.append("a priority") }
    if hasLinkedExercises {
      let n = item.linkedExercises.count
      parts.append("\(n) connected exercise\(n == 1 ? "" : "s")")
    }
    if item.keys.count > 1 { parts.append("\(item.keys.count) keys") }
    if item.variations.count > 1 { parts.append("\(item.variations.count) variations") }
    if !item.subtitle.isEmpty { parts.append(item.subtitle) }
    if let key = item.keyDisplay { parts.append(key) }
    if let tempo = item.tempoLineSpoken { parts.append(tempo) }
    if metaLine == nil, item.subtitle.isEmpty, showsMissingDetailsPrompt {
      parts.append(missingDetailsPrompt)
    }
    return parts.joined(separator: ", ")
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.card) {
        LibraryItemCard(item: .previewPiece)
        LibraryItemCard(item: .previewExercise)
        LibraryItemCard(item: .previewExerciseWithTwelveVariations)
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
