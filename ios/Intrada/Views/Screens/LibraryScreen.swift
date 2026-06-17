import SharedTypes
import SwiftUI

struct LibraryScreen: View {
  @Environment(Store.self) private var store
  @State private var adding = false
  @State private var prioritiesExpanded = true
  private let previewSearch: String?

  init() { previewSearch = nil }

  #if DEBUG
    /// Preview/snapshot seed: render with the search bar already revealed.
    init(previewSearch: String) { self.previewSearch = previewSearch }
  #endif

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }

  var body: some View {
    ScreenScaffold(
      title: "Library", subtitle: subtitle,
      trailing: .init(label: "Add item", action: { adding = true })
    ) {
      VStack(spacing: 0) {
        BrowseControlsBar(previewSearch: previewSearch)
        content
      }
    }
    // The list draws its own serif header, so suppress the nav bar here; the
    // detail keeps it for the back chevron.
    .toolbar(.hidden, for: .navigationBar)
    .sheet(isPresented: $adding) {
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
  }

  @ViewBuilder private var content: some View {
    if items.isEmpty {
      PlaceholderContent(
        systemImage: isSearching ? "magnifyingglass" : "books.vertical",
        message: emptyMessage)
    } else {
      ScrollView {
        LazyVStack(spacing: IntradaSpacing.row) {
          if !priorityItems.isEmpty {
            prioritiesHeader
            if prioritiesExpanded {
              ForEach(priorityItems, id: \.id) { libraryRow($0) }
            }
            if !regularItems.isEmpty {
              Divider().overlay(IntradaColor.divider)
                .padding(.vertical, IntradaSpacing.controlGap)
            }
          }
          ForEach(regularItems, id: \.id) { libraryRow($0) }
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
        .padding(.bottom, IntradaSpacing.card)
      }
      .scrollDismissesKeyboard(.interactively)
      .scrollEdgeShadow()
    }
  }

  // ── Priority view ─────────────────────────────────────────────────────
  // Partition the *visible* items (the core already applied search/filter) so
  // the pinned section naturally respects the active query.

  private var priorityItems: [LibraryItemView] { items.filter(\.priority) }
  private var regularItems: [LibraryItemView] { items.filter { !$0.priority } }

  private var prioritiesHeader: some View {
    Button {
      withAnimation(.easeInOut(duration: 0.2)) { prioritiesExpanded.toggle() }
    } label: {
      HStack(spacing: 6) {
        Image(systemName: "star.fill").font(.system(size: 11))
          .foregroundStyle(IntradaColor.accent)
        Text("PRIORITIES · \(priorityItems.count)")
          .font(IntradaFont.badge).tracking(1.5)
          .foregroundStyle(IntradaColor.inkFaint)
        Spacer()
        Image(systemName: prioritiesExpanded ? "chevron.down" : "chevron.right")
          .font(.system(size: 12, weight: .semibold))
          .foregroundStyle(IntradaColor.inkFaint)
      }
      .padding(.horizontal, 4)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel("Priorities, \(priorityItems.count) items")
    .accessibilityHint(prioritiesExpanded ? "Collapses the section" : "Expands the section")
  }

  private func libraryRow(_ item: LibraryItemView) -> some View {
    NavigationLink(value: item.id) { LibraryItemCard(item: item) }
      .buttonStyle(.plain)
      .overlay(alignment: .topTrailing) { priorityStar(item) }
  }

  // A separate tap target on top of the row's NavigationLink: tapping the star
  // toggles priority; tapping the rest of the row still navigates to detail.
  private func priorityStar(_ item: LibraryItemView) -> some View {
    Button {
      store.send(.item(.update(id: item.id, input: togglePriority(item))))
    } label: {
      Image(systemName: item.priority ? "star.fill" : "star")
        .font(.system(size: 16))
        .foregroundStyle(item.priority ? IntradaColor.accent : IntradaColor.inkFaint)
        .padding(IntradaSpacing.row)
        .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel(
      item.priority ? "Remove \(item.title) from priorities"
        : "Add \(item.title) to priorities")
  }

  // Priority-only update: title + kind are required, every optional field is
  // "no change" (outer nil), priority flips. The optimistic write reconciles via
  // the ViewModel (the star reflects item.priority); a failure surfaces on the
  // global error banner — never a silent no-op (#846).
  private func togglePriority(_ item: LibraryItemView) -> UpdateItem {
    UpdateItem(
      title: item.title, kind: item.itemType,
      composer: nil, key: nil, modality: nil, tempo: nil, notes: nil,
      tags: nil, priority: !item.priority)
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
