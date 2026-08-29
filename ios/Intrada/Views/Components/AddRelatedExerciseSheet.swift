import SharedTypes
import SwiftUI

/// Multi-select picker for the exercises in a session-builder block, opened
/// from the block's own "Add a related exercise" footer row. Same selection
/// model as `AddToSessionSheet` (#1103) — the sheet stays open, rows carry
/// their added state, and tapping an added row takes it back out of the block.
/// Scoped to exercises: those already in the block, plus any not in the
/// setlist at all.
struct AddRelatedExerciseSheet: View {
  let groupId: String
  @Environment(Store.self) private var store
  @State private var queryBeforeSheet: ListQuery?

  private var entries: [SetlistEntryView] { store.viewModel?.buildingSetlist?.entries ?? [] }

  private var blockEntryByItem: [String: String] {
    Dictionary(
      entries.filter { $0.groupId == groupId }.map { ($0.itemId, $0.id) },
      uniquingKeysWith: { first, _ in first })
  }

  private var candidates: [LibraryItemView] {
    let elsewhere = Swift.Set(entries.filter { $0.groupId != groupId }.map(\.itemId))
    return (store.viewModel?.items ?? [])
      .filter { $0.itemType == .exercise && !elsewhere.contains($0.id) }
  }

  var body: some View {
    BottomSheet(title: "Add a related exercise", detents: [.large]) {
      VStack(spacing: 0) {
        BrowseControlsBar(elevated: true, showsTypeFilter: false)
        list
      }
    }
    .onAppear(perform: scopeQueryToExercises)
    .onDisappear { store.send(.setQuery(queryBeforeSheet)) }
  }

  @ViewBuilder private var list: some View {
    if candidates.isEmpty {
      PlaceholderContent(systemImage: emptyIcon, message: emptyMessage)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    } else {
      ScrollView {
        LazyVStack(spacing: IntradaSpacing.cardCompact) {
          ForEach(candidates, id: \.id) { item in
            SelectableLibraryRow(
              item: item, added: blockEntryByItem[item.id] != nil,
              addHint: "Adds it to this block",
              removeHint: "Takes it back out of this block"
            ) {
              toggle(item)
            }
          }
        }
        .padding(IntradaSpacing.card)
      }
      .scrollDismissesKeyboard(.interactively)
    }
  }

  private func toggle(_ item: LibraryItemView) {
    let event: Event =
      blockEntryByItem[item.id].map { .session(.removeFromSetlist(entryId: $0)) }
      ?? .session(.addExerciseToBlock(groupId: groupId, itemId: item.id))
    store.send(event, onSuccess: .impact)
  }

  // The shared library query is whatever Library last left behind; its text or
  // type filter would narrow this sheet with no control here showing why.
  private func scopeQueryToExercises() {
    queryBeforeSheet = store.viewModel?.activeQuery
    store.send(.setQuery(ListQuery(text: nil, itemType: .exercise, key: nil, tags: [])))
  }

  private var searchText: String { store.viewModel?.activeQuery?.text ?? "" }

  private var activeTags: [String] { store.viewModel?.activeQuery?.tags ?? [] }

  private var emptyIcon: String {
    if !searchText.isEmpty { return "magnifyingglass" }
    return activeTags.isEmpty ? "dumbbell" : "line.3.horizontal.decrease.circle"
  }

  private var emptyMessage: String {
    if !searchText.isEmpty { return "No exercises match “\(searchText)”." }
    if !activeTags.isEmpty { return "No exercises carry those tags." }
    return "No other exercises left to add."
  }
}
