import SharedTypes
import SwiftUI

struct LibraryScreen: View {
  @Environment(Store.self) private var store
  @State private var filter: LibraryFilter = .all

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }

  var body: some View {
    ScreenScaffold(title: "Library", subtitle: subtitle) {
      VStack(spacing: 0) {
        LibraryFilterTabs(selection: $filter)
          .frame(maxWidth: .infinity, alignment: .leading)
          .padding(.horizontal, 16)
          .padding(.top, 12)
        content
      }
    }
    .onChange(of: filter) { _, newValue in
      store.send(.setQuery(query(for: newValue)))
    }
    // Re-assert the pill's filter onto the core so a recreated view (future
    // detail-nav / iPad split) or a core reset can't leave them out of sync.
    .onAppear {
      store.send(.setQuery(query(for: filter)))
    }
    // Key on the id (not the value) so an edit — which changes the item's
    // hash — doesn't break the pushed destination; the detail re-reads the
    // fresh item from the store, so edits reflect without re-navigating.
    .navigationDestination(for: String.self) { id in
      if let found = items.first(where: { $0.id == id }) {
        LibraryDetailScreen(item: found)
      }
    }
    // The list draws its own serif header, so suppress the nav bar here; the
    // detail keeps it for the back chevron.
    .toolbar(.hidden, for: .navigationBar)
  }

  @ViewBuilder private var content: some View {
    if items.isEmpty {
      PlaceholderContent(
        systemImage: "books.vertical",
        message: emptyMessage)
    } else {
      ScrollView {
        LazyVStack(spacing: 14) {
          ForEach(items, id: \.id) { item in
            NavigationLink(value: item.id) {
              LibraryItemCard(item: item)
            }
            .buttonStyle(.plain)
          }
        }
        .padding(16)
      }
    }
  }

  private var emptyMessage: String {
    switch filter {
    case .all: "Your pieces and exercises will live here."
    case .pieces: "No pieces yet."
    case .exercises: "No exercises yet."
    }
  }

  private func query(for filter: LibraryFilter) -> ListQuery? {
    filter.kind.map { ListQuery(text: nil, itemType: $0, key: nil, tags: []) }
  }

  private var subtitle: String? {
    guard store.viewModel != nil else { return nil }
    let pieces = items.filter { $0.itemType == .piece }.count
    let exercises = items.filter { $0.itemType == .exercise }.count
    let parts = [count(pieces, "piece"), count(exercises, "exercise")].compactMap { $0 }
    return parts.isEmpty ? "No items yet" : parts.joined(separator: " · ")
  }

  private func count(_ n: Int, _ noun: String) -> String? {
    n == 0 ? nil : "\(n) \(noun)\(n == 1 ? "" : "s")"
  }
}

#if DEBUG
  #Preview("Populated") {
    NavigationStack { LibraryScreen() }
      .environment(Store.previewSeeded)
  }

  #Preview("Empty") {
    NavigationStack { LibraryScreen() }
      .environment(Store.preview)
  }
#endif
