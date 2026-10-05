import SharedTypes
import SwiftUI

/// Edit sheet for a library item. Sends the fields and the variation rows as
/// one save; the core validates and reconciles.
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
      store.send(.item(form.editEvent(id: item.id)))
    }
  }
}

#if DEBUG
  #Preview {
    LibraryEditScreen(item: .previewDetail)
      .environment(Store.preview)
  }
#endif
