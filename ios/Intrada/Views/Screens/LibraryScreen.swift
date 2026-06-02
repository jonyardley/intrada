import SharedTypes
import SwiftUI

struct LibraryScreen: View {
  @Environment(Store.self) private var store
  @State private var adding = false
  @State private var searchText = ""

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }

  // Core owns the filter: read `activeQuery`, write back via `setQuery` (#792).
  private var filterBinding: Binding<LibraryFilter> {
    Binding(
      get: { LibraryFilter(kind: store.viewModel?.activeQuery?.itemType) },
      set: { sendQuery(kind: $0.kind, text: searchText) })
  }

  var body: some View {
    VStack(spacing: 0) {
      if let subtitle {
        Text(subtitle)
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkFaint)
          .frame(maxWidth: .infinity, alignment: .leading)
          .padding(.horizontal, 16)
          .padding(.bottom, 10)
      }
      HStack(spacing: 8) {
        LibraryFilterTabs(selection: filterBinding)
          .frame(maxWidth: .infinity, alignment: .leading)
        LibrarySortMenu(
          current: store.viewModel?.activeSort
            ?? LibrarySort(field: .dateAdded, direction: .descending),
          onChange: { store.send(.setSort($0)) })
      }
      .padding(.horizontal, 16)
      .padding(.bottom, 14)
      content
    }
    .padding(.top, 4)
    .background(PaperBackground().ignoresSafeArea())
    .navigationTitle("Library")
    .navigationBarTitleDisplayMode(.large)
    // Native pull-to-reveal: the drawer tucks under the large title and is
    // revealed by pulling the list. The core does the filtering via ListQuery.text.
    .searchable(
      text: $searchText,
      placement: .navigationBarDrawer(displayMode: .automatic),
      prompt: "Search library"
    )
    .toolbar {
      ToolbarItem(placement: .topBarTrailing) {
        Button {
          adding = true
        } label: {
          Image(systemName: "plus")
            .font(.system(size: 16, weight: .semibold))
        }
        .tint(IntradaColor.accent)
        .accessibilityLabel("Add item")
      }
    }
    .sheet(isPresented: $adding) {
      // Pre-select the kind the list is filtered to; "All" falls back to Piece.
      LibraryAddScreen(defaultKind: store.viewModel?.activeQuery?.itemType ?? .piece)
        .environment(store)
    }
    // Key on the id (not the value) so an edit — which changes the item's hash —
    // doesn't break the pushed destination; the detail re-reads the fresh item.
    .navigationDestination(for: String.self) { id in
      if let found = items.first(where: { $0.id == id }) {
        LibraryDetailScreen(item: found)
      }
    }
    .onChange(of: searchText) { _, newValue in
      sendQuery(kind: store.viewModel?.activeQuery?.itemType, text: newValue)
    }
  }

  @ViewBuilder private var content: some View {
    if items.isEmpty {
      PlaceholderContent(
        systemImage: isSearching ? "magnifyingglass" : "books.vertical",
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

  private var isSearching: Bool {
    !(store.viewModel?.activeQuery?.text ?? "").isEmpty
  }

  private var emptyMessage: String {
    if let text = store.viewModel?.activeQuery?.text, !text.isEmpty {
      return "No items match “\(text)”."
    }
    switch LibraryFilter(kind: store.viewModel?.activeQuery?.itemType) {
    case .all: return "Your pieces and exercises will live here."
    case .pieces: return "No pieces yet."
    case .exercises: return "No exercises yet."
    }
  }

  /// Build the combined query from the current type filter and search text so
  /// neither resets the other; an empty filter + empty text clears the query.
  private func sendQuery(kind: ItemKind?, text: String) {
    let trimmed = text.trimmingCharacters(in: .whitespaces)
    let query =
      (kind == nil && trimmed.isEmpty)
      ? nil
      : ListQuery(text: trimmed.isEmpty ? nil : trimmed, itemType: kind, key: nil, tags: [])
    store.send(.setQuery(query))
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
