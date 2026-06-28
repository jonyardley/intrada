import SharedTypes
import SwiftUI

struct LibraryScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @State private var adding = false
  @State private var prioritiesExpanded = true
  private let previewSearch: String?

  // Shared BrowseControlsBar spring — one motion vocabulary (honours Reduce Motion).
  private static let motion = Animation.spring(response: 0.35, dampingFraction: 0.85)

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
        VStack(spacing: 0) {
          prioritySection
          LazyVStack(spacing: IntradaSpacing.cardCompact) {
            ForEach(regularItems, id: \.id) { libraryRow($0) }
          }
          .padding(.horizontal, IntradaSpacing.card)
          // Flush under the toolbar — no mismatched page-colour strip above it.
          .padding(.top, priorityItems.isEmpty ? IntradaSpacing.card : 0)
        }
        .padding(.bottom, IntradaSpacing.card)
      }
      .scrollDismissesKeyboard(.interactively)
      .scrollEdgeShadow()
    }
  }

  // Own VStack (LazyVStack ignores zIndex) so the header layers above the cards
  // on collapse; full-bleed header, inset cards.
  @ViewBuilder private var prioritySection: some View {
    if !priorityItems.isEmpty {
      VStack(spacing: 0) {
        prioritiesHeader.zIndex(1)
        if prioritiesExpanded {
          VStack(spacing: IntradaSpacing.cardCompact) {
            ForEach(priorityItems, id: \.id) { libraryRow($0) }
          }
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.top, IntradaSpacing.cardCompact)
          .transition(.move(edge: .top).combined(with: .opacity))
        }
      }
      .clipped()
      if !regularItems.isEmpty {
        Divider().overlay(IntradaColor.divider)
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.vertical, IntradaSpacing.controlGap)
      }
    }
  }

  // Partition the already query-filtered items, so the section respects search/filter.
  private var priorityItems: [LibraryItemView] { items.filter(\.priority) }
  private var regularItems: [LibraryItemView] { items.filter { !$0.priority } }

  private var prioritiesHeader: some View {
    Button {
      withAnimation(reduceMotion ? nil : Self.motion) { prioritiesExpanded.toggle() }
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
      .padding(.vertical, IntradaSpacing.cardCompact)
      .padding(.horizontal, IntradaSpacing.card)
      .frame(maxWidth: .infinity, alignment: .leading)
      // Flat, opaque, full-bleed like the toolbar so collapsing cards tuck under it.
      .background(IntradaColor.paperTop)
      .overlay(alignment: .bottom) { Rectangle().fill(IntradaColor.hairline).frame(height: 1) }
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel("Priorities, \(priorityItems.count) items")
    .accessibilityHint(prioritiesExpanded ? "Collapses the section" : "Expands the section")
  }

  // Star is a sibling of the NavigationLink (not nested) so the link can't steal its taps.
  private func libraryRow(_ item: LibraryItemView) -> some View {
    ZStack(alignment: .topTrailing) {
      NavigationLink(value: item.id) { LibraryItemCard(item: item, trailingGutter: 40) }
        .buttonStyle(.plain)
      priorityStar(item)
    }
  }

  private func priorityStar(_ item: LibraryItemView) -> some View {
    Button {
      toggleStar(item.id)
    } label: {
      Image(systemName: item.priority ? "star.fill" : "star")
        .font(.system(size: 18, weight: .medium))
        .foregroundStyle(item.priority ? IntradaColor.accent : IntradaColor.inkFaint)
        .padding(IntradaSpacing.row)
        .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel(
      item.priority ? "Remove \(item.title) from priorities"
        : "Add \(item.title) to priorities")
  }

  // Read the item fresh at tap time — a render-captured value can go stale.
  private func toggleStar(_ id: String) {
    guard let item = store.viewModel?.items.first(where: { $0.id == id }) else { return }
    withAnimation(reduceMotion ? nil : Self.motion) {
      store.send(.item(.update(id: id, input: togglePriority(item))))
    }
  }

  // Priority-only update: every optional field is "no change" (nil), priority flips.
  // A failed write surfaces on the global banner, not a silent no-op (#846).
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
