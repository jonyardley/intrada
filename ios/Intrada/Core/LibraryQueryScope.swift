import SharedTypes
import SwiftUI

/// Clears the shared library `ListQuery` while a sheet is up and restores the
/// Library's own on dismiss (#1440). Clears rather than narrows: `items` also
/// resolves the pushed detail screen, so pinning it pops that screen.
private struct LibraryQueryScope: ViewModifier {
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
        store.send(.setQuery(nil))
      }
      .onDisappear {
        scoped = false
        store.send(.setQuery(queryBeforeSheet))
      }
  }
}

extension View {
  func libraryQueryScope() -> some View {
    modifier(LibraryQueryScope())
  }
}
