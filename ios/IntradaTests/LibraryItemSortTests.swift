import Foundation
import SharedTypes
import Testing

@testable import Intrada

/// Ties are the whole point: the core breaks them with newest-first then id,
/// and a sheet that sorts its own candidates has to do the same or the two
/// screens disagree (#1445).
struct LibraryItemSortTests {
  private static func exercise(
    id: String, title: String, createdAt: String, lastPractised: String? = nil
  ) -> LibraryItemView {
    var item = LibraryItemView.previewExercise
    item.id = id
    item.title = title
    item.createdAt = createdAt
    item.practice = lastPractised.map { .fixture(lastPracticedAt: $0) }
    return item
  }

  // Deliberately in the wrong order for every assertion below, so a comparator
  // that returns "equal" and leaves the input alone fails.
  private static var neverPractised: [LibraryItemView] {
    [
      exercise(id: "b", title: "Scales", createdAt: "2026-01-01"),
      exercise(id: "c", title: "Thirds", createdAt: "2026-06-01"),
      exercise(id: "a", title: "Arpeggios", createdAt: "2026-06-01"),
    ]
  }

  @Test("exercises never practised fall back to newest first, then id")
  func tiesResolveLikeTheCore() {
    let sorted = Self.neverPractised.sortedLikeTheLibrary(
      by: LibrarySort(field: .lastPracticed, direction: .ascending))
    #expect(sorted.map(\.id) == ["a", "c", "b"])
  }

  @Test("reversing the sort does not reverse the tiebreak")
  func tiebreakIgnoresDirection() {
    let sorted = Self.neverPractised.sortedLikeTheLibrary(
      by: LibrarySort(field: .lastPracticed, direction: .descending))
    #expect(sorted.map(\.id) == ["a", "c", "b"])
  }

  // Compared as typed, capital "Scales" wins; only a case-insensitive one doesn't.
  @Test("titles sort case-insensitively, ascending and descending")
  func titleOrdering() {
    let items = [
      Self.exercise(id: "a", title: "Scales", createdAt: "2026-01-01"),
      Self.exercise(id: "b", title: "arpeggios", createdAt: "2026-01-02"),
    ]
    #expect(
      items.sortedLikeTheLibrary(by: LibrarySort(field: .title, direction: .ascending))
        .map(\.title) == ["arpeggios", "Scales"])
    #expect(
      items.sortedLikeTheLibrary(by: LibrarySort(field: .title, direction: .descending))
        .map(\.title) == ["Scales", "arpeggios"])
  }

  // The never-practised one is older, so the tiebreak alone would put it second.
  @Test("a practised exercise sorts after one never practised")
  func neverPractisedSortsEarliest() {
    let items = [
      Self.exercise(id: "a", title: "Scales", createdAt: "2026-01-02", lastPractised: "2026-08-01"),
      Self.exercise(id: "b", title: "Arpeggios", createdAt: "2026-01-01"),
    ]
    #expect(
      items.sortedLikeTheLibrary(by: LibrarySort(field: .lastPracticed, direction: .ascending))
        .map(\.id) == ["b", "a"])
  }
}
