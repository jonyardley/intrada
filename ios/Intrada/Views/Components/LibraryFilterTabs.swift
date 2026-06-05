import SharedTypes
import SwiftUI

/// The library's type filter. `kind` is `nil` for All; the screen maps it to a
/// core `ListQuery` so the core does the filtering, not the shell.
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

struct LibraryFilterTabs: View {
  @Binding var selection: LibraryFilter
  var edgeInset: CGFloat = 0

  var body: some View {
    SegmentedPills(
      options: LibraryFilter.allCases, selection: $selection, label: \.label,
      layout: .inlineScrolling(edgeInset: edgeInset))
  }
}

#if DEBUG
  private struct LibraryFilterTabsPreview: View {
    @State private var selection: LibraryFilter = .pieces
    var body: some View {
      ZStack {
        PaperBackground()
        LibraryFilterTabs(selection: $selection)
      }
    }
  }

  #Preview {
    LibraryFilterTabsPreview()
  }
#endif
