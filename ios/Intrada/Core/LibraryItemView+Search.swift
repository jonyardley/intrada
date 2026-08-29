import SharedTypes

extension LibraryItemView {
  /// The shell's copy of the core's free-text match (`apply_query_filter` in
  /// `app.rs`): title, composer, notes and tags, case-insensitively. A sheet
  /// that filters its own candidates uses this so search means the same thing
  /// there as in the Library (#1440).
  func matchesSearch(_ text: String) -> Bool {
    let query = text.trimmingCharacters(in: .whitespaces)
    guard !query.isEmpty else { return true }
    if title.localizedCaseInsensitiveContains(query) { return true }
    if subtitle.localizedCaseInsensitiveContains(query) { return true }
    if notes?.localizedCaseInsensitiveContains(query) == true { return true }
    return tags.contains { $0.localizedCaseInsensitiveContains(query) }
  }
}
