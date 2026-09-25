import SharedTypes

extension [LibraryItemView] {
  func rows(withIds ids: [String]) -> [LibraryItemView] {
    let byId = Dictionary(map { ($0.id, $0) }, uniquingKeysWith: { first, _ in first })
    return ids.compactMap { byId[$0] }
  }
}
