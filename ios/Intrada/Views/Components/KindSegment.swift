import SharedTypes
import SwiftUI

/// Full-width Piece/Exercise selector — the accent-pill language of
/// `LibraryFilterTabs`, shared by the add and edit forms.
struct KindSegment: View {
  @Binding var selection: ItemKind
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Namespace private var pill

  var body: some View {
    HStack(spacing: 4) {
      ForEach([ItemKind.piece, ItemKind.exercise], id: \.self) { kind in
        let isSelected = kind == selection
        Button {
          withAnimation(reduceMotion ? nil : .spring(response: 0.35, dampingFraction: 0.8)) {
            selection = kind
          }
        } label: {
          Text(kind.label)
            .font(IntradaFont.segment)
            .foregroundStyle(isSelected ? IntradaColor.onAccent : IntradaColor.inkSecondary)
            .frame(maxWidth: .infinity)
            .padding(.vertical, IntradaSpacing.controlGap)
            .background {
              if isSelected {
                Capsule()
                  .fill(IntradaColor.accent)
                  .matchedGeometryEffect(id: "kindPill", in: pill)
              }
            }
        }
        .buttonStyle(.plain)
        .accessibilityLabel(kind.label)
        .accessibilityAddTraits(isSelected ? [.isSelected] : [])
      }
    }
    .padding(4)
    .background(IntradaColor.cardFill, in: Capsule())
    .overlay(Capsule().stroke(IntradaColor.hairline, lineWidth: 1))
  }
}
