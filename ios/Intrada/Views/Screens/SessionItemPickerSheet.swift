import SharedTypes
import SwiftUI

struct SessionItemPickerSheet: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var addError: String?
  @State private var filtering = false
  @State private var searchText = ""
  @State private var searchRevealed = false
  @FocusState private var searchFocused: Bool

  init() {}

  #if DEBUG
    init(previewSearch: String) {
      _searchText = State(initialValue: previewSearch)
      _searchRevealed = State(initialValue: true)
    }
  #endif

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }

  // Binary membership: the core doesn't dedupe by item id (#939); keep first.
  private var entryByItem: [String: String] {
    let entries = store.viewModel?.buildingSetlist?.entries ?? []
    return Dictionary(entries.map { ($0.itemId, $0.id) }, uniquingKeysWith: { first, _ in first })
  }

  private var addedCount: Int { store.viewModel?.buildingSetlist?.entries.count ?? 0 }
  private var activeTags: [String] { store.viewModel?.activeQuery?.tags ?? [] }

  // Core owns the filter (#792): read activeQuery, write back via setQuery — the
  // same shared query the Library tab uses.
  private var filterBinding: Binding<LibraryFilter> {
    Binding(
      get: { LibraryFilter(kind: store.viewModel?.activeQuery?.itemType) },
      set: { sendQuery(kind: $0.kind, text: searchText) })
  }

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        VStack(spacing: 0) {
          browseControls
          if let addError {
            FormErrorBanner(message: addError)
              .padding(.horizontal, IntradaSpacing.card)
              .padding(.top, IntradaSpacing.card)
          }
          content
        }
      }
      .navigationTitle("Add to session")
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .topBarTrailing) {
          Button(addedCount > 0 ? "Done · \(addedCount)" : "Done") { dismiss() }
            .accessibilityIdentifier("sessionPickerDone")
        }
      }
      .sensoryFeedback(.selection, trigger: searchRevealed)
      .sheet(isPresented: $filtering) {
        TagFilterSheet(
          available: store.viewModel?.availableTags ?? [],
          selected: activeTags,
          onChange: sendTagFilter)
      }
      .onChange(of: searchText) { _, newValue in
        sendQuery(kind: store.viewModel?.activeQuery?.itemType, text: newValue)
      }
      .onAppear { revealInheritedSearch() }
    }
  }

  // The shared query outlives this sheet's local state, so reflect an inherited
  // search on open — otherwise a filtered list sits behind a collapsed bar (#936).
  private func revealInheritedSearch() {
    let inherited = store.viewModel?.activeQuery?.text ?? ""
    if !inherited.isEmpty {
      searchText = inherited
      searchRevealed = true
    }
  }

  private var browseControls: some View {
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
      // Opaque + on top so the search bar emerges from behind the pills.
      .background(IntradaColor.paperTop)
      .zIndex(1)
      if searchRevealed {
        LibrarySearchBar(text: $searchText, focused: $searchFocused, onCancel: cancelSearch)
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.bottom, IntradaSpacing.cardCompact)
          .background(IntradaColor.paperTop)
          .transition(.move(edge: .top).combined(with: .opacity))
      }
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
            row(item)
          }
        }
        .padding(IntradaSpacing.card)
      }
      .scrollDismissesKeyboard(.interactively)
      .scrollEdgeShadow()
    }
  }

  private func row(_ item: LibraryItemView) -> some View {
    let added = entryByItem[item.id] != nil
    return Button {
      toggle(item)
    } label: {
      LibraryItemCard(item: item)
        .overlay(alignment: .trailing) {
          Image(systemName: added ? "checkmark.circle.fill" : "plus.circle")
            .font(.title2)
            .foregroundStyle(added ? IntradaColor.accent : IntradaColor.inkFaint)
            .padding(.trailing, IntradaSpacing.card)
            .accessibilityHidden(true)
        }
        .overlay(
          RoundedRectangle(cornerRadius: IntradaRadius.card)
            .stroke(IntradaColor.accent, lineWidth: 2)
            .opacity(added ? 1 : 0)
        )
    }
    .buttonStyle(.plain)
    .accessibilityValue(added ? "Added" : "Not added")
    .accessibilityHint(added ? "Removes it from the session" : "Adds it to the session")
  }

  // RootView's error banner sits behind this sheet — surface failures here.
  private func toggle(_ item: LibraryItemView) {
    let before = store.viewModel?.error
    if let entryId = entryByItem[item.id] {
      store.send(.session(.removeFromSetlist(entryId: entryId)))
    } else {
      store.send(.session(.addToSetlist(itemId: item.id)))
    }
    if let error = store.viewModel?.error, error != before {
      addError = error
    } else {
      addError = nil
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
  }

  private func toggleSearch() {
    if searchRevealed {
      cancelSearch()
    } else {
      withAnimation(.spring(response: 0.35, dampingFraction: 0.85)) { searchRevealed = true }
      searchFocused = true
    }
  }

  private func cancelSearch() {
    searchText = ""
    searchFocused = false
    withAnimation(.spring(response: 0.35, dampingFraction: 0.85)) { searchRevealed = false }
  }

  private var isSearching: Bool {
    !(store.viewModel?.activeQuery?.text ?? "").isEmpty
  }

  private var emptyMessage: String {
    if let text = store.viewModel?.activeQuery?.text, !text.isEmpty {
      return "No items match “\(text)”."
    }
    switch LibraryFilter(kind: store.viewModel?.activeQuery?.itemType) {
    case .all: return "Your library is empty — add pieces and exercises first."
    case .pieces: return "No pieces yet."
    case .exercises: return "No exercises yet."
    }
  }

  private func sendQuery(kind: ItemKind?, text: String) {
    applyQuery(kind: kind, text: text, tags: activeTags)
  }

  private func sendTagFilter(_ tags: [String]) {
    applyQuery(kind: store.viewModel?.activeQuery?.itemType, text: searchText, tags: tags)
  }

  private func applyQuery(kind: ItemKind?, text: String, tags: [String]) {
    let trimmed = text.trimmingCharacters(in: .whitespaces)
    let query =
      (kind == nil && trimmed.isEmpty && tags.isEmpty)
      ? nil
      : ListQuery(text: trimmed.isEmpty ? nil : trimmed, itemType: kind, key: nil, tags: tags)
    store.send(.setQuery(query))
  }
}

#if DEBUG
  #Preview {
    SessionItemPickerSheet()
      .environment(Store.previewLibrary)
  }
#endif
