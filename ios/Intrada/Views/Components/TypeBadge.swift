import SharedTypes
import SwiftUI

/// Tinted icon-chip badge for an item's type — used where a type-coded list bar
/// isn't present (e.g. the detail header). Piece = indigo, Exercise = gold,
/// Journal = neutral.
struct TypeBadge: View {
  private let icon: String
  private let text: String
  private let spoken: String
  private let foreground: Color
  private let background: Color

  /// Overrides the kind's own label where the surface has a narrower word for
  /// it — the coach loop calls an exercise a "Drill".
  init(kind: ItemKind, label: String? = nil) {
    icon = kind.iconName
    text = label ?? kind.label
    spoken = kind.label
    foreground = kind == .piece ? IntradaColor.pieceBadgeFg : IntradaColor.exerciseBadgeFg
    background = kind == .piece ? IntradaColor.pieceBadgeBg : IntradaColor.exerciseBadgeBg
  }

  /// The compose sheet's three-plus-one kinds (#1256). Same weight and shape as
  /// the library's: a journal target is a third kind, not a second-class one.
  init(kind: ComposeKind, label: String? = nil) {
    icon = kind.iconName
    text = label ?? kind.label
    spoken = kind.label
    foreground = kind.badgeForeground
    background = kind.badgeBackground
  }

  var body: some View {
    HStack(spacing: 5) {
      Image(systemName: icon)
        .imageScale(.small)
      Text(text)
    }
    .font(IntradaFont.badge)
    .foregroundStyle(foreground)
    .padding(.vertical, 5)
    .padding(.horizontal, 10)
    .background(
      background, in: RoundedRectangle(cornerRadius: IntradaRadius.badge, style: .continuous)
    )
    .accessibilityElement(children: .combine)
    .accessibilityLabel(spoken)
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.cardCompact) {
        HStack(spacing: IntradaSpacing.cardCompact) {
          TypeBadge(kind: ItemKind.piece)
          TypeBadge(kind: ItemKind.exercise)
        }
        HStack(spacing: IntradaSpacing.cardCompact) {
          TypeBadge(kind: ComposeKind.journal)
          TypeBadge(kind: ComposeKind.unresolved)
        }
      }
    }
  }
#endif
