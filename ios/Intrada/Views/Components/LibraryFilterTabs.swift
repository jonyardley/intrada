import SharedTypes
import SwiftUI

/// The library's type filter. `kind` is `nil` for All; the screen maps it to a
/// core `ListQuery` so the core does the filtering, not the shell.
enum LibraryFilter: CaseIterable, Identifiable {
  case all, pieces, exercises

  var id: Self { self }

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

/// Segmented pill control: indigo-filled selected pill, ghost for the rest.
struct LibraryFilterTabs: View {
  @Binding var selection: LibraryFilter

  var body: some View {
    HStack(spacing: 8) {
      ForEach(LibraryFilter.allCases) { filter in
        let isSelected = filter == selection
        Button {
          selection = filter
        } label: {
          Text(filter.label)
            .font(.system(size: 13, weight: .medium))
            .foregroundStyle(isSelected ? IntradaColor.onAccent : IntradaColor.inkFaint)
            .padding(.vertical, 6)
            .padding(.horizontal, 14)
            .background(isSelected ? IntradaColor.accent : .clear, in: Capsule())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(filter.label)
        .accessibilityAddTraits(isSelected ? [.isSelected] : [])
      }
    }
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
