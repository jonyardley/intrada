import SharedTypes

extension LibraryItemView {
  /// The core's free-text fields (`apply_query_filter` in `app.rs`), for a
  /// sheet that filters its own candidates (#1440).
  func matchesSearch(_ text: String) -> Bool {
    let query = text.trimmingCharacters(in: .whitespaces)
    guard !query.isEmpty else { return true }
    if title.localizedCaseInsensitiveContains(query) { return true }
    if subtitle.localizedCaseInsensitiveContains(query) { return true }
    if notes?.localizedCaseInsensitiveContains(query) == true { return true }
    return tags.contains { $0.localizedCaseInsensitiveContains(query) }
  }
}
