import SharedTypes
import SwiftUI

/// Edit sheet for a library item. Sends `Event.item(.update)` — the core
/// validates and reconciles; the shell only collects field values.
struct LibraryEditScreen: View {
  let item: LibraryItemView
  @Environment(Store.self) private var store
  @State private var form: ItemFormModel

  init(item: LibraryItemView) {
    self.item = item
    _form = State(initialValue: ItemFormModel(item: item))
  }

  #if DEBUG
    init(item: LibraryItemView, previewError: String) {
      self.item = item
      let form = ItemFormModel(item: item)
      form.formError = previewError
      _form = State(initialValue: form)
    }
  #endif

  var body: some View {
    ItemFormScaffold(
      form: form,
      title: "Edit",
      confirmLabel: "Save",
      composerSuggestions: store.viewModel?.availableComposers ?? [],
      tagSuggestions: store.viewModel?.availableTags ?? []
    ) {
      store.send(.item(.update(id: item.id, input: form.updateInput())))
    }
  }
}

#if DEBUG
  #Preview {
    LibraryEditScreen(item: .previewDetail)
      .environment(Store.preview)
  }
#endif
