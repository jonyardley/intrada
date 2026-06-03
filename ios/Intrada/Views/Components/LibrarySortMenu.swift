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

extension LibrarySort {
  /// The sort to apply when the user taps `field` in the menu: flip direction
  /// if it's already the active field, else switch to it at its natural default.
  static func selecting(_ field: LibrarySortField, from current: LibrarySort) -> LibrarySort {
    guard field.core == current.field else {
      return LibrarySort(field: field.core, direction: field.naturalDefault)
    }
    let flipped: SortDirection = current.direction == .ascending ? .descending : .ascending
    return LibrarySort(field: current.field, direction: flipped)
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
          onChange(.selecting(field, from: current))
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
        .padding(IntradaSpacing.controlGap)
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
