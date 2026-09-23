import SharedTypes
import SwiftUI

/// "Add to session" sheet: browse the library and tap to add/remove items from
/// the building setlist. A piece brings its related exercises as a group (the
/// core forms the block); the shell only sends add/remove.
struct AddToSessionSheet: View {
  @Environment(Store.self) private var store

  private var visibleItems: [LibraryItemView] { store.viewModel?.visibleItems ?? [] }
  private var recentlyPractised: [LibraryItemView] {
    store.viewModel?.recentlyPractisedItems ?? []
  }
  private var entries: [SetlistEntryView] { store.viewModel?.buildingSetlist?.entries ?? [] }
  private var entryByItem: [String: String] {
    Dictionary(entries.map { ($0.itemId, $0.id) }, uniquingKeysWith: { first, _ in first })
  }

  var body: some View {
    BottomSheet(title: "Add to session", detents: [.large]) {
      VStack(spacing: 0) {
        BrowseControlsBar(elevated: true, showsStarFilter: true)
        library
      }
    }
    .libraryQueryScope()
  }

  @ViewBuilder private var library: some View {
    let rows = visibleItems
    if rows.isEmpty {
      PlaceholderContent(systemImage: emptyIcon, message: emptyMessage)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    } else {
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
          recentlyPractisedSection
          if showsRecentlyPractised { SectionHeader(title: "Library") }
          Text("Pieces bring their related exercises as a group.")
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
          LazyVStack(spacing: IntradaSpacing.cardCompact) {
            ForEach(rows, id: \.id) { libraryRow($0) }
          }
        }
        .padding(IntradaSpacing.card)
      }
      .scrollDismissesKeyboard(.interactively)
    }
  }

  private var isFiltered: Bool { store.viewModel?.activeQuery != nil }

  private var showsRecentlyPractised: Bool {
    !isFiltered && !recentlyPractised.isEmpty
  }

  @ViewBuilder private var recentlyPractisedSection: some View {
    if showsRecentlyPractised {
      VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
        SectionHeader(title: "Recently practised")
        VStack(spacing: IntradaSpacing.cardCompact) {
          ForEach(recentlyPractised, id: \.id) { libraryRow($0) }
        }
        HairlineDivider()
      }
    }
  }

  private func libraryRow(_ item: LibraryItemView) -> some View {
    SelectableLibraryRow(
      item: item, added: entryByItem[item.id] != nil,
      addHint: "Adds it to the session",
      removeHint: "Removes it from the session"
    ) {
      toggle(item)
    }
  }

  private func toggle(_ item: LibraryItemView) {
    let event: Event =
      entryByItem[item.id].map { .session(.removeFromSetlist(entryId: $0)) }
      ?? .session(.addToSetlist(itemId: item.id))
    store.send(event, onSuccess: .impact)
  }

  private var isSearching: Bool { !(store.viewModel?.activeQuery?.text ?? "").isEmpty }

  private var priorityOnly: Bool { store.viewModel?.activeQuery?.priorityOnly ?? false }

  private var emptyIcon: String {
    if isSearching { return "magnifyingglass" }
    return priorityOnly ? "star" : "books.vertical"
  }

  private var emptyMessage: String {
    if let text = store.viewModel?.activeQuery?.text, !text.isEmpty {
      return "No items match “\(text)”."
    }
    if priorityOnly {
      return "No priorities yet. Swipe a row to add it to priorities."
    }
    return "The library is empty · add pieces and exercises first."
  }
}
