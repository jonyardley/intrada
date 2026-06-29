import SharedTypes
import SwiftUI

/// The shared Library/builder browse header: type pills, sort menu, tag filter,
/// and a button-revealed search bar — all writing the core's shared `activeQuery`
/// / `activeSort` (#792). Extracted from LibraryScreen so the builder reuses it
/// instead of cloning the orchestration (#942).
struct BrowseControlsBar: View {
  @Environment(Store.self) private var store
  private let elevated: Bool
  // Opt-in leading "priorities only" star — only the Library passes it; the
  // session builder reuses this bar without it.
  private let starFilter: Binding<Bool>?
  @State private var filtering = false
  @State private var searchText: String
  @State private var searchRevealed: Bool
  @FocusState private var searchFocused: Bool

  init(elevated: Bool = false, previewSearch: String? = nil, starFilter: Binding<Bool>? = nil) {
    self.elevated = elevated
    self.starFilter = starFilter
    _searchText = State(initialValue: previewSearch ?? "")
    _searchRevealed = State(initialValue: previewSearch != nil)
  }

  private var activeTags: [String] { store.viewModel?.activeQuery?.tags ?? [] }

  private var filterBinding: Binding<LibraryFilter> {
    Binding(
      get: { LibraryFilter(kind: store.viewModel?.activeQuery?.itemType) },
      set: { sendQuery(kind: $0.kind, text: searchText) })
  }

  var body: some View {
    VStack(spacing: 0) {
      // Shadow + zIndex stay on the header alone so the revealed search bar
      // slides out from *under* it rather than sharing its elevation.
      elevatedHeader
        .zIndex(1)
      if searchRevealed {
        LibrarySearchBar(text: $searchText, focused: $searchFocused, onCancel: cancelSearch)
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.bottom, IntradaSpacing.cardCompact)
          .background(IntradaColor.paperTop)
          .transition(.move(edge: .top).combined(with: .opacity))
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
  }

  @ViewBuilder private var elevatedHeader: some View {
    if elevated {
      header.cardShadow()
    } else {
      header
    }
  }

  private var header: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      if let starFilter {
        Button {
          starFilter.wrappedValue.toggle()
        } label: {
          Image(systemName: starFilter.wrappedValue ? "star.fill" : "star")
            .font(IntradaFont.tab)
            .foregroundStyle(starFilter.wrappedValue ? IntradaColor.accent : IntradaColor.inkFaint)
            .padding(.vertical, 6)
            .padding(.horizontal, 10)
            .overlay(Capsule().stroke(IntradaColor.divider, lineWidth: 1))
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Show priorities only")
        .accessibilityAddTraits(starFilter.wrappedValue ? [.isSelected] : [])
      }
      LibraryFilterMenu(current: filterBinding.wrappedValue, onChange: { filterBinding.wrappedValue = $0 })
        .padding(.leading, IntradaSpacing.controlGap)
      Spacer(minLength: IntradaSpacing.controlGap)
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
  }

  private func toggleSearch() {
    if searchRevealed {
      cancelSearch()
    } else {
      withAnimation(IntradaMotion.standard) { searchRevealed = true }
      searchFocused = true
    }
  }

  private func cancelSearch() {
    searchText = ""
    searchFocused = false
    withAnimation(IntradaMotion.standard) { searchRevealed = false }
  }

  // Change one dimension while preserving the others, so the three filters
  // (type / search / tags) don't reset each other.
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
