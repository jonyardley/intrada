import SharedTypes
import SwiftUI

/// "Add to session" sheet: browse the library and tap to add/remove items from
/// the building setlist. A piece brings its related exercises as a group (the
/// core forms the block); the shell only sends add/remove.
struct AddToSessionSheet: View {
  @Environment(Store.self) private var store
  @State private var starFilter = false

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }
  private var displayedItems: [LibraryItemView] {
    starFilter ? items.filter(\.priority) : items
  }
  private var entries: [SetlistEntryView] { store.viewModel?.buildingSetlist?.entries ?? [] }
  private var entryByItem: [String: String] {
    Dictionary(entries.map { ($0.itemId, $0.id) }, uniquingKeysWith: { first, _ in first })
  }

  var body: some View {
    BottomSheet(title: "Add to session", detents: [.large]) {
      VStack(spacing: 0) {
        BrowseControlsBar(elevated: true, starFilter: $starFilter)
        library
      }
    }
  }

  @ViewBuilder private var library: some View {
    if displayedItems.isEmpty {
      PlaceholderContent(systemImage: emptyIcon, message: emptyMessage)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    } else {
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
          Text("Pieces bring their related exercises as a group.")
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
          LazyVStack(spacing: IntradaSpacing.cardCompact) {
            ForEach(displayedItems, id: \.id) { libraryRow($0) }
          }
        }
        .padding(IntradaSpacing.card)
      }
      .scrollDismissesKeyboard(.interactively)
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

  private func toggle(_ item: LibraryItemView) {
    let before = store.viewModel?.errorSeq
    if let entryId = entryByItem[item.id] {
      store.send(.session(.removeFromSetlist(entryId: entryId)))
    } else {
      store.send(.session(.addToSetlist(itemId: item.id)))
    }
    // Only ack once the core confirms — errors surface in RootView's banner (#846).
    if store.viewModel?.errorSeq == before {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
  }

  private var isSearching: Bool { !(store.viewModel?.activeQuery?.text ?? "").isEmpty }

  private var emptyIcon: String {
    if starFilter { return "star" }
    return isSearching ? "magnifyingglass" : "books.vertical"
  }

  private var emptyMessage: String {
    if starFilter && !items.isEmpty {
      return "No priorities yet. Swipe a row to star it."
    }
    if let text = store.viewModel?.activeQuery?.text, !text.isEmpty {
      return "No items match “\(text)”."
    }
    return "Your library is empty — add pieces and exercises first."
  }
}
