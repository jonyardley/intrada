import SharedTypes

/// Distinct, sorted composer names drawn from the whole library (pieces *and*
/// exercises) to feed the composer `AutocompleteField`. Composer is surfaced as
/// `LibraryItemView.subtitle` (see core `view()`), empty when unset.
enum ComposerSuggestions {
  static func from(_ items: [LibraryItemView]?) -> [String] {
    let names = (items ?? []).map { $0.subtitle.trimmingCharacters(in: .whitespacesAndNewlines) }
    return Array(Swift.Set(names.filter { !$0.isEmpty })).sorted {
      $0.localizedCaseInsensitiveCompare($1) == .orderedAscending
    }
  }
}
