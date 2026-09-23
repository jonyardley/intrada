import SharedTypes

extension ViewModel {
  /// The rows the Library filter leaves showing, in sort order (#1998).
  var visibleItems: [LibraryItemView] { libraryItems(for: visibleIds) }

  var recentlyPractisedItems: [LibraryItemView] { libraryItems(for: recentlyPractisedIds) }

  private func libraryItems(for ids: [String]) -> [LibraryItemView] {
    let byId = Dictionary(items.map { ($0.id, $0) }, uniquingKeysWith: { first, _ in first })
    return ids.compactMap { byId[$0] }
  }
}
