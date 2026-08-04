import SharedTypes
import SwiftUI

/// Tinted icon-chip badge for an item's type — used where a type-coded list bar
/// isn't present (e.g. the detail header). Piece = indigo, Exercise = gold.
struct TypeBadge: View {
  let kind: ItemKind
  /// Overrides the kind's own label where the surface has a narrower word for
  /// it — the coach loop calls an exercise a "Drill".
  var label: String?

  var body: some View {
    HStack(spacing: 5) {
      Image(systemName: kind.iconName)
        .imageScale(.small)
      Text(label ?? kind.label)
    }
    .font(IntradaFont.badge)
    .foregroundStyle(foreground)
    .padding(.vertical, 5)
    .padding(.horizontal, 10)
    .background(
      background, in: RoundedRectangle(cornerRadius: IntradaRadius.badge, style: .continuous)
    )
    .accessibilityElement(children: .combine)
    .accessibilityLabel(kind.label)
  }

  private var foreground: Color {
    switch kind {
    case .piece: IntradaColor.pieceBadgeFg
    case .exercise: IntradaColor.exerciseBadgeFg
    }
  }

  private var background: Color {
    switch kind {
    case .piece: IntradaColor.pieceBadgeBg
    case .exercise: IntradaColor.exerciseBadgeBg
    }
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      HStack(spacing: IntradaSpacing.cardCompact) {
        TypeBadge(kind: .piece)
        TypeBadge(kind: .exercise)
      }
    }
  }
#endif
