import SharedTypes
import SwiftUI

// Library-first builder: browse/search the library up top, tap to add, and the
// setlist forms in the reorderable queue tray at the bottom. `cancelBuilding`
// fires from PracticeScreen's navigation binding on dismiss.
struct SessionBuilderScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var confirmingCancel = false
  @State private var filtering = false
  @State private var searchText = ""
  @State private var searchRevealed = false
  @FocusState private var searchFocused: Bool

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }
  private var entries: [SetlistEntryView] { store.viewModel?.buildingSetlist?.entries ?? [] }
  private var activeTags: [String] { store.viewModel?.activeQuery?.tags ?? [] }

  private var entryByItem: [String: String] {
    let entries = store.viewModel?.buildingSetlist?.entries ?? []
    return Dictionary(entries.map { ($0.itemId, $0.id) }, uniquingKeysWith: { first, _ in first })
  }

  private var filterBinding: Binding<LibraryFilter> {
    Binding(
      get: { LibraryFilter(kind: store.viewModel?.activeQuery?.itemType) },
      set: { sendQuery(kind: $0.kind, text: searchText) })
  }

  var body: some View {
    ZStack {
      PaperBackground()
      VStack(spacing: 0) {
        browseControls
        library
        queueTray
        startBar
      }
    }
    .navigationTitle("New session")
    .navigationBarTitleDisplayMode(.inline)
    .navigationBarBackButtonHidden(true)
    .toolbar {
      ToolbarItem(placement: .topBarLeading) {
        Button("Cancel") { cancel() }
      }
    }
    .sensoryFeedback(.selection, trigger: searchRevealed)
    .sheet(isPresented: $filtering) {
      TagFilterSheet(
        available: store.viewModel?.availableTags ?? [],
        selected: activeTags, onChange: sendTagFilter)
    }
    .onChange(of: searchText) { _, newValue in
      sendQuery(kind: store.viewModel?.activeQuery?.itemType, text: newValue)
    }
    .alert("Discard session plan?", isPresented: $confirmingCancel) {
      Button("Discard", role: .destructive) { dismiss() }
      Button("Keep editing", role: .cancel) {}
    } message: {
      Text("The items you've added will be cleared.")
    }
  }

  // ── Library (browse + add) ───────────────────────────────────────────

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
              ? "line.3.horizontal.decrease.circle" : "line.3.horizontal.decrease.circle.fill"
          )
          .font(IntradaFont.tab)
          .foregroundStyle(activeTags.isEmpty ? IntradaColor.inkFaint : IntradaColor.accent)
          .padding(IntradaSpacing.controlGap)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Filter by tag")
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

  @ViewBuilder private var library: some View {
    if items.isEmpty {
      PlaceholderContent(
        systemImage: isSearching ? "magnifyingglass" : "books.vertical", message: emptyMessage
      )
      .frame(maxWidth: .infinity, maxHeight: .infinity)
    } else {
      ScrollView {
        LazyVStack(spacing: IntradaSpacing.row) {
          ForEach(items, id: \.id) { item in
            libraryRow(item)
          }
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.vertical, IntradaSpacing.card)
      }
      .scrollDismissesKeyboard(.interactively)
      .frame(maxHeight: .infinity)
    }
  }

  private func libraryRow(_ item: LibraryItemView) -> some View {
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
            .stroke(IntradaColor.accent, lineWidth: 2).opacity(added ? 1 : 0))
    }
    .buttonStyle(.plain)
    .accessibilityValue(added ? "Added" : "Not added")
    .accessibilityHint(added ? "Removes it from the session" : "Adds it to the session")
  }

  // ── Queue tray (the setlist, reorderable) ────────────────────────────

  private var queueTray: some View {
    VStack(alignment: .leading, spacing: 0) {
      HStack {
        Text("In this session").font(IntradaFont.cardTitle(16)).foregroundStyle(IntradaColor.ink)
        Spacer()
        Text("\(entries.count)").font(IntradaFont.metaMedium).foregroundStyle(IntradaColor.inkFaint)
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.cardCompact)

      if entries.isEmpty {
        Text("Tap items above to build your session.")
          .font(IntradaFont.meta).foregroundStyle(IntradaColor.inkFaint)
          .frame(maxWidth: .infinity, alignment: .leading)
          .padding(IntradaSpacing.card)
      } else {
        List {
          ForEach(entries, id: \.id) { entry in
            SetlistQueueRow(entry: entry) {
              store.send(.session(.removeFromSetlist(entryId: entry.id)))
            }
            .listRowBackground(Color.clear)
            .listRowSeparator(.hidden)
            .listRowInsets(
              EdgeInsets(
                top: 0, leading: IntradaSpacing.card, bottom: 0, trailing: IntradaSpacing.card))
          }
          .onMove(perform: move)
        }
        .listStyle(.plain)
        .scrollContentBackground(.hidden)
        // Always-on reorder grips. No `.onDelete` — remove is the inline button,
        // so edit mode adds no second delete control.
        .environment(\.editMode, .constant(.active))
        .frame(height: min(CGFloat(entries.count) * 44 + 8, 176))
      }
    }
    .frame(maxWidth: .infinity)
    .background(IntradaColor.paperTop)
    .overlay(alignment: .top) { Rectangle().fill(IntradaColor.hairline).frame(height: 1) }
  }

  private var startBar: some View {
    VStack(spacing: 6) {
      Label("Start session", systemImage: "play.fill")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.onAccent)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.row)
        .background(LinearGradient.brandBar)
        .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
        .opacity(0.5)
      Text("Coming soon")
        .font(IntradaFont.micro).foregroundStyle(IntradaColor.inkFaint)
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Start session, coming soon")
    .padding(IntradaSpacing.card)
    .background(IntradaColor.paperTop)
  }

  // ── Actions ──────────────────────────────────────────────────────────

  private func toggle(_ item: LibraryItemView) {
    let before = store.viewModel?.error
    if let entryId = entryByItem[item.id] {
      store.send(.session(.removeFromSetlist(entryId: entryId)))
    } else {
      store.send(.session(.addToSetlist(itemId: item.id)))
    }
    // Only ack the tap once the core confirms (errors surface in RootView's banner).
    if store.viewModel?.error == before {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
  }

  // SwiftUI's `to` is an insert-before index; reorderSetlist wants a final index.
  private func move(_ source: IndexSet, to destination: Int) {
    guard let from = source.first else { return }
    let target = from < destination ? destination - 1 : destination
    store.send(.session(.reorderSetlist(entryId: entries[from].id, newPosition: UInt64(target))))
  }

  private func cancel() {
    if entries.isEmpty { dismiss() } else { confirmingCancel = true }
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

  private var isSearching: Bool { !(store.viewModel?.activeQuery?.text ?? "").isEmpty }

  private var emptyMessage: String {
    if let text = store.viewModel?.activeQuery?.text, !text.isEmpty {
      return "No items match “\(text)”."
    }
    return "Your library is empty — add pieces and exercises first."
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
  #Preview("Populated") {
    NavigationStack { SessionBuilderScreen() }.environment(Store.previewBuilding)
  }
#endif
