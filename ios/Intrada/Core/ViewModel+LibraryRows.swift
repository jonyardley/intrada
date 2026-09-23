import SharedTypes

extension ViewModel {
  /// The rows the Library filter leaves showing, in sort order (#1998).
  var visibleItems: [LibraryItemView] {
    let byId = Dictionary(items.map { ($0.id, $0) }, uniquingKeysWith: { first, _ in first })
    return visibleIds.compactMap { byId[$0] }
  }

  var recentlyPractisedItems: [LibraryItemView] {
    recentlyPractisedIds.compactMap { id in items.first { $0.id == id } }
  }
}
