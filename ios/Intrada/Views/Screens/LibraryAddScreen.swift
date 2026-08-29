import SharedTypes
import SwiftUI

/// Create sheet for a new library item. Sends `Event.item(.add)` — the core
/// validates and (in local-first mode) persists locally with a client-minted
/// ulid; the shell only collects field values.
struct LibraryAddScreen: View {
  @Environment(Store.self) private var store
  @State private var form: ItemFormModel
  /// Set when the exercise is being written for a piece: the core creates and
  /// links it in one event, so it can never land unlinked (#1431).
  private let relatedToPieceId: String?

  init(defaultKind: ItemKind = .piece) {
    _form = State(initialValue: ItemFormModel(kind: defaultKind))
    relatedToPieceId = nil
  }

  init(relatedToPieceId: String) {
    _form = State(initialValue: ItemFormModel(kind: .exercise))
    self.relatedToPieceId = relatedToPieceId
  }

  #if DEBUG
    init(previewError: String) {
      let form = ItemFormModel(kind: .piece)
      form.formError = previewError
      _form = State(initialValue: form)
      relatedToPieceId = nil
    }
  #endif

  var body: some View {
    ItemFormScaffold(
      form: form,
      title: "New \(form.kind.label)",
      confirmLabel: "Add",
      composerSuggestions: store.viewModel?.availableComposers ?? [],
      tagSuggestions: store.viewModel?.availableTags ?? [],
      showsKindPicker: relatedToPieceId == nil,
      header: {
        ScanPageEntry(
          photoId: recognition?.photoId,
          status: recognition?.status ?? .idle,
          readNothing: recognition?.draft.map(readNothing) ?? false,
          onCaptured: { store.send(.item(.readPhoto(photoId: $0))) })
      }
    ) {
      if let pieceId = relatedToPieceId {
        store.send(.item(.addLinkedExercise(pieceId: pieceId, input: form.createInput())))
      } else {
        store.send(.item(.add(form.createInput())))
      }
    }
    // Keyed on the projection, not the draft: re-picking the same library file
    // reads to an equal `PhotoDraft`, so a rescan would silently do nothing.
    .onChange(of: recognition) { _, next in
      form.photoId = next?.photoId
      guard let draft = next?.draft else { return }
      form.fill(from: draft)
    }
  }

  private var recognition: PhotoRecognitionView? { store.viewModel?.photoRecognition }

  private func readNothing(_ draft: PhotoDraft) -> Bool {
    draft.title == nil && draft.composer == nil && draft.tempo == nil && draft.chartText == nil
  }
}

#if DEBUG
  #Preview {
    LibraryAddScreen()
      .environment(Store.preview)
  }
#endif
