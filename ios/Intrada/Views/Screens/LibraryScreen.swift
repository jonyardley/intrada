import SharedTypes
import SwiftUI

struct LibraryScreen: View {
  @Environment(Store.self) private var store
  @State private var adding = false
  @State private var filtering = false
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
        HStack(spacing: IntradaSpacing.controlGap) {
          LibraryFilterTabs(selection: filterBinding, edgeInset: IntradaSpacing.card)
            .frame(maxWidth: .infinity, alignment: .leading)
          LibrarySortMenu(
            current: store.viewModel?.activeSort
              ?? LibrarySort(field: .dateAdded, direction: .descending),
            onChange: { store.send(.setSort($0)) })
          Button {
            filtering = true
          } label: {
            Image(
              systemName: activeTags.isEmpty
                ? "line.3.horizontal.decrease.circle"
                : "line.3.horizontal.decrease.circle.fill"
            )
            .font(IntradaFont.tab)
            .foregroundStyle(activeTags.isEmpty ? IntradaColor.inkFaint : IntradaColor.accent)
            .padding(IntradaSpacing.controlGap)
          }
          .buttonStyle(.plain)
          .accessibilityLabel("Filter by tag")
          .accessibilityValue(activeTags.isEmpty ? "Off" : "\(activeTags.count) selected")
          Button(action: toggleSearch) {
            Image(systemName: "magnifyingglass")
              .font(IntradaFont.tab)
              .foregroundStyle(searchRevealed ? IntradaColor.accent : IntradaColor.inkFaint)
              .padding(IntradaSpacing.controlGap)
          }
          .buttonStyle(.plain)
          .accessibilityLabel("Search")
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.cardCompact)
        .padding(.bottom, IntradaSpacing.row)
        // Opaque + on top so the bar emerges from behind the pills rather than
        // ghosting over them (see Design System Rules → animated reveals).
        .background(IntradaColor.paperTop)
        .zIndex(1)
        if searchRevealed {
          LibrarySearchBar(text: $searchText, focused: $searchFocused, onCancel: cancelSearch)
            .padding(.horizontal, IntradaSpacing.card)
            .padding(.bottom, IntradaSpacing.cardCompact)
            .background(IntradaColor.paperTop)
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
      LibraryAddScreen(defaultKind: store.viewModel?.activeQuery?.itemType ?? .piece)
        .environment(store)
    }
    .sheet(isPresented: $filtering) {
      TagFilterSheet(
        available: store.viewModel?.availableTags ?? [],
        selected: activeTags,
        onChange: sendTagFilter)
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
        LazyVStack(spacing: IntradaSpacing.row) {
          ForEach(items, id: \.id) { item in
            NavigationLink(value: item.id) {
              LibraryItemCard(item: item)
            }
            .buttonStyle(.plain)
          }
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
        .padding(.bottom, IntradaSpacing.card)
      }
      .scrollDismissesKeyboard(.interactively)
      .scrollEdgeShadow()
    }
  }

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

  /// The currently active tag filter (OR-matched in the core).
  private var activeTags: [String] {
    store.viewModel?.activeQuery?.tags ?? []
  }

  /// Change the type filter / search text while preserving the active tag
  /// filter, so the three dimensions don't reset each other.
  private func sendQuery(kind: ItemKind?, text: String) {
    applyQuery(kind: kind, text: text, tags: activeTags)
  }

  /// Change the tag filter while preserving the active type filter + search.
  private func sendTagFilter(_ tags: [String]) {
    applyQuery(kind: store.viewModel?.activeQuery?.itemType, text: searchText, tags: tags)
  }

  /// Build the combined query from all three dimensions; all-empty clears it.
  private func applyQuery(kind: ItemKind?, text: String, tags: [String]) {
    let trimmed = text.trimmingCharacters(in: .whitespaces)
    let query =
      (kind == nil && trimmed.isEmpty && tags.isEmpty)
      ? nil
      : ListQuery(text: trimmed.isEmpty ? nil : trimmed, itemType: kind, key: nil, tags: tags)
    store.send(.setQuery(query))
  }

  private var subtitle: String? {
    guard let vm = store.viewModel else { return nil }
    let parts = [
      count(Int(vm.visiblePieces), "piece"), count(Int(vm.visibleExercises), "exercise"),
    ]
    .compactMap { $0 }
    if parts.isEmpty {
      return (vm.activeQuery == nil) ? "No items yet" : "No matches"
    }
    return parts.joined(separator: " · ")
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
