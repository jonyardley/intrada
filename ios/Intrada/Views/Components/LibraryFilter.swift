import SharedTypes

/// The library's type filter (`nil` kind = All) — mapped to a core `ListQuery`.
enum LibraryFilter: CaseIterable, Identifiable {
  case all, pieces, exercises

  var id: Self { self }

  init(kind: ItemKind?) {
    switch kind {
    case .none: self = .all
    case .piece: self = .pieces
    case .exercise: self = .exercises
    }
  }

  var label: String {
    switch self {
    case .all: "All"
    case .pieces: "Pieces"
    case .exercises: "Exercises"
    }
  }

  var kind: ItemKind? {
    switch self {
    case .all: nil
    case .pieces: .piece
    case .exercises: .exercise
    }
  }
}
