import SharedTypes
import SwiftUI

/// Discrete session-position indicator — N filled segments of M, one per
/// setlist entry. Stepped (not a continuous fill) so it reads
/// as "which item", distinct from the timer's continuous target bar. Completed
/// segments are tinted by that entry's item type (piece vs. exercise) rather
/// than a single brand gradient, so the strip doubles as a glance-able session
/// shape.
struct SegmentedProgress: View {
  let types: [ItemKind]
  let filled: Int
  var height: CGFloat = 4

  private var total: Int { types.count }

  var body: some View {
    HStack(spacing: 5) {
      ForEach(Array(types.enumerated()), id: \.offset) { index, type in
        Capsule()
          .fill(index < filled ? AnyShapeStyle(type.accent) : AnyShapeStyle(IntradaColor.divider))
          .frame(height: height)
      }
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Item \(filled) of \(total)")
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      SegmentedProgress(types: [.piece, .exercise, .exercise, .piece, .piece], filled: 2)
        .padding(IntradaSpacing.card)
    }
  }
#endif
