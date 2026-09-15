import SharedTypes
import SwiftUI

/// Edit sheet for a library item. Sends the fields and, for an exercise, its
/// variation rows; the core validates and reconciles.
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
      // Stop at the first refusal: an event that lands after it clears the error
      // the form is about to show.
      for event in form.editEvents(id: item.id) {
        store.send(.item(event))
        if store.viewModel?.error != nil { return }
      }
    }
  }
}

#if DEBUG
  #Preview {
    LibraryEditScreen(item: .previewDetail)
      .environment(Store.preview)
  }
#endif
