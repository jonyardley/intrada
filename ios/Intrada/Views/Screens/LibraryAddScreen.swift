import SharedTypes
import SwiftUI

/// Create sheet for a new library item. Sends `Event.item(.add)` — the core
/// validates and (in local-first mode) persists locally with a client-minted
/// ulid; the shell only collects field values.
struct LibraryAddScreen: View {
  @Environment(Store.self) private var store
  @State private var form: ItemFormModel

  init(defaultKind: ItemKind = .piece) {
    _form = State(initialValue: ItemFormModel(kind: defaultKind))
  }

  #if DEBUG
    init(previewError: String) {
      let form = ItemFormModel(kind: .piece)
      form.formError = previewError
      _form = State(initialValue: form)
    }
  #endif

  var body: some View {
    ItemFormScaffold(
      form: form,
      title: "New \(form.kind.label)",
      confirmLabel: "Add",
      composerSuggestions: store.viewModel?.availableComposers ?? [],
      tagSuggestions: store.viewModel?.availableTags ?? []
    ) {
      store.send(.item(.add(form.createInput())))
    }
  }
}

#if DEBUG
  #Preview {
    LibraryAddScreen()
      .environment(Store.preview)
  }
#endif
