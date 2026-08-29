import Foundation
import SharedTypes

extension Array where Element == LibraryItemView {
  /// Mirrors the core's `sort_library_items` (`app.rs`): ties fall back to
  /// newest first, then id, so a sheet sorting its own candidates cannot order
  /// the same library differently from the Library screen (#1445).
  func sortedLikeTheLibrary(by sort: LibrarySort) -> [LibraryItemView] {
    sorted { a, b in
      let primary = compareField(a, b, sort.field)
      if primary != .orderedSame {
        return sort.direction == .ascending
          ? primary == .orderedAscending : primary == .orderedDescending
      }
      if a.createdAt != b.createdAt { return a.createdAt > b.createdAt }
      return a.id < b.id
    }
  }
}

private func compareField(
  _ a: LibraryItemView, _ b: LibraryItemView, _ field: SortField
) -> ComparisonResult {
  switch field {
  case .title:
    // The core's lowercased compare, not a localised one, so both screens agree.
    return compare(a.title.lowercased(), b.title.lowercased())
  case .dateAdded:
    return compare(a.createdAt, b.createdAt)
  case .lastPracticed:
    // Never practised sorts earliest, as `Option`'s own ordering does in the core.
    return compare(a.practice?.lastPracticedAt ?? "", b.practice?.lastPracticedAt ?? "")
  }
}

private func compare(_ a: String, _ b: String) -> ComparisonResult {
  if a == b { return .orderedSame }
  return a < b ? .orderedAscending : .orderedDescending
}
