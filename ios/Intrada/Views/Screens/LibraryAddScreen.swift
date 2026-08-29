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
          reading: recognition?.status == .reading,
          onCaptured: { store.send(.item(.readPhoto(photoId: $0))) })
      }
    ) {
      if let pieceId = relatedToPieceId {
        store.send(.item(.addLinkedExercise(pieceId: pieceId, input: form.createInput())))
      } else {
        store.send(.item(.add(form.createInput())))
      }
    }
    .onChange(of: recognition?.draft) { _, draft in
      guard let draft else { return }
      form.fill(from: draft)
    }
    // The draft belongs to this sheet. Leaving without it cleared would hand
    // the next Add screen the last page's fields.
    .onDisappear { store.send(.discardPhotoDraft) }
  }

  private var recognition: PhotoRecognitionView? { store.viewModel?.photoRecognition }
}

#if DEBUG
  #Preview {
    LibraryAddScreen()
      .environment(Store.preview)
  }
#endif
