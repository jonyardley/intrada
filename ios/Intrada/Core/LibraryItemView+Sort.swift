import Foundation
import SharedTypes

extension Array where Element == LibraryItemView {
  /// The core's `sort_library_items` (`app.rs`), for a sheet that sorts its own
  /// candidates: ties fall back to newest first, then id, so the same library
  /// never comes out in a different order here than on the Library screen
  /// (#1445).
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
