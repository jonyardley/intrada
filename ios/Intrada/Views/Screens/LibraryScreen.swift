import SharedTypes
import SwiftUI

struct LibraryScreen: View {
  @Environment(Store.self) private var store
  @State private var adding = false
  @State private var searchText = ""
  @State private var searchRevealed = false
  @FocusState private var searchFocused: Bool

  init() {}

  #if DEBUG
    /// Preview/snapshot seed: render with the search bar already revealed and a
    /// query in flight, so the searching state has its own visual regression test.
    init(previewSearch: String) {
      _searchText = State(initialValue: previewSearch)
      _searchRevealed = State(initialValue: true)
    }
  #endif

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }

  // Core owns the filter: read `activeQuery`, write back via `setQuery` (#792).
  private var filterBinding: Binding<LibraryFilter> {
    Binding(
      get: { LibraryFilter(kind: store.viewModel?.activeQuery?.itemType) },
      set: { sendQuery(kind: $0.kind, text: searchText) })
  }

  var body: some View {
    ScreenScaffold(
      title: "Library", subtitle: subtitle,
      trailing: .init(label: "Add item", action: { adding = true })
    ) {
      VStack(spacing: 0) {
        HStack(spacing: 8) {
          LibraryFilterTabs(selection: filterBinding)
            .frame(maxWidth: .infinity, alignment: .leading)
          LibrarySortMenu(
            current: store.viewModel?.activeSort
              ?? LibrarySort(field: .dateAdded, direction: .descending),
            onChange: { store.send(.setSort($0)) })
          Button(action: toggleSearch) {
            Image(systemName: "magnifyingglass")
              .font(IntradaFont.tab)
              .foregroundStyle(searchRevealed ? IntradaColor.accent : IntradaColor.inkFaint)
              .padding(8)
          }
          .buttonStyle(.plain)
          .accessibilityLabel("Search")
        }
        .padding(.horizontal, 16)
        .padding(.top, 12)
        .padding(.bottom, 14)
        if searchRevealed {
          LibrarySearchBar(text: $searchText, focused: $searchFocused, onCancel: cancelSearch)
            .padding(.horizontal, 16)
            .padding(.bottom, 12)
            .transition(.move(edge: .top).combined(with: .opacity))
        }
        content
      }
    }
    // The list draws its own serif header, so suppress the nav bar here; the
    // detail keeps it for the back chevron.
    .toolbar(.hidden, for: .navigationBar)
    .sensoryFeedback(.selection, trigger: searchRevealed)
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
      .scrollDismissesKeyboard(.interactively)
      .scrollEdgeShadow()
    }
  }

  /// Magnifier button: reveal + focus the field, or (when already open) dismiss.
  private func toggleSearch() {
    if searchRevealed {
      cancelSearch()
    } else {
      withAnimation(.spring(response: 0.35, dampingFraction: 0.85)) {
        searchRevealed = true
      }
      searchFocused = true
    }
  }

  private func cancelSearch() {
    searchText = ""
    searchFocused = false
    withAnimation(.spring(response: 0.35, dampingFraction: 0.85)) {
      searchRevealed = false
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

  #Preview("Searching") {
    NavigationStack { LibraryScreen(previewSearch: "clair") }
      .environment(Store.previewLibrarySearching)
  }

  #Preview("Empty") {
    NavigationStack { LibraryScreen() }
      .environment(Store.preview)
  }
#endif
