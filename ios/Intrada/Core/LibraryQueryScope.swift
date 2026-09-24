import SharedTypes
import SwiftUI

/// Clears the shared library `ListQuery` while a sheet is up and restores the
/// Library's own on dismiss (#1440). A sheet scoped to one kind starts its
/// query there, so the core does the narrowing.
private struct LibraryQueryScope: ViewModifier {
  let kind: ItemKind?
  @Environment(Store.self) private var store
  @State private var queryBeforeSheet: ListQuery?
  @State private var scoped = false

  func body(content: Content) -> some View {
    content
      // A second `onAppear` would capture the already-cleared query to put back.
      .onAppear {
        guard !scoped else { return }
        scoped = true
        queryBeforeSheet = store.viewModel?.activeQuery
        store.send(
          .setQuery(
            kind.map {
              ListQuery(text: nil, itemType: $0, key: nil, tags: [], priorityOnly: false)
            }))
      }
      .onDisappear {
        scoped = false
        store.send(.setQuery(queryBeforeSheet))
      }
  }
}

extension View {
  func libraryQueryScope(kind: ItemKind? = nil) -> some View {
    modifier(LibraryQueryScope(kind: kind))
  }
}
