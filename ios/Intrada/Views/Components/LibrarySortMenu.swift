import SharedTypes
import SwiftUI

/// Display wrapper over the core `SortField` — owns the menu labels and the
/// natural default direction when switching *to* a field (a shell concern,
/// like `LibraryFilter` wraps `ItemKind`).
enum LibrarySortField: CaseIterable, Identifiable {
  case dateAdded, lastPracticed, title

  var id: Self { self }

  init(_ core: SortField) {
    switch core {
    case .dateAdded: self = .dateAdded
    case .lastPracticed: self = .lastPracticed
    case .title: self = .title
    }
  }

  var core: SortField {
    switch self {
    case .dateAdded: .dateAdded
    case .lastPracticed: .lastPracticed
    case .title: .title
    }
  }

  var label: String {
    switch self {
    case .dateAdded: "Date Added"
    case .lastPracticed: "Last Practiced"
    case .title: "Title"
    }
  }

  /// Direction applied when the user switches to this field.
  var naturalDefault: SortDirection {
    switch self {
    case .dateAdded, .lastPracticed: .descending
    case .title: .ascending
    }
  }
}

/// Native pull-down sort control (Files/Mail idiom). Tapping the active field
/// flips direction; tapping another switches to it at its natural default.
struct LibrarySortMenu: View {
  let current: LibrarySort
  let onChange: (LibrarySort) -> Void

  var body: some View {
    Menu {
      ForEach(LibrarySortField.allCases) { field in
        Button {
          onChange(next(for: field))
        } label: {
          if field.core == current.field {
            Label(field.label, systemImage: directionSymbol)
          } else {
            Text(field.label)
          }
        }
      }
    } label: {
      Image(systemName: "arrow.up.arrow.down")
        .font(IntradaFont.tab)
        .foregroundStyle(IntradaColor.inkFaint)
        .padding(8)
    }
    .accessibilityLabel("Sort")
    .accessibilityValue("\(LibrarySortField(current.field).label), \(directionAccessibility)")
  }

  private var directionSymbol: String {
    current.direction == .ascending ? "chevron.up" : "chevron.down"
  }

  private var directionAccessibility: String {
    current.direction == .ascending ? "ascending" : "descending"
  }

  private func next(for field: LibrarySortField) -> LibrarySort {
    if field.core == current.field {
      let flipped: SortDirection = current.direction == .ascending ? .descending : .ascending
      return LibrarySort(field: current.field, direction: flipped)
    }
    return LibrarySort(field: field.core, direction: field.naturalDefault)
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      LibrarySortMenu(
        current: LibrarySort(field: .dateAdded, direction: .descending),
        onChange: { _ in })
    }
  }
#endif
