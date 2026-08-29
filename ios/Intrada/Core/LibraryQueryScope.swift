import SharedTypes
import SwiftUI

/// Clears the core's shared library `ListQuery` while a browsing sheet is up,
/// and puts back whatever the Library had when it goes away (#1440). Sheets
/// browse the same query the Library screen writes, so without this a filter
/// left behind on one screen silently narrows the other, and a filter applied
/// in a sheet follows the musician back out.
///
/// It clears rather than narrows: `items` also resolves the pushed detail
/// screen, so pinning the query to one type pops the screen underneath the
/// sheet. A sheet that wants one type filters its own candidates.
private struct LibraryQueryScope: ViewModifier {
  @Environment(Store.self) private var store
  @State private var queryBeforeSheet: ListQuery?

  func body(content: Content) -> some View {
    content
      .onAppear {
        queryBeforeSheet = store.viewModel?.activeQuery
        store.send(.setQuery(nil))
      }
      .onDisappear { store.send(.setQuery(queryBeforeSheet)) }
  }
}

extension View {
  /// Opens this sheet on a clean library query and restores the Library's own
  /// query on dismiss.
  func libraryQueryScope() -> some View {
    modifier(LibraryQueryScope())
  }
}
