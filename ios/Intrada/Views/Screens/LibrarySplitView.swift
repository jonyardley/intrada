import SharedTypes
import SwiftUI

/// iPad-adaptive Library: a sidebar list + a detail pane on regular width; the
/// unchanged push-navigation stack on compact (iPhone). Selection drives the
/// detail pane directly on iPad rather than pushing.
struct LibrarySplitView: View {
  @Environment(Store.self) private var store
  @Environment(\.horizontalSizeClass) private var sizeClass
  @State private var selectedId: String?

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }
  private var selectedItem: LibraryItemView? {
    selectedId.flatMap { id in items.first { $0.id == id } }
  }

  var body: some View {
    if sizeClass == .regular {
      HStack(spacing: 0) {
        NavigationStack { LibraryScreen(selection: $selectedId) }
          .frame(maxWidth: 380)
        Divider()
        NavigationStack {
          detailColumn
            // Related exercises / pieces push within the detail pane, not the list.
            .navigationDestination(for: String.self) { id in
              if let found = items.first(where: { $0.id == id }) {
                LibraryDetailScreen(item: found)
              }
            }
        }
        .frame(maxWidth: .infinity)
      }
    } else {
      NavigationStack { LibraryScreen() }
    }
  }

  @ViewBuilder private var detailColumn: some View {
    if let selectedItem {
      LibraryDetailScreen(item: selectedItem)
    } else {
      ZStack {
        PaperBackground()
        PlaceholderContent(
          systemImage: "books.vertical", message: "Select an item to see its details.")
      }
    }
  }
}
