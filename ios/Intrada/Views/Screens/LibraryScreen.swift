import SharedTypes
import SwiftUI

struct LibraryScreen: View {
  @Environment(Store.self) private var store

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }

  // Core owns the filter: read `activeQuery`, write back via `setQuery` (#792).
  private var filterBinding: Binding<LibraryFilter> {
    Binding(
      get: { LibraryFilter(kind: store.viewModel?.activeQuery?.itemType) },
      set: { store.send(.setQuery(query(for: $0))) })
  }

  var body: some View {
    ScreenScaffold(title: "Library", subtitle: subtitle) {
      VStack(spacing: 0) {
        LibraryFilterTabs(selection: filterBinding)
          .frame(maxWidth: .infinity, alignment: .leading)
          .padding(.horizontal, 16)
          .padding(.top, 12)
          .padding(.bottom, 14)
        content
      }
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
        .padding(.horizontal, 16)
        .padding(.top, 16)
        .padding(.bottom, 16)
      }
      .scrollEdgeShadow()
    }
  }

  private var emptyMessage: String {
    switch LibraryFilter(kind: store.viewModel?.activeQuery?.itemType) {
    case .all: "Your pieces and exercises will live here."
    case .pieces: "No pieces yet."
    case .exercises: "No exercises yet."
    }
  }

  private func query(for filter: LibraryFilter) -> ListQuery? {
    filter.kind.map { ListQuery(text: nil, itemType: $0, key: nil, tags: []) }
  }

  private var subtitle: String? {
    guard let vm = store.viewModel else { return nil }
    let parts = [count(Int(vm.totalPieces), "piece"), count(Int(vm.totalExercises), "exercise")]
      .compactMap { $0 }
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
