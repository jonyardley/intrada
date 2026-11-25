import IntradaCoreFFI
import SharedTypes
import SwiftUI

/// Display wrapper over the core `SortField` for the menu labels, like
/// `LibraryFilter` wraps `ItemKind`; the direction a tap picks is the core's (#2754).
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
    case .dateAdded: "Date added"
    case .lastPracticed: "Last practised"
    case .title: "Title"
    }
  }
}

extension LibrarySort {
  static func selecting(_ field: LibrarySortField, from current: LibrarySort) -> LibrarySort {
    LibrarySort(
      librarySortOnTap(current: PickerSortArg(current), field: PickerSortField(field.core)))
  }

  fileprivate init(_ arg: PickerSortArg) {
    let field: SortField =
      switch arg.field {
      case .dateAdded: .dateAdded
      case .lastPracticed: .lastPracticed
      case .title: .title
      }
    let direction: SortDirection =
      switch arg.direction {
      case .ascending: .ascending
      case .descending: .descending
      }
    self.init(field: field, direction: direction)
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
        .font(IntradaFont.smallMedium)
        .foregroundStyle(IntradaColor.inkFaintIcon)
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
