import SharedTypes
import SwiftUI

/// What one exercise prepares on this piece: the whole piece, any of its
/// sections, or both (#2248). Saved together on Done.
struct LinkSectionsSheet: View {
  let piece: LibraryItemView
  let exercise: LinkedExerciseView

  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var wholePiece: Bool
  @State private var sectionIds: Swift.Set<String>
  @State private var formError: String?

  init(piece: LibraryItemView, exercise: LinkedExerciseView) {
    self.piece = piece
    self.exercise = exercise
    _wholePiece = State(initialValue: exercise.wholePiece)
    _sectionIds = State(initialValue: Swift.Set(exercise.sections.map(\.id)))
  }

  var body: some View {
    BottomSheet(
      title: exercise.title, detents: [.large], dismissesOnDone: false, onDone: save,
      leadingAction: {
        Button("Cancel") { dismiss() }
          .accessibilityIdentifier("linkSectionsSheet.cancel")
      },
      content: { content }
    )
  }

  private var content: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: IntradaSpacing.card) {
        if let formError {
          FormErrorBanner(message: formError)
        }
        VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
          SectionTitle("Practise it for")
          VStack(spacing: 0) {
            row("The whole piece", caption: nil, chosen: wholePiece) { wholePiece.toggle() }
            ForEach(piece.sections, id: \.id) { section in
              HairlineDivider()
              row(
                section.label, caption: section.barsCaption, chosen: sectionIds.contains(section.id)
              ) {
                if sectionIds.contains(section.id) {
                  sectionIds.remove(section.id)
                } else {
                  sectionIds.insert(section.id)
                }
              }
            }
          }
          .cardSurface()
        }
        Text("Untick everything to take it off this piece.")
          .font(IntradaFont.small)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .padding(IntradaSpacing.card)
    }
  }

  private func row(
    _ label: String, caption: String?, chosen: Bool, toggle: @escaping () -> Void
  ) -> some View {
    Button {
      toggle()
      Haptic.selection.play()
    } label: {
      HStack(spacing: IntradaSpacing.cardCompact) {
        Image(systemName: chosen ? "checkmark.circle.fill" : "circle")
          .iconSize(.control)
          .foregroundStyle(chosen ? IntradaColor.ink : IntradaColor.inkFaintIcon)
        VStack(alignment: .leading, spacing: 3) {
          Text(label)
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
          if let caption {
            Text(caption)
              .font(IntradaFont.secondary)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
        }
        .multilineTextAlignment(.leading)
        .fixedSize(horizontal: false, vertical: true)
        .frame(maxWidth: .infinity, alignment: .leading)
      }
      .padding(.vertical, IntradaSpacing.cardCompact)
      .padding(.horizontal, IntradaSpacing.card)
      .frame(minHeight: 44)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel(label)
    .accessibilityAddTraits(chosen ? .isSelected : [])
    .accessibilityIdentifier("linkSectionsSheet.row")
  }

  private func save() {
    let ordered = piece.sections.map(\.id).filter(sectionIds.contains)
    let unchanged = wholePiece == exercise.wholePiece && ordered == exercise.sections.map(\.id)
    if unchanged {
      dismiss()
      return
    }
    formError = nil
    let links = piece.pieceLinks(setting: exercise.id, wholePiece: wholePiece, sectionIds: ordered)
    let error = store.sendFromSheet(.item(.setPieceLinks(pieceId: piece.id, links: links)))
    withAnimation { formError = error }
    if error == nil { dismiss() }
  }
}
