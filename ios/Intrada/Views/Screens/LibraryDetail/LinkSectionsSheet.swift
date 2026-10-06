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
    TickRow(
      label: label, caption: caption, chosen: chosen, identifier: "linkSectionsSheet.row",
      toggle: toggle)
  }

  private func save() {
    if wholePiece == exercise.wholePiece && sectionIds == Swift.Set(exercise.sections.map(\.id)) {
      dismiss()
      return
    }
    formError = nil
    let change = LinkChange.set(
      exerciseId: exercise.id, wholePiece: wholePiece, sectionIds: Array(sectionIds))
    let error = store.sendFromSheet(.item(.changePieceLink(pieceId: piece.id, change: change)))
    withAnimation { formError = error }
    if error == nil { dismiss() }
  }
}
