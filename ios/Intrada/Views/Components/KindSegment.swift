import SharedTypes
import SwiftUI

/// Binary Piece/Exercise selector — the same accent-pill language as
/// `LibraryFilterTabs`, laid out full-width inside a segment track. Shared by
/// the add and edit forms so an item's type can be set on create and changed
/// on edit.
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
            .padding(.vertical, 8)
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
