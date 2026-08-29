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
        if item.priority || hasLinkedExercises || hasStepLadder || !item.tags.isEmpty {
          HStack(spacing: 6) {
            if item.priority {
              Image(systemName: "star.fill")
                .font(.system(size: 11))
                .foregroundStyle(IntradaColor.accent)
                .accessibilityHidden(true)
            }
            if hasLinkedExercises {
              linkedCountChip
            }
            if hasStepLadder {
              stepCountChip
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

  private var hasLinkedExercises: Bool {
    item.itemType == .piece && !item.linkedExercises.isEmpty
  }

  // A one-rung ladder is the same as no ladder, so the chip starts at two.
  private var hasStepLadder: Bool {
    item.itemType == .exercise && item.variants.count > 1
  }

  // Count of related exercises — the gold dumbbell mirrors the exercise type bar.
  private var linkedCountChip: some View {
    HStack(spacing: 3) {
      Image(systemName: "dumbbell.fill").font(.system(size: 9))
      Text("\(item.linkedExercises.count)").font(IntradaFont.meta)
    }
    .foregroundStyle(IntradaColor.exerciseBadgeFg)
    .padding(.horizontal, 7)
    .padding(.vertical, 3)
    .background(IntradaColor.exerciseBadgeBg, in: Capsule())
    .accessibilityHidden(true)
  }

  // Named for what the ladder holds: inversions and positions aren't keys.
  private var ladderIsKeys: Bool {
    item.variants.allSatisfy { KeyHelper.isKeyLabel($0.label) }
  }

  private var stepCountLabel: String {
    "\(item.variants.count) \(ladderIsKeys ? "keys" : "steps")"
  }

  private var stepCountChip: some View {
    HStack(spacing: 3) {
      Image(systemName: ladderIsKeys ? "key" : "stairs").font(.system(size: 9))
      Text(stepCountLabel).font(IntradaFont.meta)
    }
    .foregroundStyle(IntradaColor.exerciseBadgeFg)
    .padding(.horizontal, 7)
    .padding(.vertical, 3)
    .background(IntradaColor.exerciseBadgeBg, in: Capsule())
    .accessibilityHidden(true)
  }

  private var metaLine: String? {
    let parts = [item.keyDisplay, item.tempoDisplay].compactMap { $0 }.filter { !$0.isEmpty }
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }

  private var accessibilityLabel: String {
    var parts = [item.itemType.label, item.title]
    if item.priority { parts.append("a priority") }
    if hasLinkedExercises {
      let n = item.linkedExercises.count
      parts.append("\(n) connected exercise\(n == 1 ? "" : "s")")
    }
    if hasStepLadder { parts.append(stepCountLabel) }
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
        LibraryItemCard(item: .previewExerciseWithFullLadder)
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
