import IntradaCoreFFI
import SharedTypes

extension Array where Element == LibraryItemView {
  /// The picker sheet's own scope, sort and search (#1445, #1440, #1653,
  /// #1999): a plain call, not an `Event`, so a tap in the picker never
  /// touches the Library screen's own `ListQuery` state.
  func sortedAndFiltered(
    by sort: LibrarySort, search: String, filter: PickerFilterArg
  ) -> [LibraryItemView] {
    let candidates = map { item in
      PickerCandidateArg(
        id: item.id, title: item.title, subtitle: item.subtitle, notes: item.notes,
        tags: item.tags, createdAt: item.createdAt,
        lastPracticedAt: item.practice?.lastPracticedAt,
        kind: PickerKind(item.itemType), priority: item.priority)
    }
    let orderedIds = sortAndFilterPickerCandidates(
      candidates: candidates, sort: PickerSortArg(sort), search: search, filter: filter)
    let byId = Dictionary(map { ($0.id, $0) }, uniquingKeysWith: { first, _ in first })
    return orderedIds.compactMap { byId[$0] }
  }
}

extension PickerSortField {
  init(_ field: SortField) {
    switch field {
    case .dateAdded: self = .dateAdded
    case .lastPracticed: self = .lastPracticed
    case .title: self = .title
    }
  }
}

extension PickerSortArg {
  init(_ sort: LibrarySort) {
    let direction: PickerSortDirection =
      switch sort.direction {
      case .ascending: .ascending
      case .descending: .descending
      }
    self.init(field: PickerSortField(sort.field), direction: direction)
  }
}

extension PickerKind {
  init(_ kind: ItemKind) {
    switch kind {
    case .piece: self = .piece
    case .exercise: self = .exercise
    }
  }
}

extension PickerFilterArg {
  init(kind: ItemKind, priorityOnly: Bool = false, tags: [String] = []) {
    self.init(kind: PickerKind(kind), priorityOnly: priorityOnly, tags: tags)
  }
}
