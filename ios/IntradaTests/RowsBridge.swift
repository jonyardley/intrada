import SharedTypes

@testable import Intrada

/// The live bridge plus the rows the core sends apart from the ViewModel
/// (#1801), so a bridge test reads what the Store would hold.
final class RowsBridge: CoreBridge {
  private let live = LiveBridge()
  private(set) var items: [LibraryItemView] = []
  private(set) var sessions: [PracticeSessionView] = []
  private(set) var practiceWeeks: [PracticeWeekView] = []
  private(set) var lastSections: [AppEffect] = []

  func update(_ event: Event) throws -> [Request] { keep(try live.update(event)) }

  func resolve(_ id: UInt32, persistenceOutput: PersistenceOutput) throws -> [Request] {
    keep(try live.resolve(id, persistenceOutput: persistenceOutput))
  }

  func resolve(_ id: UInt32, recognitionOutput: RecognitionOutput) throws -> [Request] {
    keep(try live.resolve(id, recognitionOutput: recognitionOutput))
  }

  func resolveEmpty(_ id: UInt32) throws -> [Request] { keep(try live.resolveEmpty(id)) }

  func view() throws -> ViewModel { try live.view() }

  func rendered() throws -> Rendered {
    Rendered(
      view: try live.view(), items: items, sessions: sessions, practiceWeeks: practiceWeeks)
  }

  private func keep(_ requests: [Request]) -> [Request] {
    lastSections = []
    for request in requests {
      guard case .app(let effect) = request.effect else { continue }
      switch effect {
      case .libraryChanged(let rows): items = rows
      case .historyChanged(let rows): sessions = rows
      case .weeksChanged(let weeks): practiceWeeks = weeks
      default: continue
      }
      lastSections.append(effect)
    }
    return requests
  }
}

@dynamicMemberLookup
struct Rendered {
  let view: ViewModel
  let items: [LibraryItemView]
  let sessions: [PracticeSessionView]
  let practiceWeeks: [PracticeWeekView]

  subscript<T>(dynamicMember path: KeyPath<ViewModel, T>) -> T { view[keyPath: path] }

  var visibleItems: [LibraryItemView] { items.rows(withIds: view.visibleIds) }
  var recentlyPractisedItems: [LibraryItemView] { items.rows(withIds: view.recentlyPractisedIds) }
}

extension RowsBridge {
  /// A whole-piece link added to the piece's set as it stands (#2248).
  func link(exercise: String, to piece: String) throws {
    try setLinks(of: piece) { $0 + [LinkEdit(exercise: .existing(id: exercise), sectionId: nil)] }
  }

  func unlink(exercise: String, from piece: String) throws {
    try setLinks(of: piece) { links in
      links.filter {
        guard case .existing(let id) = $0.exercise else { return true }
        return id != exercise
      }
    }
  }

  private func setLinks(of piece: String, _ change: ([LinkEdit]) -> [LinkEdit]) throws {
    let current = items.first { $0.id == piece }?.linkedExercises.flatMap(\.linkEdits) ?? []
    _ = try update(.item(.setPieceLinks(pieceId: piece, links: change(current))))
  }
}
